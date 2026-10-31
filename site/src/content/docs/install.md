---
title: Installation
description: Run mail-sieve with Docker or Docker Compose, keep the secret key safe, and update.
lead: One container, one volume. Most people are running in a couple of minutes.
group: Getting started
order: 2
---

## Docker

```sh
git clone https://github.com/pmqueiroz/mail-sieve
cd mail-sieve
docker build -t mail-sieve .
docker run -d --name mail-sieve \
  -p 127.0.0.1:8080:8080 \
  -v mail-sieve-data:/data \
  mail-sieve
```

Open `http://localhost:8080`, choose an admin password, then add your first account.

## Docker Compose

```sh
docker compose -f docker/compose.yml up -d
```

Set `MAIL_SIEVE_PORT` to use a host port other than 8080, and `MAIL_SIEVE_HOST_IP=0.0.0.0` to reach it from other devices. Both can live in `docker/.env`.

:::note
The data volume holds `secret.key`, which encrypts your stored credentials. Back it up. Without it, saved accounts must be re-added.
:::

## Updating

```sh
git pull
docker compose -f docker/compose.yml up -d --build
```

Database migrations run automatically on start. Back up the data volume before upgrading.
