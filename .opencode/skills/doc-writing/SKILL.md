---
name: doc-writing
description: Write technical documentation (README, architecture notes, developer guides, API references, changelogs, code comments) in the repository owner's voice, verified against real code. Use when creating, rewriting, or reviewing any documentation file, project introduction, or doc comment set.
---

## Goal

Write docs like the owner explaining real software to someone who wants to understand
it: direct, technically precise, concrete about paths/commands/behavior, informal only
when the document type allows it. Style never outranks truth.

Evidence-based voice profile with short examples from the owner's own messages:
`subskills/voice-profile/SKILL.md` — load it before drafting anything longer than a
changelog entry.

## Workflow

1. **Inspect reality first.** Read the source, config, tests, Makefile/scripts, existing docs, and `AGENTS.md`. Docs follow code, not filenames.
2. **Set the register.** Pick the document type from the table below before drafting a word.
3. **Extract facts.** Commands, paths, config keys, error strings, behavior, limitations — each must come from evidence you actually read.
4. **Outline** around what the reader must understand or accomplish.
5. **Draft** using the voice rules below.
6. **Verify claims.** Cross-check every command, path, filename, signature, and behavior against the code.
7. **Remove noise.** Filler, generic openings, restated intros, duplicated sections, empty adjectives.
8. **Review** the result against repo conventions and the document's intended question.

Never document intended behavior as implemented. Never invent supported features, test
results, benchmarks, install steps, or compatibility claims.

## Voice rules (condensed)

- Start with the subject, problem, or observation. Generic AI openings ("In today's rapidly evolving...") are banned.
- One idea per sentence; never repeat the same point across sections.
- Concrete beats vague: real paths, commands, code fences, error messages, config snippets.
- Explain why with cause → effect: constraint → decision → consequence. Keep trade-offs and real limits visible — "early and minimal, idles after the smoke test" beats pretending completeness.
- Short declaratives. Informal docs may use the owner's casual openers ("okay, now...", "wait, that breaks...") and parenthetical clarifications ("(not runit, pronounced like R-Unix)").
- Shorthand only when it adds density (`rn`, numbered answers); never when it turns cryptic.
- Humor only in informal documents and only when it carries information (relatable failure, absurd comparison). Never in API references, security instructions, or serious troubleshooting.
- Never imitate typos or add decorative slang/emoji. The voice profile marks which conversational quirks are observed and which are unverified — follow it, not the chat log.
- Treat pasted briefs/specs as the formal register the owner commissions (many are ChatGPT-drafted), not as their conversational voice — the document type, not the brief's polish, decides the register.

## Register by document type

| Document | Approach |
|---|---|
| README | What the project is, honest status, quick start with exact commands, doc links. See `README.md` for the target register. |
| Architecture doc | Components, responsibilities, relationships, data/control flow, trade-offs; layout trees and concrete file paths (see `docs/architecture.md`). |
| Developer guide | Prerequisites → exact commands → expected result; note real gotchas (see `.opencode/skills/kernel-build/SKILL.md`). |
| API reference | Exact contracts: inputs, outputs, errors, auth, examples. Formal register, zero jokes. |
| Design proposal | Problem → constraints → alternatives → decision → consequences. |
| Troubleshooting | Symptom → cause → verification → fix; quote the real error string. |
| Changelog / commit | Concrete change, behavior impact, compatibility notes. Commit messages follow the `commit` skill. |
| Code comments | Non-obvious intent, invariants, constraints only. Never narrate what the code plainly says. |
| Informal project notes | The owner's conversational voice fully allowed: shorthand, asides, first-person. |

## Do not

- Corporate marketing language, inflated claims, meaningless adjectives.
- Repeat the README in every other doc; tables added just to look organized; a file per tiny concept.
- Line-by-line explanation of obvious code; long dependency/tooling lists without a concrete need.
- Architecture invented from filenames; installation steps copied from another project.
- Style imitation that copies catchphrases but misses the reasoning patterns behind them.

## Calibration

A README, a developer guide, and an architecture doc must read like the same
technically minded author: same precision, same honesty about limits, same concrete
reasoning — only the formality changes. Synthesized test samples for all three registers
are in `subskills/voice-profile/SKILL.md`.
