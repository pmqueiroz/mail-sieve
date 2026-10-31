import { defineConfig } from 'astro/config';
import { satteri } from '@astrojs/markdown-satteri';
import sitemap from '@astrojs/sitemap';
import tailwindcss from '@tailwindcss/vite';
import { callouts } from './src/plugins/callouts.ts';

const site = process.env.SITE_URL ?? 'https://pmqueiroz.github.io';
const base = process.env.SITE_BASE ?? '/mail-sieve';

export default defineConfig({
  site,
  base,
  trailingSlash: 'always',
  build: { format: 'directory', inlineStylesheets: 'always' },
  redirects: { '/docs': `${base}/docs/intro/` },
  integrations: [sitemap()],
  markdown: {
    syntaxHighlight: false,
    processor: satteri({
      features: { directive: true },
      mdastPlugins: [callouts],
    }),
  },
  vite: { plugins: [tailwindcss()] },
});
