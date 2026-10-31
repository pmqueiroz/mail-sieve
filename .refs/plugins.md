# Plugins

Plugins add features or UI to the app without touching its core. There are
**core** plugins, compiled into the SPA, and **community** plugins, ES
modules dropped into `<data dir>/plugins/<id>/` and loaded at runtime. The
core plugins are the ghoul wingman, undo history, weekly digest, rule
recipes, query builder, unsubscribe audit, AI categorizer and shared
blocklists.

## Shape

The plugin system lives in the web app; the server only stores on/off
state and serves community files. Rationale: everything a plugin does goes
through the same HTTP API the UI uses, so the server needs no plugin hooks
and every existing guard (pins, `confirm`, rule validation) still applies.

- `web/src/lib/plugins/types.ts`: `PluginDefinition` = optional `slots`,
  optional `configure` and optional `setup(host)`. Slot and configure
  content (`Content<P>`) is either a Svelte component (core plugins) or
  `{ mount(target, host, context) }` returning a cleanup (community
  plugins, framework agnostic, since they cannot share our compiled Svelte
  runtime). `context` is `{ close }` plus the slot's own values
  (`SlotContexts`); components receive the same as props. `MountSlot`
  hands `mount` a proxy that reads the current context on every access,
  so a community plugin sees the live query or account without being
  remounted.
- `web/src/plugins/index.ts`: `CORE_PLUGINS`, the list of built-in plugins.
  A core plugin adds `id`, `name`, `description`, `version`, `avatar`
  and `defaultEnabled`. Every core plugin ships with `defaultEnabled: false`:
  plugins are opt-in, so a fresh install shows only the core app and the
  user turns on what they want on the Plugins page.
- `registry.svelte.ts` (`plugins`): loads `GET /api/plugins` once after
  sign-in (`App.svelte`), merges core and community entries (a community
  folder with a core id is ignored), imports enabled community modules with
  a dynamic `import()`, runs `setup`, and keeps cleanups to run on disable.
  `fail(id, error)` records the error on the entry and drops it from the
  slots; it is not retried until the page reloads.
- `PluginContent.svelte` renders one piece of plugin content inside a
  `<svelte:boundary>`, so a throwing plugin is disabled instead of
  breaking the app; `MountSlot.svelte` adapts `mount` functions.
  `PluginSlot.svelte` renders every active plugin's content for a slot
  with it, and the Plugins page renders `configure` with it.
- The Plugins page (`routes/Plugins.svelte`) lists core and community
  plugins in two sections; Community shows an empty state with install
  instructions when nothing is installed. A plugin with `configure` gets a
  Configure button (disabled while the plugin is off) that opens a `Modal`
  whose whole body is the plugin's content. The plugin owns its form,
  including Save and Cancel, and calls `close` when done. Each card opens
  with the plugin's avatar and name; community cards add a
  `by <author> · v<version>` line, core cards do not.
- Avatars (`lib/plugins/avatar.ts`, `PluginAvatar.svelte`, from the
  "Plugin Avatars" design handoff): one preset icon and one preset color,
  never an uploaded image, so every plugin stays legible and on-palette.
  `PLUGIN_ICONS` holds the 29 icon names (Lucide paths), `PLUGIN_COLORS`
  the 9 color names and their hues (`gray` has no chroma). Core plugins
  set `avatar: { icon, color }` in code; community plugins set it in
  `plugin.json`. `resolveAvatar` falls back per field: an unknown or
  missing icon shows the name's monogram (first letter of the first two
  words), an unknown or missing color is picked from the non-gray colors
  by a hash of the plugin id, so a plugin keeps its color across reloads.
- Slots (`SlotContexts` in `types.ts`, the context each one passes):
  - `overlay`: once at the end of the signed-in shell, for floating UI.
    No context.
  - `topbar`: in `TopBar`, between Sync now and the account menu. No
    context.
  - `overview`: on Overview between the stat row and Biggest piles.
    `{ accountId }`. Each plugin renders its own bordered `section`.
  - `rules`: on Rules above the rule cards. `{ accountId, rules, reload }`,
    where `rules` is the account's rules and `reload` refreshes the page
    after a plugin creates one.
  - `search`: under the `QueryBar` on Senders and Messages.
    `{ accountId, query, setQuery }`; `setQuery` replaces the page's
    query, so the plugin drives the same search the bar does.
  - `activity`: on Activity above the pending unsubscribe links.
    `{ accountId }` (may be `null`).
  Adding a slot: add it to `SlotContexts` and place a
  `<PluginSlot name="..." context={...} />` where it should render.
