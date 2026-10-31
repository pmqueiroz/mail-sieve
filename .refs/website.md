# Website

The public landing page and user documentation live in `site/`, separate
from the app's `web/` SPA. It implements the Claude Design handoff
`Website.dc.html` (same design project as `Mail Sieve.dc.html`).

## Why Astro, why static

Requirements were: fully static, SEO-first, as fast as possible, docs in
Markdown. Astro renders every page to HTML at build time and ships zero JS
by default, so the output is plain files that any static host can serve.

- **Real routes, not the design's hash router.** The mock switches pages
  with `#/docs/...`; crawlers do not index hash routes, so every doc is its
  own URL (`/docs/<id>/`). `/docs/` is a meta-refresh redirect to
  `/docs/intro/` (marked `noindex`, canonical to the target).
- **Only JS on the page** is the copy-to-clipboard handler in
  `CopyButton.astro` (a few hundred bytes, inlined by Astro). Nav
  scrolling, responsive layout and the sticky header are CSS.
- **CSS is inlined** (`build.inlineStylesheets: 'always'`), so there is no
  render-blocking stylesheet request. The home page is about 12 KB gzipped.
- **Fonts** are self-hosted variable woff2 files, Latin subset only (one file
  per family covers every weight). Plus Jakarta Sans is preloaded; JetBrains
  Mono is not, it only styles small labels and code.
- **The hero demo is a static screenshot**, not the design's live iframe.
  The mock embeds the whole interactive prototype (React + Babel from unpkg,
  hundreds of KB) which would wreck load time. `src/assets/demo.png` is a
  2560x1600 capture of `Mail Sieve.dc.html` (Overview, dark) and is served as
  responsive AVIF with a WebP fallback via `astro:assets`. The caption says
  "sample data" instead of the mock's "Click around". Re-capture it if the UI
  changes noticeably.
- Lighthouse (mobile, simulated throttling) at the time of writing: 100 on
  performance, accessibility, best practices and SEO for home and docs.
  Keep it there: no client frameworks, no third-party scripts or fonts.

## SEO

`layouts/Base.astro` owns the head: title, description, canonical, Open
Graph and Twitter card, theme color, favicon, sitemap link and JSON-LD.
Home emits `SoftwareApplication` + `WebSite`; each doc emits `TechArticle` +
`BreadcrumbList`. `@astrojs/sitemap` writes `sitemap-index.xml`;
`pages/robots.txt.ts` points to it. `public/og.png` is a 1200x630 card
rendered from the hero headline (the repo banner is 1280x320 and crops
badly in link previews). Each doc's frontmatter `description` is its meta
description, `lead` is the visible subtitle.

## Base path and site URL

`astro.config.mjs` reads `SITE_URL` (default `https://pmqueiroz.github.io`)
and `SITE_BASE` (default `/mail-sieve`), which fits a GitHub Pages project
site. For a custom domain build with `SITE_URL=https://example.com
SITE_BASE=/`. Every internal link goes through `lib/paths.ts`
(`href`, `docHref`) so the base is never hard-coded. Under a base path
`robots.txt` is not at the domain root and crawlers ignore it; the sitemap
link in the head still works.

## Docs authoring

One Markdown file per page in `src/content/docs/`. The file name is the URL
slug. Frontmatter (validated in `src/content.config.ts`):

```yaml
title: Search syntax
description: Meta description, one sentence.
lead: The subtitle shown under the title.
group: Using mail-sieve   # Getting started | Using mail-sieve | Reference | Community
order: 2                  # position inside the group
```

Sidebar groups, ordering and Previous/Next all derive from `group` +
`order` (`lib/docs.ts`); there is no separate nav file. "Edit this page on
GitHub" links to the file on `main`.

Supported syntax: headings, paragraphs, lists, fenced code, inline code,
links and GFM tables. Two-column tables render as the design's key/value
rows (the header row is visually hidden but kept for screen readers).
Custom components use container directives:

```md
:::note
Shown as the purple note box.
:::

:::warning
Shown as the amber warning box.
:::
```

Astro 7's default Markdown engine is Sätteri, not remark/unified, so remark
plugins do not apply. Directives are enabled with
`features: { directive: true }` and turned into `<aside class="callout
callout-*">` by the Sätteri mdast plugin in `src/plugins/callouts.ts`. To
add a callout kind, add its name there and a `.callout-<name>` rule in
`global.css`. Syntax highlighting is off on purpose: the design shows
monochrome code and Shiki would add inline styles to every token.

Markdown typography lives in the `.doc` component layer of
`src/styles/global.css`, the one place the site uses selectors instead of
utilities, because rendered Markdown cannot carry classes.

## Styling

Same rules as the app (see [conventions.md](./conventions.md) and
[design-system.md](./design-system.md)): Tailwind v4 utilities only, 2px
spacing unit, every color, size, radius and width a named `@theme` token in
`site/src/styles/global.css`, no raw px or hex in class names. The site is
dark only, as drawn. Breakpoint `wide` is 860px, the mock's `w >= 860`
switch (desktop nav and docs sidebar versus the mobile chip row).

Deviations from the mock:

- `ink-3` is `#82828b`, not the mock's `#73737c`: the original fails WCAG AA
  (4.15:1 on the page, 3.8:1 on cards) for the small labels it is used on.
- Header gap is 16px below `wide` (28px above): at 390px the mock's header
  overflows the viewport.

## Copy: what is real and what is placeholder

The designer's copy was placeholder. Claims were checked against the code
and README and rewritten where they were wrong:

- Install: no image is published (see [ci-cd.md](./ci-cd.md)), so the
  hero command and install docs use `git clone` + `docker compose`, not the
  mock's `ghcr.io/...` image. Swap `installCommand` in `lib/site.ts` once
  an image exists.
- Env vars are the real `MAIL_SIEVE_*` set, not the mock's `MS_*`.
- Search syntax is the real DSL (`has:list`, not `list:*`).
- Removed features that do not exist: rules as YAML export/import, pinned
  senders, arm64 images, "undo for 30 days" (undo exists, from the toast or
  Activity, once).

Support is GitHub Sponsors only, on the owner's personal account
(`github.com/sponsors/pmqueiroz`); Patreon was dropped. Still placeholder:
the "Free forever / no investors / donations pay for..." support copy and
the illustrative numbers in the home page panels (sender counts, "1,246
messages", "Last run 09:15").

## Commands

```sh
cd site
npm install
npm run dev       # http://localhost:4321/mail-sieve/
npm run check     # astro check
npm run build     # static output in site/dist
npm run preview
```
