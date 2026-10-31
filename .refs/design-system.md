# Design system

The web UI implements the Claude Design handoff "Mail Sieve" (`Mail
Sieve.dc.html`). Tailwind v4 is the only styling layer; every value comes
from the `@theme` block in `web/src/app.css`.

## Tokens

- **Spacing unit is 2px** (`--spacing: 2px`). Every spacing, size and
  inset utility is therefore a multiple of 2: `p-7` is 14px, `w-42` is
  84px, `h-26` is 52px. Arbitrary values use `--spacing(n)` (for example
  grid tracks `grid-cols-[--spacing(9)_minmax(0,2.4fr)]`), never raw px.
  The design's few odd paddings (3px pill and segmented paddings, 7px tag
  padding) were rounded to the nearest even value.
- **Colors** are the design's semantic palette (`page surface surface2 ink
  ink2 ink3 line chip btn btn-ink good bad warn` plus `*-bg` tints, `ring`,
  `track`, `overlay`). Dark values override the same variables under
  `prefers-color-scheme: dark`, so components never branch on theme.
- **Type scale** is named by role, not size: `micro 10 · tiny 11 · xs 12 ·
  sm 13 · base 14 · md 16 · lg 18 · xl 20 · 2xl 26 · 3xl 28 · hero-sm 72 ·
  hero 104`. Default Tailwind scales (fonts, text, radius, shadows,
  breakpoints, colors) are reset with `--*: initial` so only design values
  exist.
- **Breakpoints** mirror the design's JS layout switches: `wide` 760px
  (rail and full table columns), `split` 1100px (message preview docks
  beside the list), `board` 1240px (Overview's two-column board).
- **Utilities**: `bg-sieve` (the coral/amber/mint gradient), `bg-sieve-bloom`,
  `bg-ticks` (the striped never-opened track) and `border-check` (the
  1.5px checkbox/radio stroke).
- **Fonts** are Plus Jakarta Sans and JetBrains Mono from `@fontsource`,
  Latin subset and only the used weights. They are bundled because the CSP
  is `default-src 'self'`; Google Fonts would be blocked.

## The content-box gotcha

The design renders with the browser default `box-sizing: content-box` on
divs and inputs (buttons are border-box by UA default). A row declared
`height: 52px` with a bottom border is 53px tall there; a 48px search bar
with a border is 50px. Tailwind's preflight makes everything border-box.
Instead of compensating with offsets, the elements that rely on it carry
`box-content`: the search bar, text inputs, table header and rows, job
rows, unsubscribe result rows, popovers and toasts. Virtualized rows
therefore pass `ROW_CONTENT_HEIGHT + HAIRLINE` to `VirtualList`. When
porting another design element with an explicit size and a border, check
which box model it was drawn in.

## Components

One concept, one component (`web/src/lib/components`):

- `Button` (variants `primary outline ghost danger danger-text muted-text
  inverse on-dark on-dark-outline`; sizes are the design's heights:
  `xs 28 · sm 30 · md 32 · lg 34 · xl 36 · 2xl 38 · dialog 40`).
- `Chip` (toggle pill: filter chips, mobile nav, rule choices),
  `Segmented`, `Checkbox`, `Pill` (status, tag and badge sizes), `Avatar`
  (tint from `tint.ts`, an `oklch` hue derived from a stable hash of the
  key), `ProgressBar`, `Icon` (the design's SVG set).
- `Modal` (native `<dialog>`, backdrop click and Esc close), `Popover`
  (outside click and Esc close), `ConfirmDialog`, `TextInput`, `Field`.
- `QueryBar` (search pill, live count, syntax popover), `BulkBar` (floating
  action bar with More menu, folder submenu and the permanent-delete
  confirmation), `Toasts` (message toasts plus job toasts with progress,
  Cancel and Undo), `UnsubscribeDialog`, `ConnectDialog`, `GuideModal`,
  `NavRail`, `TopBar`.

## Deliberate deviations from the mock

- Overview says "messages in your mailbox" and "% of your mail": the index
  covers every synced folder, not only the inbox.
- On narrow screens the "Biggest piles" rows use three columns; the mock's
  two-column grid overlaps the Review buttons.
- The Messages page reuses `BulkBar` (the mock only shows it on Senders),
  and Activity offers Undo on finished actions in addition to the toast.
- Copy that the mock fills with sample data (rule sentences, last-run
  summaries, job subtitles) is generated from real data in
  `lib/describe.ts` and `routes/Activity.svelte`.
