TGT=i686-runix-target
QEMU_TGT=i386
KERNEL_NAME=rkrnl

.PHONY: build iso vm tests run clean help

# Subcommand helpers: `make iso test`, `make iso build`, `make vm launch`...
# The second word of MAKECMDGOALS is the subcommand; read it into $(SUB) here
# (the swallowing rule for it lives at the bottom of this file).
SUB=$(word 2,$(MAKECMDGOALS))

build:
	@echo "[*] building..."
	@cargo +nightly build \
	-Z build-std=core \
	-Z build-std-features=compiler-builtins-mem \
	-Z json-target-spec
	@echo "[*] done!"

iso:
	@test -n "$(SUB)" || { echo "usage: make iso test|build"; exit 1; }
	@bash scripts/iso.sh "$(SUB)"

vm:
	@test -n "$(SUB)" || { echo "usage: make vm launch|agent|screen|stop"; exit 1; }
	@bash scripts/vm.sh "$(SUB)"

tests:
	@echo "[*] running kernel module tests on host..."
	@cd tests && cargo test

run:
	@echo "[*] starting QEMU with built ISO..."
	@qemu-system-$(QEMU_TGT) -cdrom runix.iso -display gtk

clean:
	@echo "[*] cleaning..."
	@echo "	[*] cargo"
	@cargo clean
	@echo "	[*] ISO"
	@rm runix.iso iso/boot/$(KERNEL_NAME) -f
	@echo "	[*] tests"
	@rm tests/target -rf

help:
	@echo -e "available cmds:\n\
	 build: build kernel\n\
	 iso test: validate kernel image (ELF + multiboot2)\n\
	 iso build: pack iso/boot into runix.iso\n\
	 vm launch: boot runix.iso in QEMU (gtk)\n\
	 vm agent: boot headless with a monitor socket for agent use\n\
	 vm screen: dump the VM's VGA text via the monitor socket\n\
	 vm stop: stop the agent VM\n\
	 tests: run the host-side kernel module tests\n\
	 run: alias of vm launch\n\
	 clean: cleanup"

# Swallow the subcommand goal (`make iso test` -> `test`) so make does not
# resolve it via its own rules. This must live last: a recipe here overrides
# real targets that share a name (e.g. `build`), otherwise `make iso build`
# would also kick off the kernel build in the wrong order.
ifeq ($(filter iso vm,$(firstword $(MAKECMDGOALS))),$(firstword $(MAKECMDGOALS)))
ifneq ($(SUB),)
.PHONY: $(SUB)
$(SUB): ;@:
endif
endif
