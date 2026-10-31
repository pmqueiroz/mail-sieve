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

## Image publishing

`.github/workflows/image.yml` publishes `ghcr.io/pmqueiroz/mail-sieve`.

- **Triggers**: every push to `main` publishes `edge` and `sha-<short>`;
  a `v*` tag publishes `X.Y.Z`, `X.Y`, `X` (skipped while the major is 0,
  since 0.x minors may break) and `latest`. `latest` therefore always means
  the newest release, never `main`. Manual dispatch rebuilds the current
  ref.
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
- **Release**: on a tag, after the image is published, a GitHub release is
  created with generated notes. Cut a release by pushing an annotated tag
  that matches the workspace version: `git tag -a v0.1.0 -m v0.1.0 && git
  push origin v0.1.0`.
- The CI `docker` job still builds the image (amd64, not pushed) on every
  PR, so a broken Dockerfile is caught before it reaches `main`.

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

- Automated versioning (release-please or similar). Tags and the
  workspace version are bumped by hand for now.
- Release binaries outside the image.
- A Docker Hub mirror. GHCR only for now.
- Real-provider tests (Gmail, Outlook) need secrets and accounts and are
  not automated.
