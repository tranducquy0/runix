---
name: commit
description: Write git commit messages for runix in its established Conventional Commits style. Use when committing, writing or reviewing a commit message, or deciding how to split changes into commits.
---

## Format

```
type: subject

[optional body]

[optional trailers]
```

- One-line subject: `type: subject`, lowercase, no trailing period.
- No scopes — this repo does not use `type(scope):`.
- Body only when the "why" or a list of changes is not obvious from the subject.
  Use `-` bullets; wrap continuation lines.
- Trailer for agent-authored commits:
  `Co-authored-by: opencode-agent[bot] <opencode-agent[bot]@no-reply.users.github.com>`

## Types (as used in history)

- `init` — first-ever commit / project bootstrap (`init: first tiny step...`).
- `feat` — new capability (`feat: multiboot2 parser`, `feat: memory allocator`).
- `fix` — bug fix (`fix: vga somehow corrupting memory`, `fix: linker script`).
- `ref` — refactor / rename / restructure (`ref: rename things`,
  `ref: move subsystems under src/subs ...`). Use `ref`, not `refactor`.
- `docs` — documentation, AGENTS.md, skills, comments (`docs: adding comments`,
  `docs: add agent skills and AGENTS.md`).
- `chore` — build/tooling/misc housekeeping (`chore: fix cargo`, `chore: stuff`).

## Rules

- Subject describes the whole commit; if you need "and", it may deserve two commits.
- Prefer splitting unrelated changes (e.g. a doc restructure vs. adding a skill)
  into separate commits rather than one mixed commit.
- Never commit build artifacts: `/target`, `runix.iso`, `iso/boot/rkrnl`,
  `iso/boot/kernel.bin` are gitignored — leave them alone.
- Stage explicitly (`git add <paths>`); do not blanket `git add -A`.
- Check `git status`, `git diff`, and `git log --oneline -10` before committing.

## Subject examples from history

```
feat: real vga implementation
fix: GRUB can't see the kernel...
ref: memory.rs contains allocator, so it should be "allocator.rs"
docs: split README into docs/ and refresh boot notes
chore: stuff (check desc)
```
