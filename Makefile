TGT=i686-runix-target
QEMU_TGT=i386
KERNEL_NAME=rkrnl

.PHONY: build iso vm setup tests run clean help

# Subcommand helpers: `make iso test`, `make iso build`, `make vm launch`...
# The second word of MAKECMDGOALS is the subcommand; read it into $(SUB) here.
SUB=$(word 2,$(MAKECMDGOALS))
IS_SUBCOMMAND_GOAL=$(filter iso vm setup,$(firstword $(MAKECMDGOALS)))

# Swallow the subcommand goal (`make iso test` -> `test`) so make does not
# resolve it via its own rules.
ifneq ($(IS_SUBCOMMAND_GOAL),)
ifneq ($(SUB),)
.PHONY: $(SUB)
$(SUB): ;@:
endif
endif

# Prefer rustup's nightly. Termux has no rustup package, so fall back to the
# distro stable rustc: the rust-src package + RUSTC_BOOTSTRAP=1 builds the
# same kernel with the same -Z flags.
ifneq ($(shell command -v rustup 2>/dev/null),)
CARGO_BUILD = cargo +nightly build
else
CARGO_BUILD = RUSTC_BOOTSTRAP=1 cargo build
endif

# Compile the freestanding i686 kernel (build-std, custom target spec).
ifeq ($(IS_SUBCOMMAND_GOAL),)
build:
	@echo "[*] building..."
	@$(CARGO_BUILD) \
	-Z build-std=core \
	-Z build-std-features=compiler-builtins-mem \
	-Z json-target-spec
	@echo "[*] done!"
endif

iso:
	@test -n "$(SUB)" || { echo "usage: make iso test|build"; exit 1; }
	@bash scripts/iso.sh "$(SUB)"

vm:
	@test -n "$(SUB)" || { echo "usage: make vm launch|agent|screen|stop"; exit 1; }
	@bash scripts/vm.sh "$(SUB)"

setup:
	@test -n "$(SUB)" || { echo "usage: make setup android|arch"; exit 1; }
	@bash scripts/setup.sh "$(SUB)"

# Run the host-side kernel module tests (tests/ crate). --target follows the
# real host triple (CLI overrides the tests/.cargo/config.toml fallback).
tests:
	@echo "[*] running kernel module tests on host..."
	@cd tests && cargo test --target "$$(rustc -vV | sed -n 's/^host: //p')"

# Boot runix.iso in QEMU with a visible GTK window.
run:
	@echo "[*] starting QEMU with built ISO..."
	@qemu-system-$(QEMU_TGT) -cdrom runix.iso -display gtk

# Remove all build artifacts (kernel, ISO, host-test target).
clean:
	@echo "[*] cleaning..."
	@echo "	[*] cargo"
	@cargo clean
	@echo "	[*] ISO"
	@rm runix.iso iso/boot/$(KERNEL_NAME) -f
	@echo "	[*] tests"
	@rm tests/target -rf

# List every supported command with a one-line description.
help:
	@echo -e "available cmds:\n\
	 build: build kernel\n\
	 iso test: validate kernel image (ELF + multiboot2)\n\
	 iso build: pack iso/boot into runix.iso\n\
	 vm launch: boot runix.iso in QEMU (gtk)\n\
	 vm agent: boot headless with a monitor socket for agent use\n\
	 vm screen: dump the VM's VGA text via the monitor socket\n\
	 vm stop: stop the agent VM\n\
	 setup android|arch: install dev dependencies (Termux / Arch Linux)\n\
	 tests: run the host-side kernel module tests\n\
	 run: alias of vm launch\n\
	 clean: cleanup"

