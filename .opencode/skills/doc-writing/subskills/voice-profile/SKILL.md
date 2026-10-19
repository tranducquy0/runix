---
name: voice-profile
description: Evidence-based profile of the repository owner's writing voice — observed, contextual, and inferred patterns with short examples from the owner's own messages. Load when drafting documentation in the owner's voice or calibrating tone for a document type.
---

# Owner voice profile

## Evidence actually reviewed

- OpenCode CLI prompt history (19 entries) and OpenCode desktop prompt history (65 entries), spanning runix, gemini-cli/Antigravity, proot, simplemail, quickshell, tree-vcs, OpenJukebox, profilio, plus user turns in stored sessions.
- Codex CLI rollout sessions (user turns only) and the Gemini CLI prompt log.
- Git history in this repo: early hand-written commits (`chore: stuff`, `fix: vga somehow corrupting memory`, body `README.md [new]: a... "documentation" :)`) and the owner's original pre-agent README text (`runix` (not runit, pronounced like R-Unix) ... a hobby project that I built to learn).

Assistant replies and generated drafts were excluded as voice evidence. Pasted template
text (e.g. the AGENTS.md generator prompt) counts only as evidence of the register the
owner requests, not of their natural voice. The doc-writing task prompt itself is
ChatGPT-origin (owner's own note) — it was used as a checklist of hypotheses to validate,
never as voice evidence. Elaborate pasted plans/specs may also be ChatGPT-drafted: they
show the formal register the owner commissions, not how they type.
No ChatGPT conversation export exists on this machine — the profile comes from the
accessible session history above. Never claim ChatGPT history was reviewed.

## Observed — repeatedly supported by the owner's messages

**Terse imperatives.** "continue", "commit", "go for it", "okay, commit". Requests to
the agent are short; docs inherit that economy — no padding around the point.

**Casual openers and dropped apostrophes.** "wait, dont run tsc rn, it's slow...", "now it doesnt print anything", "also, the CPU's Silvermont, not braswell". Connectors
*okay / now / wait / also / just* open sentences, often lowercase mid-conversation.

**Shorthand for density, not style.** "input are placeholder rn", "ofc", "cuz", "yup",
and numbered replies ("1. yes / 2. boot, mem (rename from memory), handlers... wait,
does it belong to the subsystem group? / 3. input are placeholder rn").

**Plans become lists.** "plan: - first, ... - then, ... - make sure tests cover all the
scope this kernel CURRENTLY have"; stack decisions as bullets ("- value simplicity",
"- gruvbox", "- Pi agent via `pi --mode rpc`"). Multi-step anything gets structure.

**Parenthetical clarifications.** "(not runit, pronounced like R-Unix)", "(so you or any
other agents can test the kernel live)", "(rename from memory)", "(install `@caveman-ai/block` via `npm` and use `block init`)".

**Evidence-first reporting.** Pastes the real output and reasons from it: "I booted, it
shows \"usable: s\"", the pasted `linker ld.lld not found` failure, "tested, and
tool-calling on the model side works, but next prob come as usual:".

**Direct correction of wrong assumptions.** "wait, where's the file bro?", "also, the
CPU's Silvermont, not braswell".

**Cause → effect with consequences.** "dont perform build, that'll burn 4 hrs"; "I
realized that we can just access Antigravity through a Google Cloud Project, so we can
just add env detection. also, that ofc not a secret cuz it's Google's OAuth cred in
shipped bins".

**Humor riding on real frustration.** "bro,  calm down, you overthought :))) it booted
successfully"; old commit body: "README.md [new]: a... \"documentation\" :)", "src/drivers
[new]: drivers support! yayyy!".

**First person for ownership.** "a hobby project that I built to learn about operating
system...", "I added myself, doing a forced push is fine since there's only one commit".

**The owner controls register explicitly.** "state this as a PoC impl, should not use on
main Google Accounts (with playful, dev-friendly tone)"; "rewrite docs so it looks like
normal for me and other users". The same person commissions structured specs when a task
needs them (stack bullet lists typed straight into the agent, the tests/vm plan above).
Formality is a choice per document, never a loss of precision.

## Contextual — real but situational

- `bro` — two informal asides only. Fine in casual notes, never in formal docs.
- Emphatic CAPS — one instance ("this kernel CURRENTLY have"). Rare by nature.
- Smilies `:)` / `:)))` — a handful of times in chat and old commit bodies; optional in informal notes only.
- Humor and first-person essays land in project notes and commits, not in references.
- **ChatGPT voice ≠ agent voice (self-reported, unverifiable here).** The owner notes their
  ChatGPT-drafted prompts are slightly different from how they type to agents. When text is
  known or likely ChatGPT-drafted, treat it as the formal register they commission; the
  conversational baseline above comes from their own typed agent messages only.

## Inferred / unverified — use with caution

- 💀, `:P`, emoji decoration — zero occurrences in accessible user messages. Do not add.
- "Sarcastic" as a trait — only self-deprecating jokes found; humor is occasional, never snide.
- Lowercase openings everywhere — observed in chat prompts; documents the owner asked for
  in formal register used standard capitalization. Match the document, not the chat log.
- Typos in prompts ("approriate", "rediesign", "obfusticate") reflect fast typing, not a
  style to reproduce.

## Calibration samples (synthesized — not owner quotes)

Same author, three registers:

1. **Informal project note:** "boot works now. grub hands us the memory map, we filter out
   the kernel image and the boot stack (they're obviously still in use), the bitmap
   allocator smoke-tests alloc/free/reuse, then it idles. no scheduler, no paging —
   that's next."
2. **Developer guide:** "Run `make build`, then `make iso test` before `make iso build` —
   the validation script needs `readelf` and `grub-file` on PATH. If linking fails with
   `linker ld.lld not found` (resolved by using bundled `rust-lld`), export `CARGO_TARGET_I686_RUNIX_TARGET_LINKER` pointing at
   the toolchain's `rust-lld`."
3. **Architecture doc:** "The frame allocator owns physical frames over the
   reserved-filtered boot map. RAM is never assumed contiguous: regions come from the
   Multiboot2 memory map and are walked first-fit. `free()` validates alignment and region
   ownership, so double-free is rejected instead of silently corrupting the bitmap."

All three: concrete evidence, honest limits, cause-effect reasoning — only formality shifts.
