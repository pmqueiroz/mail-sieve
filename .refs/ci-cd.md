# CI/CD

GitHub Actions, `.github/workflows/ci.yml`. Runs on pushes to `main`, every
PR into `main`, and manual dispatch. A newer run on the same ref cancels the
older one.

The Rust toolchain is `stable` (matches `rust-toolchain.toml`) via
`dtolnay/rust-toolchain`; the workspace declares `rust-version = "1.88"`
because it uses let chains and `as_chunks`. Node is 24.

## Jobs

- **`web`**: `npm ci`, `npm run check` (svelte-check, warnings fail the
  build), `npm run build`. Uploads `web/dist` as an artifact so the Rust job
  embeds the real UI.
- **`site`**: the marketing and docs website in `site/`. `npm ci`,
  `npm run check` (`astro check`), `npm run build`. Independent of the
  other jobs. See [website.md](./website.md).

## Releases

`.github/workflows/release.yml` runs release-please on every push to
`main`. It keeps one open release PR that bumps the version and writes
`CHANGELOG.md` from the Conventional Commits since the last tag. Merging
that PR makes release-please create the `vX.Y.Z` tag and the GitHub
release, and the same workflow then calls `image.yml` and `binaries.yml`
with the tag.

- **Why calls instead of a tag trigger**: a tag created with the default
  `GITHUB_TOKEN` does not trigger other workflows, so a `push: tags`
  listener would never fire. The release workflow passes the tag to the
  reusable workflows instead. There is deliberately no tag trigger left:
  releases go through the release PR only.
- **Versioning**: one version for the whole project, tags without a
  component prefix (`v0.2.0`). Pre-1.0, a breaking change bumps the minor
  and `feat` bumps the patch (`bump-minor-pre-major`,
  `bump-patch-for-minor-pre-major`). `.release-please-manifest.json` holds
  the current version.
- **Files bumped** (`release-please-config.json`, release type `simple`
  plus `extra-files`): `workspace.package.version` in `Cargo.toml`, the five
  workspace crates in `Cargo.lock`, and `version` in `web/package.json` and
  `web/package-lock.json` (the login footer shows it). The `rust` release
  type was rejected: its updater refuses a virtual workspace manifest.
- **Cargo.lock gotcha**: release-please's TOML parser wraps every value as
  `{start, end, value}`, so the JSONPath filter must compare
  `@.name.value`, not `@.name`, and the safe evaluator allows `==` and `||`
  but no method calls. If a crate is added to the workspace, add it to that
  filter. CI builds with `--locked`, so a missed lockfile entry fails the
  release PR instead of shipping.
- **Token**: the action uses the `RELEASE_PLEASE_TOKEN` secret when it
  exists, otherwise `GITHUB_TOKEN`. With `GITHUB_TOKEN`, the repo setting
  "Allow GitHub Actions to create and approve pull requests" must be on, and
  CI does not run on the release PR (events from `GITHUB_TOKEN` never
  trigger workflows). A fine-grained token with contents and pull requests
  write access fixes both.

## Image publishing

`.github/workflows/image.yml` publishes `ghcr.io/pmqueiroz/mail-sieve`.

- **Triggers**: every push to `main` publishes `edge` and `sha-<short>`.
  When called (or dispatched) with a release tag it publishes `X.Y.Z`,
  `X.Y`, `X` (skipped while the major is 0, since 0.x minors may break)
  and `latest`. `latest` therefore always means the newest release, never
  `main`. The tag list lives once in the workflow-level `META_TAGS` so the
  per-platform images and the final manifest carry the same version
  labels.
- **Multi-arch on native runners**: `linux/amd64` builds on `ubuntu-24.04`
  and `linux/arm64` on `ubuntu-24.04-arm` (free for public repos). QEMU
  emulation was rejected: the release profile is fat LTO with one codegen
  unit, which takes tens of minutes emulated. Each runner pushes its image
  by digest only; the `merge` job stitches the digests into one tagged
  manifest list with `docker buildx imagetools create`. The Dockerfile
  needs no cross-compilation because every base image is multi-arch.
