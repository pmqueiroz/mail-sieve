# CI/CD

> Template. Replace the specifics (CI provider, workflow file, toolchain,
> version) once the pipeline exists. Keep the shape: what triggers a run,
> what gates what, and what's explicitly not set up yet.

[CI provider], `[path to workflow file]`. Runs on [triggers — push to
main, every PR into main, manual dispatch, …]. [Concurrency/cancellation
behavior, if any.]

Toolchain version is pinned to `[version]` (matches `[version pin file]`) via
`[action/tool used to install it]`. Bump both together when the local
toolchain moves.

## Jobs

- **`[job name]`** — [what it gates, what it runs: install deps, format
  check, lint, test]. This is the gate everything else needs.
- **`[job name]`** — [build/package job]. Depends on `[checks job]`.
  [What artifact it produces, where it's uploaded, retention.] [Any
  deliberate scope limit — e.g. debug-only build because there's no signing
  yet — and why a "real" version would be misleading right now.]

## Not set up yet

[Be explicit about what's deliberately deferred: release signing, store
upload, secrets management, deployment automation, etc. State why it's a
deliberate scope decision rather than an oversight, and what has to be true
first (e.g. a `legal.md`/compliance sign-off) before setting it up.]
