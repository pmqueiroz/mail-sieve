---
title: Contributing
description: Set up a development environment, run the tests and send a pull request.
lead: Bug reports, provider testing and pull requests are all welcome.
group: Community
order: 1
---

Requirements: Rust 1.88+, Node 24 and Docker for the IMAP test server.

```sh
docker compose -f docker/compose.test.yml up -d
cargo run -p mail-sieve -- serve
(cd web && npm install && npm run dev)
MAIL_SIEVE_TEST_IMAP=127.0.0.1:3143 cargo test --workspace
```

- Open an issue before large changes so we can agree on the approach
- `cargo fmt` and `cargo clippy -D warnings` must pass
- Project notes for contributors live in `AGENTS.md` and `.refs/`

## This website

The site lives in `site/` and every docs page is a Markdown file in `site/src/content/docs/`. Use the "Edit this page on GitHub" link at the bottom of any page to change it.
