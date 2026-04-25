build:
	cargo build
	cp target/x86_64-unknown-none/debug/runix iso/boot/kernel.bin -f
	rm runix.iso
	grub-mkrescue -o runix.iso iso
run:
	qemu-system-i386 -cdrom runix.iso