- `host.svelte.ts` (`host`, `HOST_API_VERSION = 1`) is the stable surface
  for community code: `api`, `runAction`, `toast`, `go`,
  `config.load|save`, `ai.listModels` and `ai.complete` (see
  `lib/plugins/config.ts`),
  live getters for
  route/account/accounts/jobs, and `subscribe(listener)`, an
  `$effect.root` that re-runs on route, account and `dataVersion` changes.
  Bump the version when its shape changes incompatibly.

## Server

`crates/server/src/plugins.rs` + `api/plugins.rs`:

- `GET /api/plugins` -> `{ states, community }`. `states` is the
  `plugins.enabled` row of `settings` (JSON id -> bool). Missing ids fall
  back to the plugin default (core: `defaultEnabled`, community: off).
- `PUT /api/plugins/{id}` `{ enabled }` writes one id. Any valid id is
  accepted; the server does not know the core list.
- `GET /api/plugins/{id}/files/{*path}` serves a community file. Behind
  the session like every other API route.
- `GET /api/plugins/{id}/config` -> `{ settings, secrets }`: `settings`
  is the plugin's JSON (`null` when never saved, at most 64 KB), `secrets`
  only the *names* of stored secrets. `PUT` takes `{ settings, secrets }`
  where each secret is a string to set, or `null` / empty to remove;
  omitted names are kept. Settings live in `settings` as
  `plugins.config.<id>`; secrets as one vault-sealed map in
  `plugins.secrets.<id>` (base64). Secret values never leave the server.
- `POST /api/ai/models` `{ kind: anthropic|openai, base_url, api_key?,
  plugin? }` lists a provider's models (`ai.rs`), the connection test of
  any AI settings form. A blank `api_key` with `plugin` uses that plugin's
  stored `api_key` secret (`resolve_key`, shared with `/ai/complete`). Anthropic: `GET {base}/v1/models` with
  `x-api-key` and `anthropic-version: 2026-10-21`. `openai` is any
  OpenAI-compatible server (OpenAI, Ollama, Docker Model Runner, vLLM):
  `GET {base}/models` with an optional bearer token; `base` already ends
  in `/v1`. 10 s timeout; failures are 502 with the provider's
  `error.message` or the innermost transport error (DNS, refused).
- `POST /api/ai/complete` `{ kind, base_url, model, api_key?, plugin?,
  system?, prompt, max_tokens? }` -> `{ text }`: one single-turn
  completion. Anthropic: `POST {base}/v1/messages` (text blocks joined,
  other block types such as thinking ignored). `openai`:
  `POST {base}/chat/completions`, `choices[0].message.content`.
  `max_tokens` defaults to 1024 and is clamped to 4096; system plus
  prompt are capped at 100 KB; 90 s timeout. Same error mapping as the
  model list. It is a plain relay, so the caller parses the text.
- `GET /api/accounts/{id}/unsubscribes/still-sending` lists senders whose
  unsubscribe `completed` and who still sent mail more than two days
  later (`unsubscribes::still_sending`, grace in
  `UNSUBSCRIBE_GRACE_SECONDS`): address, name, method, unsubscribe time,
  messages since, latest date. Two days is the window big mailbox
  providers give senders to honor one-click requests; mail inside it is
  not counted. `record` and `mark_link_done` both reset `at`, so the
  window always starts at the latest attempt.
- `POST /api/accounts/{id}/blocklists/preview` `{ url }` fetches a
  public text file through the unsubscribe SSRF guard
  (`unsub::fetch_public_text`, 8 MB cap), parses it with
  `core::blocklist::parse_domains` and joins it against
  `messages::domain_counts`. Returns `{ listed, matches }`, at most 500
  matches sorted by messages, each `{ domain, messages, sender_domains }`
  where `domain` is the listed entry and `sender_domains` the real sender
  domains it covers (a listed domain covers its subdomains, like
  `domain:`). Guard rejections and size errors are 400, fetch failures
  502.
- Discovery reads `plugin.json` (`name` required; `version`,
  `description`, `author`, `avatar` `{ icon?, color? }`, `entry` default
  `index.js`) on every list call. The server passes `avatar` through
  without checking names; the web app owns the preset list.
  A folder with an unreadable manifest or missing entry is still listed
  with `error`, so the user sees why it does not load. Folders whose name
  is not a valid id (`[a-z0-9][a-z0-9-]{0,63}`) are skipped.

See [security.md](./security.md) for the trust model and the path guard.

