build:
	@echo "[*] building..."
	@cargo build
	@echo "[*] crafting ISO..."
	@echo " 	[*] getting built kernel..."
	@cp target/x86_64-unknown-none/debug/runix iso/boot/kernel.bin -f
	@echo "		[*] building ISO using grub-mkrescue..."
	@rm runix.iso
	@grub-mkrescue -o runix.iso iso
run:
	@echo "[*] starting QEMU with built ISO..."
	@qemu-system-i386 -cdrom runix.iso