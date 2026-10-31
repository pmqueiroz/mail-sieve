const repo = 'https://github.com/pmqueiroz/mail-sieve';

export const site = {
  name: 'mail-sieve',
  title: 'mail-sieve: bulk-clean any IMAP mailbox, self-hosted',
  description:
    'mail-sieve groups every message by sender, then lets you archive, trash or unsubscribe from thousands at once. Self-hosted, works with any IMAP server.',
  locale: 'en',
};

export const links = {
  repo,
  issues: `${repo}/issues`,
  releases: `${repo}/releases`,
  license: `${repo}/blob/main/LICENSE`,
  docsSource: `${repo}/edit/main/site/src/content/docs`,
  sponsors: 'https://github.com/sponsors/pmqueiroz',
};

export const installCommand =
  'docker run -d -p 127.0.0.1:8080:8080 -v mail-sieve-data:/data ghcr.io/pmqueiroz/mail-sieve';

export const composeFile = `services:
  mail-sieve:
    image: ghcr.io/pmqueiroz/mail-sieve:latest
    restart: unless-stopped
    ports:
      - "127.0.0.1:8080:8080"
    volumes:
      - mail-sieve-data:/data

volumes:
  mail-sieve-data:`;