## Ghoul wingman (`web/src/plugins/wingman/`)

Implements the "Wingman Ghoul" design handoff (`Wingman Ghoul.dc.html`).

- `Ghoul.svelte`: the SVG mascot with six moods (`idle think suggest
  happy alert sleep`). Eyes follow the pointer (`look`, at most 4 x 3.2
  user units, scaled down within 180px of the ghoul, rAF throttled).
- `Balloon.svelte`: the speech balloon shell. The square bottom-right
  corner is the tail and always points at the ghoul.
- `suggestions.ts`: builds suggestions from existing endpoints, in
  priority order:
  1. sync failure (`account.last_error`, not while syncing) -> start a sync;
  2. a non-list sender whose every message is unread, at least 20 ->
     trash `from:<addr>`;
  3. a list whose every message is unread, at least 10, with no rule on
     the account already containing `list:<id>` -> create an
     `archive` rule with `run_after_sync`.
  Senders that carry a `list_id` are left to (3): a list keeps sending, so
  a rule is the better fix than a one-off trash. Pinned groups (`pin`) are
  never suggested, and the action itself still goes through the pin guard.
- `wingman.svelte.ts`: the state machine. Phases `idle suggest working
  done ack sleep`. Accept snoozes the key for a day (so the same item does
  not reappear while the index catches up); decline snoozes for
  `snoozeDays` (7, sync failure 1) and shows the ack for 2.2 s; done shows
  for 3 s with a countdown and Undo. A trash job is watched until it
  finishes (`working` shows the thinking balloon), its generic "Done"
  toast is suppressed with `toasts.silence(jobId)`, and Undo calls
  `POST /jobs/{id}/undo` and hands the restore job to the normal job
  toast. Rule Undo deletes the rule. Napping and snoozes persist in
  `localStorage` (`mail-sieve.wingman`), per browser.
- While a sync runs and nothing else is showing, the ghoul thinks and the
  balloon shows the sync percentage.
- `WingmanSettings.svelte`: the configure modal, a thin wrapper over the
  shared `AiSettingsForm` (see AI settings below) with all three modes.
- Model-backed suggestions (`model.ts`): in `cloud` or `docker` mode,
  `refresh` loads the stored settings and `aiTarget` turns them into a
  completion target. The model gets one tab-separated table of the
  candidate pools (non-list senders and unruled lists, unpinned, with
  unread mail; at most 25 each) and must answer a JSON array of
  `{ id, action, reason }`. Senders allow `trash` or `mark_read`, lists
  `archive_rule` or `mark_read`; anything else, unknown ids and
  duplicates are dropped, at most 5 picks survive. The reason becomes the
  balloon text (em dashes replaced, 180 chars max). Picks are cached per
  account and target for 30 minutes, because `refresh` runs after every
  job. A failure toasts once, is cached the same way, and falls back to
  the built-in rules. The sync-failure suggestion is always rule-based.
  While the call runs and nothing else shows, the ghoul thinks with
  "Looking through your senders". Model suggestions use their own snooze
  keys (`ai-trash:`, `ai-archive:`, `ai-read:`), so declining a rule
  suggestion does not hide the model's take on the same sender.
- Hidden below the `wide` breakpoint. While mounted it sets
  `toasts.shifted`, which moves the toast stack left of the ghoul.

## AI settings (`web/src/lib/plugins/ai/`)

Shared by every plugin that talks to a model, so the provider form, key
handling and copy exist once.

- `settings.ts`: `AiSettings` (`mode` plus a `cloud` and a `docker`
  sub-object, so switching modes keeps both), the provider and runtime
  presets, `withAiDefaults(stored, mode)` and `aiTarget(settings,
  pluginId)`, which returns the completion target or `null` in `rules`
  mode. Cloud targets pass the plugin id so the server uses its stored
  key; Docker targets use `kind: openai` without a key.
- `AiSettingsForm.svelte`: the whole configure body, generic over the
  plugin's settings type (`T extends AiSettings`). Props: `pluginId`,
  `name` (toast text), `modes` (which of `rules cloud docker` to offer),
  `offText` (the `rules` mode explanation), `defaults` (fills a stored
  partial) and an `extra` snippet for plugin fields, rendered above Save
  and bound to the same settings object. It loads and saves the config
  itself. The saved key is only reused while the provider and base URL
  match the ones it was saved with; otherwise Save removes it.
- `json.ts`: `extractJson` strips code fences and parses the first JSON
  array or object in a model answer. Models wrap JSON in prose often
  enough that every caller needs it.

