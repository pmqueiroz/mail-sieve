---
title: Installation
description: Run mail-sieve with Docker or Docker Compose, keep the secret key safe, and update.
lead: One container, one volume. Most people are running in a couple of minutes.
group: Getting started
order: 2
---

## Docker

```sh
docker run -d --name mail-sieve \
  -p 127.0.0.1:8080:8080 \
  -v mail-sieve-data:/data \
  ghcr.io/pmqueiroz/mail-sieve
```

Open `http://localhost:8080`, choose an admin password, then add your first account.

Images are published for `linux/amd64` and `linux/arm64`, so the same command works on a Raspberry Pi or an ARM NAS.

| Tag | What it is |
| --- | --- |
| `latest` | The newest release |
| `0.1.0`, `0.1` | A specific release, or the newest patch of a minor version |
| `edge` | The current `main` branch. Newest features, least tested |

## Docker Compose

Save this as `compose.yml`:

```yaml
services:
  mail-sieve:
    image: ghcr.io/pmqueiroz/mail-sieve:latest
    restart: unless-stopped
    ports:
      - "127.0.0.1:8080:8080"
    volumes:
      - mail-sieve-data:/data

volumes:
  mail-sieve-data:
```

Then start it:

```sh
docker compose up -d
```

The repository also ships this file as `docker/compose.yml`, where `MAIL_SIEVE_PORT` picks a host port other than 8080 and `MAIL_SIEVE_HOST_IP=0.0.0.0` makes it reachable from other devices. Both can live in `docker/.env`.

:::note
The data volume holds `secret.key`, which encrypts your stored credentials. Back it up. Without it, saved accounts must be re-added.
:::

## Updating

```sh
docker compose pull
docker compose up -d
```

Database migrations run automatically on start. Back up the data volume before upgrading.
