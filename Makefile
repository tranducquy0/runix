TGT=i686-runix-target
QEMU_TGT=i386
KERNEL_NAME=rkrnl
build:
	@echo "[*] building..."
	@cargo +nightly build \
	-Z build-std=core \
	-Z build-std-features=compiler-builtins-mem \
	-Z json-target-spec
	@echo "[*] done!"
build_iso:
	@echo "[*] crafting ISO..."
	@echo "	[*] getting built kernel for packing..."
	@cp target/$(TGT)/debug/runix iso/boot/$(KERNEL_NAME) -f
	@echo "	[*] building ISO using grub-mkrescue..."
	@grub-mkrescue -o runix.iso iso
run:
	@echo "[*] starting QEMU with built ISO..."
	@qemu-system-$(QEMU_TGT) -cdrom runix.iso
clean:
	@echo "[*] cleaning..."
	@echo "	[*] cargo"
	@cargo clean
	@echo "	[*] ISO"
	@rm runix.iso iso/boot/$(KERNEL_NAME) -f

test:
	@echo "[*] testing kernel image"
	@bash tests/kernel_image.sh
