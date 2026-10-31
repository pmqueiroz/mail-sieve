# Design system

The web UI implements the Claude Design handoff "Mail Sieve" (`Mail
Sieve.dc.html`), its `Login.dc.html` for sign-in and `Wingman Ghoul.dc.html`
for the wingman plugin. Tailwind v4 is the only styling layer; every value comes
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
  sm 13 · base 14 · md 16 · lg 18 · xl 20 · 2xl 26 · 3xl 28 · wordmark 36 ·
  hero-sm 72 · hero 104`. Default Tailwind scales (fonts, text, radius, shadows,
  breakpoints, colors) are reset with `--*: initial` so only design values
  exist.
- **Breakpoints** mirror the design's JS layout switches: `wide` 760px
  (rail and full table columns), `split` 1100px (message preview docks
  beside the list), `board` 1240px (Overview's two-column board).
- **Utilities**: `bg-page-glow` (Login's radial surface-to-page glow),
  `bg-sieve` (the coral/amber/mint gradient), `bg-sieve-bloom`,
  `bg-ticks` (the striped never-opened track) and `border-check` (the
  1.5px checkbox/radio stroke).
- **Wingman tokens**: `ghoul-hi body shade stroke eye accent alert
  ground` colors (the design's "mist" tint; `ground` and the
  `drop-shadow-ghoul` glow have dark values), and the `animate-ghoul-*`,
  `animate-balloon-pop|fade` and `animate-countdown` animations with their
  `@keyframes` inside `@theme`. Every animated wingman element also carries
  `motion-reduce:animate-none`. SVG fills and strokes use `fill-*` /
  `stroke-*` utilities and gradient stops `[stop-color:var(...)]`, so no
  hex lives in markup. SVG transform origins use
  `[transform-box:fill-box]` plus `origin-left|right|center` instead of the
  design's pixel origins.
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
  `xs 28 · sm 30 · md 32 · lg 34 · xl 36 · 2xl 38 · dialog 40 · block 48`,
  `block` is full width).
- `PasswordField` (label, 48px bordered field with a Show/Hide reveal
  toggle, inline error or hint; the border turns `bad` on error).
- `Chip` (toggle pill: filter chips, mobile nav, rule choices),
  `Segmented`, `Checkbox`, `Pill` (status, tag and badge sizes), `Avatar`
  (tint from `tint.ts`, an `oklch` hue derived from a stable hash of the
  key), `ProgressBar`, `Icon` (the design's SVG set).
- `Modal` (native `<dialog>`, backdrop click and Esc close), `Popover`
  (outside click and Esc close), `ConfirmDialog`, `TextInput`, `Field`.
- `PinButton` (pin toggle for a sender or domain, used on Senders rows
  and in the message preview; outline `pin` icon when unpinned, filled
  `pinned` when pinned, disabled when a different pin covers the key) and
  `PinnedConfirm` (the "Act on a pinned sender?" confirmation, a
  `ConfirmDialog` with the action as its label). `BulkBar` takes a
  `skipped` count and appends "N from pinned senders are skipped".
- Plugins: `PluginSlot` / `MountSlot` (`lib/plugins/`) render plugin
  content; the wingman's `Ghoul` and `Balloon` live with the plugin in
  `src/plugins/wingman/` because nothing else uses them.
  `AiSettingsForm` (`lib/plugins/ai/`) is the one AI provider form; any
  plugin that needs a model reuses it and adds its own fields through
  the `extra` snippet. Plugin sections on Overview follow the stat row
  pattern (`section` with `border-b border-line py-12`, an `lg` heading
  and a muted note); plugin cards on Rules and Activity reuse the
  `rounded-3xl border border-line` card, with a dashed border for
  suggestions that do not exist yet (rule recipes). The query builder's
  term chips match `Chip` height (`h-17`) and turn `bad` with a struck
  value when negated. The Plugins page
  has Core and Community sections, each a card grid like Accounts with a
  `Checkbox` per plugin and a Configure `Button`; an empty section is an
  `EmptyState` inside a dashed card. Configure opens the shared `Modal`
  (wide) with the plugin's own content. The wingman form wraps each
  `Segmented` in a `flex` row so it sizes to its options instead of
  stretching in the column.
- `QueryBar` (search pill, live count, syntax popover), `BulkBar` (floating
  action bar with More menu, folder submenu and the permanent-delete
  confirmation), `Toasts` (message toasts plus job toasts with progress,
  Cancel and Undo), `UnsubscribeDialog`, `ConnectDialog`, `GuideModal`,
  `NavRail`, `TopBar`.
- `Brand` (the inline wordmark: "mail" inherits the text color, "sieve" is
  bold `text-ink`) and `BrandText` (renders a plain string with every
  "mailsieve" swapped for `Brand`). `ConfirmDialog` messages,
  `EmptyState` titles and `GuideModal` text go through `BrandText`, so
  strings in `guides.ts` and dialog props just write "mailsieve". The
  Login logo keeps its own larger wordmark.

## Deliberate deviations from the mock

- No outer frame: the mock presents the app as a rounded card inset 24px
  on a lavender canvas. That is design-file presentation, so the app fills
  the window and the rail is full height. `page` stays a token for Login.
- Login is drawn dark-only in `Login.dc.html`. It is built from the theme
  tokens, so dark mode matches the mock exactly and light mode gets the
  same layout in light colors (`btn-hover` and `shadow-card` have light
  values picked to match). The mock's fake "Incorrect password" timeout
  is the real `/auth/login` 401. The same page also serves first-run
  setup ("Welcome", a confirm field and the 10-character hint), which the
  mock does not cover. The footer version comes from `package.json` via
  Vite's `__APP_VERSION__` define.

- Overview says "messages in your mailbox" and "% of your mail": the index
  covers every synced folder, not only the inbox.
- On narrow screens the "Biggest piles" rows use three columns; the mock's
  two-column grid overlaps the Review buttons.
- The Messages page reuses `BulkBar` (the mock only shows it on Senders),
  and Activity offers Undo on finished actions in addition to the toast.
- Wingman: the design's live demo is the only placement reference, so the
  ghoul sits fixed 28px/22px from the bottom-right corner of the window
  with balloons offset exactly as in the mock, and it is hidden below
  `wide`. The demo's "Delete 412" says "Trash 412" (the copy rule:
  destructive actions say what happens, and the done text already said
  "moved to trash"). The "Reconnect" heads-up is "Sync now", the only
  thing the app can actually do there. "Reading N new headers" became the
  sync percentage, the figure jobs report. The 1c "Choice" balloon
  (archive / delete / keep plus "Save as a rule") and the "Tip" balloon are
  not built yet; the newsletter suggestion uses the plain two-button
  balloon instead. The tint options (`lavender`, `ink`) are not exposed.
- Copy that the mock fills with sample data (rule sentences, last-run
  summaries, job subtitles) is generated from real data in
  `lib/describe.ts` and `routes/Activity.svelte`.
