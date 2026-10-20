# Contributing to runix

runix is a hobby i686 kernel in early stages: boot parsing, a physical memory map, and a
frame allocator work; there is no scheduler, paging, interrupts, or userspace yet.
Contributions that make the current scope better — correctness, tests, tooling, docs —
are more useful right now than starting a subsystem the kernel can't run yet.

## What help is welcome

- Fixes and hardening in what exists: `src/subs/boot`, `src/subs/mem`, the VGA driver,
  the linker script, `scripts/`.
- Host tests for logic modules (see [tests/README.md](tests/README.md)).
- Build/VM tooling under `scripts/` and the `Makefile`.
- Documentation that follows the code ([docs/](docs/)).

If you plan something bigger (paging, interrupts, a scheduler), open an issue first —
it needs design decisions that touch the boot path and the allocator.

## Requirements

Rust **nightly**, `grub-mkrescue` + `grub-file`, `readelf`, `qemu-system-i386`, and
`python3` (the `vm screen`/`vm stop` monitor helpers). On Termux or Arch, run
`make setup android|arch` — it installs the toolchain. Details and why each piece
exists: [docs/build.md](docs/build.md).

There is no build/test CI — verification happens on your machine, and the PR has to
carry the evidence.

## Workflow

1. Fork and branch off `master`. History uses short prefixed branches
   (`fix/...`, `ref/...`).
2. Keep the change focused. One concern per commit; split unrelated work
   (a doc restructure vs. a code fix) into separate commits.
3. Verify before you push:

```bash
make build      # compile the kernel
make iso test   # gate: valid ELF + Multiboot2 header GRUB can boot
make tests      # host-side tests for boot parser and memory modules
```

For anything that changes boot behavior or on-screen output, also boot it and read the
screen:

```bash
make vm agent
sleep 10        # QEMU + GRUB run before the kernel prints anything
make vm screen
make vm stop
```

Paste the real output in the PR description. "Should work" is not evidence; the screen
dump or the failing test is.

## Code conventions

- `no_std`, `no_main`, edition 2024, `panic = "abort"`. There is **no `alloc`** — new
  code must be fixed-capacity or static, like `MAX_MEMORY_REGIONS = 32` and the
  allocator's bitmap.
- The only external dependency is `spin`. Adding a crate needs a reason beyond
  convenience (the `multiboot2` crate was dropped precisely because it pulled in
  `alloc`).
- Layout: core subsystems under `src/subs/`, hardware drivers under `src/drivers/`.
  Follow the tree in [docs/architecture.md](docs/architecture.md) when adding files.
- `cargo test` does nothing here (`test = false` on the binary). Logic modules get
  covered by the host crate in `tests/`, which reaches kernel code through thin
  `include!`/`#[path]` shims — add a shim and a test when you add a pure-logic module.
  Hardware-coupled code (VGA, boot asm) can't run there; prove it with the QEMU flow
  instead.
- Never commit build artifacts: `/target`, `/*.iso`, `iso/boot/rkrnl`,
  `iso/boot/kernel.bin`, `/tests/target`, `/.run/` are gitignored. Stage explicit paths
  (`git add <paths>`), not `git add -A`.

## Commit messages

Conventional-style one-line subjects, lowercase, no trailing period, **no scopes**:

```
feat: multiboot2 parser
fix: vga somehow corrupting memory
ref: split frame allocator into operators and reusables
docs: split README into docs/ and refresh boot notes
chore: fix cargo
```

Types in use: `init`, `feat`, `fix`, `ref` (not `refactor`), `docs`, `chore`. Body only
when the "why" isn't obvious from the subject — use `-` bullets. If the subject needs
"and", it probably wants two commits. (Older merged PRs carry `type(scope):` subjects;
new commits follow the no-scope style above.)

Check `git status`, `git diff`, and `git log --oneline -10` before committing.

## Pull requests

- Target `master`, one concern per PR — that's how the existing ones read.
- Describe what changed and how you verified it, with the actual command output.
- Update the docs that the change invalidates: [README.md](README.md),
  [docs/architecture.md](docs/architecture.md), [docs/boot.md](docs/boot.md),
  [docs/build.md](docs/build.md). Docs follow code; never document intended behavior as
  if it were implemented.
- Commenting `/oc` or `/opencode` on a PR runs opencode in CI to help with review.

## Repo notes for agent-assisted work

[AGENTS.md](AGENTS.md) and the skills under `.opencode/skills/` document the exact
build/test commands and the invariants of each subsystem for coding agents. If you work
with one, load the relevant skill (`kernel-build`, `kernel-memory`, `kernel-multiboot2`,
`kernel-vga-output`, `commit`) instead of rediscovering the constraints.