## Undo history (`web/src/plugins/undo-history/`)

`topbar` slot, hidden below `wide`. A Button with the
count of undoable jobs opens a `Popover` listing the newest 8 from
`session.jobs` (`isUndoable` in `lib/jobs.ts`, the same test Activity
uses: finished actions that are not permanent deletes, not undos and not
undone yet). Undo calls `undoJob`, which posts `/jobs/{id}/undo` and
tracks the restore toast. The job list does not refresh the original
job's `undone_by`, so undone ids are hidden locally.

## Weekly digest (`web/src/plugins/digest/`)

`overview` slot. A "This week" row built from existing
endpoints, reloaded on `dataVersion`: arrived (`/count` for
`newer_than:7d`, plus its unread share), cleaned up (finished action
jobs of the account in the last 7 days from `session.jobs`, which holds
the newest 100 jobs, and `freed_week_bytes`), mix shift (the category
whose share of this week's mail grew most against its share in
`/overview`, or "Steady" when none grew) and the three busiest senders
of the week, each opening Senders preselected.

## Rule recipes (`web/src/plugins/rule-recipes/`)

`rules` slot. Six fixed recipes (`recipes.ts`), each a
card with the rule sentence from `describeRule`, a live match count
(`liveTotals`, with the pinned note) and Add rule, which creates the
rule with `run_after_sync` and calls `reload`. A recipe disappears once
the account has a rule with the same query and action; the section
disappears with the last one. Rules skip pinned senders on their own.

## Query builder (`web/src/plugins/query-builder/`)

`search` slot. Splits the query with `splitTerms` /
`parseTerm` (`lib/query.ts`, quote aware, also used by `describeRule`)
and shows one chip per term: clicking the chip toggles its `-`, the ×
removes it, and `+ Filter` opens a `Popover` with a field picker
(`fields.ts`), a value control per kind (text, choice chips, number plus
unit, date) and Exclude matches. Every change rebuilds the string with
`formatTerm` and calls `setQuery`. There is no OR: the DSL has none and
the list endpoints take a single query.

## Unsubscribe audit (`web/src/plugins/unsubscribe-audit/`)

`activity` slot. Shows the still-sending list with View
(Messages for that sender) and "Trash after every sync", which creates a
`trash` rule on `from:<addr>` with `run_after_sync`; the button becomes
"Trash rule active" when a rule already contains that term. Dismiss
hides an entry in `localStorage` (`mail-sieve.unsubscribe-audit`),
keyed by account, address and unsubscribe time, so a later unsubscribe
of the same sender shows again.

## AI categorizer (`web/src/plugins/ai-categorizer/`)

`overview` slot plus `configure`. It needs a
model and sends headers to it. Settings (`settings.ts`) are `AiSettings`
(modes `cloud` and `docker` only) plus `categories`, a comma-separated
string (parsed by `categoryList`, which always appends `Other`), and
`sorted`, the last result per account id: `{ at, senders: { addr:
category } }`. Results live in the plugin config so they follow the
user across browsers; 150 senders fit well inside the 64 KB limit.

Sort with AI (`categorize.ts`) takes the top 150 senders of
`category:people` by count and up to 3 subjects each from the newest
1000 People messages, asks for a JSON object mapping row ids to
categories (unknown names become `Other`, matching is case-insensitive)
and saves the result. Buckets are rebuilt from fresh group counts on
every load, so counts stay current and senders that left People drop
out; senders not in the saved result are counted as "not sorted yet".
A bucket opens `BucketDialog` with the senders, View per sender, and
Mark all read / Archive all, which run one action with a `queries`
union of `from:` terms (pinned senders skipped as usual).

## Shared blocklists (`web/src/plugins/blocklists/`)

`rules` slot. The section stores list URLs in the
plugin config (`{ lists: [{ url }] }`); Add and check saves the URL and
calls the preview endpoint. `Matches.svelte` lists the matched entries
with checkboxes (all on) and three actions over the checked ones: Trash
N messages (one action with a `queries` union of `domain:` terms), Add N
trash rules (one `domain:` rule per entry named `Blocklist: <domain>`,
skipping entries that already have one) and Pin N domains, for lists
that are allowlists. Supported formats are whatever
`core::blocklist::parse_domains` reads: plain domains, hosts files
(`0.0.0.0 domain`), adblock (`||domain^`), `*.` and `@` prefixes and
addresses (their domain); comments (`#`, `!`), IPs, single labels and
`localhost` names are skipped, at most 200,000 entries.

