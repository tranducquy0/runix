# TODO

Next milestones, roughly in dependency order. None of this exists yet — the kernel
currently relies on GRUB's flat setup, prints VGA text only, and idles after the frame
allocator smoke test.

- [ ] **GDT** — install our own flat segments instead of relying on whatever GRUB left
  in place. Prerequisite for anything ring- or segment-related (IDT, later userspace).
- [ ] **IDT / interrupt dispatch** — lives in
  [`src/subs/handlers/`](src/subs/handlers) next to the panic handler (see
  [docs/architecture.md](docs/architecture.md)). Needs the GDT first, plus a PIC remap
  so IRQs don't collide with CPU exceptions.
- [ ] **Serial output (QEMU)** — a COM1 driver under
  [`src/drivers/output/`](src/drivers/output). Complements VGA and makes headless runs
  (`make vm agent`) readable without dumping the text buffer; today there is no serial
  console fallback (see [docs/boot.md](docs/boot.md) limitations).
- [ ] **Keyboard** — [`src/drivers/input/`](src/drivers/input) is a placeholder today.
  An IRQ1 handler on top of the IDT, then a scancode decoder.
