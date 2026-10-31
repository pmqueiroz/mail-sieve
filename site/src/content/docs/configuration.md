---
title: Configuration
description: Every mail-sieve environment variable, its default and what it does.
lead: Every option is an environment variable, or the matching command-line flag.
group: Reference
order: 1
---

| Variable | Purpose |
| --- | --- |
| `MAIL_SIEVE_DATA_DIR` | Database and `secret.key`. Default `./data`, or `/data` in Docker |
| `MAIL_SIEVE_BIND` | Listen address. Default `127.0.0.1:8080`, or `0.0.0.0:8080` in Docker |
| `MAIL_SIEVE_PUBLIC_URL` | External URL, e.g. `https://mail.example.com`. Enables secure cookies and builds OAuth redirect URIs |
| `MAIL_SIEVE_SECRET_KEY` | Base64 32-byte key, used instead of `secret.key` |
| `MAIL_SIEVE_ADMIN_PASSWORD` | Sets or resets the admin password at startup |
| `MAIL_SIEVE_TRUSTED_HEADER` | Trust a reverse-proxy auth header such as `Remote-User`. Only behind a proxy that strips it from clients |
| `MAIL_SIEVE_GOOGLE_CLIENT_ID` / `_SECRET` | Enables Google sign-in |
| `MAIL_SIEVE_MICROSOFT_CLIENT_ID` / `_SECRET` | Enables Microsoft sign-in |
| `RUST_LOG` | Log filter. Default `info` |

:::note
If you expose mail-sieve beyond localhost, put it behind a TLS reverse proxy (Caddy, Traefik, nginx). Live progress uses server-sent events, so disable response buffering for `/api/events`.
:::

## Command line

```sh
mail-sieve serve                 # default
mail-sieve accounts              # list accounts
mail-sieve sync --account 1      # one-shot sync with timing
mail-sieve set-password          # reset the admin password from stdin
mail-sieve health                # exit 0 when the server answers
```
