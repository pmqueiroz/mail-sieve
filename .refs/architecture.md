# Architecture

> Template. Replace every bracketed section with the real thing as the
> project takes shape. Keep the *shape* — this file is where an agent looks
> first to understand how the codebase fits together — but the content is
> 100% project-specific. Delete this callout once filled in.

[One or two sentences: what the project is, the core abstraction it's built
around, and the one-line mental model — e.g. "a plugin host that owns X, a
plugin owns only its own Y".]

Code is the truth. This file explains the *why* behind the shape it takes.

## Layout

- [Entry point(s) — where execution starts.]
- [The shell/host layer — what it owns, what it exposes to the rest of the
  codebase.]
- [The core/platform layer — the reusable machinery every feature is built
  on.]
- [The single import point, if there is one, that feature modules use instead
  of reaching into core internals directly — this is what lets the core move
  things without touching every feature.]
- [Feature/module layout — one directory per feature, plus the single wiring
  point where they're registered.]
- [Any generated code and what generates it.]

## The extension point

[If this project has a plugin/module/extension pattern, describe it here:
what the minimal contract is, what's static vs. runtime, what owns the
lifecycle, and how a new module gets discovered without touching unrelated
code. State what's deliberately coarse-grained vs. what's left to each
module's own finer-grained state, and why.]

Adding a [feature/module/plugin]: [the actual steps — ideally "create a
directory, implement the interface, register it in one place, nothing else
needs to change."]

## Shared state

[What app-lifetime services exist (registries, session/user state, caches,
config), where they're constructed, and how the rest of the codebase reaches
them — a single injected scope/context is common. Note any deliberate
exceptions to that pattern and why they're exceptions rather than the rule.]

## Reusable mechanics

[Any cross-cutting engine-level pieces that are generic and shared across
features — sequencing/ordering, state machines, timers, permission checks,
whatever "grammar" repeats across the domain. Name them and say what each
owns.]

## Design system / shared UI (or API surface)

[Where the one place raw primitives are allowed to be constructed is, and
what composes on top of it instead. If there's an automated check enforcing
the boundary, name it and describe how strict it is on purpose.]

### Tokens

[Color, spacing, typography, or equivalent design tokens — where they live
and the rule that components never hardcode raw values.]

### Why these components exist

[The actual constraint that shaped the design system — e.g. device
constraints, accessibility requirements, brand consistency — and which
components exist specifically because of it.]

## Localization / configuration

[If applicable: where translatable strings or environment-specific config
live, and how the generated/runtime layer is produced from source files.]

## [Feature deep-dive: name of the first real feature]

[Once there's a first real feature built on the extension point, give it its
own section here (or split it into its own `.refs/<feature>.md` and link it)
— state machine, edge cases, the non-obvious rules that aren't visible from
reading the code alone.]

## Known gaps

[Be honest about what's stubbed, unfinished, or deliberately deferred, and
why — this is what stops an agent from "fixing" something that was left
incomplete on purpose, or from being surprised that a screen/endpoint does
nothing yet.]