- **Cache**: GitHub Actions cache, one scope per platform so the two
  builds do not evict each other.
- **Provenance**: `actions/attest-build-provenance` signs a SLSA build
  provenance attestation for the manifest digest and pushes it to the
  registry. Verify with `gh attestation verify
  oci://ghcr.io/pmqueiroz/mail-sieve:latest --owner pmqueiroz`.
- **Labels**: `docker/metadata-action` sets the OCI labels (source,
  revision, license from the repo), which also links the package to the
  repository on GitHub.
- **Republish** a release image with a manual dispatch and the tag as
  input.
- The CI `docker` job still builds the image (amd64, not pushed) on every
  PR, so a broken Dockerfile is caught before it reaches `main`.

## Standalone binaries

`.github/workflows/binaries.yml`, called by the release workflow or
dispatched by hand with an existing tag, attaches binaries to the GitHub
release.

- **Targets**: `x86_64` and `aarch64` Linux (musl, fully static, same as
  the image), `x86_64` and `aarch64` macOS, `x86_64` Windows. Each builds
  on a native runner; Intel macOS cross-compiles from the Apple silicon
  runner because Apple's toolchain targets both. Nothing needs a C
  toolchain beyond `cc` (TLS is rustls with ring, SQLite is bundled), so no
  cross images are involved. Linux installs `musl-tools` for the C parts.
- **Web UI**: built once in the `web` job and downloaded into `web/dist`
  before each `cargo build`, so every binary embeds the real UI instead of
  the `build.rs` placeholder.
- **Archives**: `mail-sieve-<tag>-<target>.tar.gz` (`.zip` on Windows)
  with the binary, `LICENSE` and `README.md`, plus a `SHA256SUMS` file.
  Every archive gets a build provenance attestation (`gh attestation verify
  <archive> --owner pmqueiroz`).
- **Smoke test**: each binary runs `--help` on its runner, except Intel
  macOS, which cannot run on the Apple silicon runner.
- The binaries are not code signed. macOS quarantines downloaded binaries
  and Windows SmartScreen warns; the install docs explain both.

## Website deploy

`.github/workflows/pages.yml` builds `site/` and deploys it to GitHub Pages
on every push to `main` that touches `site/**` (or the workflow itself),
plus manual dispatch. It is separate from `ci.yml` because it needs the
`pages: write` and `id-token: write` permissions and the `github-pages`
environment; deploys never cancel each other mid-flight. The repo's Pages
source is "GitHub Actions" and the custom domain
`mailsieve.pmqueiroz.dev` is set in the repo's Pages settings (a custom
Actions deploy ignores `CNAME` files; `site/public/CNAME` is kept as
documentation and for hosts that read it). DNS: a `CNAME` record from
`mailsieve` to `pmqueiroz.github.io`.
- **`rust`**: needs `web`. Starts Dovecot (`dovecot/dovecot:2.3-latest`
  with `docker/dovecot/dovecot.conf`) on `127.0.0.1:3143`, then
  `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`. With
  `MAIL_SIEVE_TEST_IMAP` set, the IMAP integration suite
  (`crates/imap/tests/dovecot.rs`) and the server end-to-end flow
  (`crates/server/src/e2e_tests.rs`) run against it. Without the variable
  those tests print a skip message and pass, so local `cargo test` works
  offline.
- **`docker`**: needs `rust`. Builds the multi-stage `Dockerfile` with
  Buildx and the GitHub Actions cache. Nothing is pushed.

## Local equivalents

```sh
docker compose -f docker/compose.test.yml up -d
MAIL_SIEVE_TEST_IMAP=127.0.0.1:3143 cargo test --workspace
(cd web && npm ci && npm run check && npm run build)
(cd site && npm ci && npm run check && npm run build)
docker build -t mail-sieve .
```

## Not set up yet

- Code signing and notarization for the macOS and Windows binaries.
- Windows on ARM and 32-bit targets.
- A Docker Hub mirror. GHCR only for now.
- Real-provider tests (Gmail, Outlook) need secrets and accounts and are
  not automated.
