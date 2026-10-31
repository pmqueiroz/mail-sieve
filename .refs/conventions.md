# Coding conventions & authoring rules

How code is written in this project. These are hard rules, not suggestions. This file is about **how to
author** code and where documentation lives.

## No comments in code

Code carries **no comments**. Do not:

- Explain a function with a comment above it.
- Annotate a line because it "might need documenting".
- Leave TODO/NOTE/why-notes inline.

If something needs explaining — a business rule, a non-obvious decision, a
gotcha — it belongs in `.refs/`, **always**. The code says *what*; `.refs/`
says *why*. A comment is a signal that a `.refs/` entry is missing: write the
entry instead.

Name things well enough that the code reads without prose. Extract a
well-named function or variable instead of writing a comment.

## Copy rules

User-facing strings never use an em dash (`—`). Split the sentence, or use
whichever of period, comma, or colon actually fits. A middle dot (`·`)
separates metadata in compact rows. Destructive actions say exactly what
happens and where ("removes 1,234 messages from the server").

This is a copy rule, not a prose rule: `.refs/` docs and commit messages
are unaffected by it.

## Rust

- Store functions take `&Connection` and are synchronous; async callers go
  through `AppState::read` / `AppState::write`, which run them on the
  blocking pool. Never hold a SQLite connection across an `.await`.
- Writes are batched: one transaction per fetched chunk or action chunk,
  never one per message.
- `imap` and `store` stay independent; only `server` knows both.
- `cargo fmt` and `cargo clippy -D warnings` are CI gates.

## Svelte

- Runes only (`$state`, `$derived`, `$effect`, `$props`). Shared reactive
  state lives in `*.svelte.ts` classes.
- An `$effect` that calls a loader must wrap the call in `untrack`.
  Loaders read their own state (`loading`, cursors, current rows), and
  without `untrack` the effect subscribes to it and reloads forever, which
  also wipes the selection on every load.
- Any server JSON used in the UI is typed in `src/lib/types.ts`.

## Documentation lives in `.refs/`

Every explanation of code behavior or business rule goes in the matching
`.refs/` file (see [AGENTS.md](../AGENTS.md) for the map). When you add or
change a feature, updating the `.refs/` doc is **part of the task**, not a
follow-up — the Operating Protocol in AGENTS.md is binding.

## Components must be reusable, and reused

The UI (or equivalent shared surface — a component library, a shared API
layer, a common CLI framework) is built from **shared, reusable pieces**. One
concept = one component, used everywhere that concept appears. Do not
reimplement the same behavior or interaction in two places.

Before building something new, check whether an existing piece already does
it (or nearly does it). If a near-match exists, **generalize and reuse it**
rather than copy-pasting a variant. If you build something likely to appear
again, extract it to the shared location from the start.

Consistency is the goal: the same interaction should look and behave the same
everywhere in the project.
