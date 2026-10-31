# Plugins

Plugins add features or UI to the app without touching its core. There are
**core** plugins, compiled into the SPA, and **community** plugins, ES
modules dropped into `<data dir>/plugins/<id>/` and loaded at runtime. The
first core plugin is the ghoul wingman.

## Shape

The plugin system lives in the web app; the server only stores on/off
state and serves community files. Rationale: everything a plugin does goes
through the same HTTP API the UI uses, so the server needs no plugin hooks
and every existing guard (pins, `confirm`, rule validation) still applies.

- `web/src/lib/plugins/types.ts`: `PluginDefinition` = optional `slots`,
  optional `configure` and optional `setup(host)`. Slot and configure
  content is either a Svelte component (core plugins) or
  `{ mount(target, host, context) }` returning a cleanup (community
  plugins, framework agnostic, since they cannot share our compiled Svelte
  runtime). `context` is `{ close }`; components receive the same as props.
- `web/src/plugins/index.ts`: `CORE_PLUGINS`, the list of built-in plugins.
  A core plugin adds `id`, `name`, `description`, `version` and
  `defaultEnabled`.
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
  including Save and Cancel, and calls `close` when done.
- Slots today: `overlay` (rendered once at the end of the signed-in shell,
  for floating UI). Adding a slot: extend `SlotName` and place a
  `<PluginSlot name="..." />` where it should render.
- `host.svelte.ts` (`host`, `HOST_API_VERSION = 1`) is the stable surface
  for community code: `api`, `runAction`, `toast`, `go`,
  `config.load|save` and `ai.listModels` (see `lib/plugins/config.ts`),
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
  stored `api_key` secret. Anthropic: `GET {base}/v1/models` with
  `x-api-key` and `anthropic-version: 2026-10-21`. `openai` is any
  OpenAI-compatible server (OpenAI, Ollama, Docker Model Runner, vLLM):
  `GET {base}/models` with an optional bearer token; `base` already ends
  in `/v1`. 10 s timeout; failures are 502 with the provider's
  `error.message` or the innermost transport error (DNS, refused).
- Discovery reads `plugin.json` (`name` required; `version`,
  `description`, `author`, `entry` default `index.js`) on every list call.
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
- `WingmanSettings.svelte` + `settings.ts`: the configure modal. Mode
  `rules` (built-in, default), `cloud` (Anthropic or OpenAI-compatible:
  base URL, API key, model) or `docker` (Ollama or Docker Model Runner,
  with the compose snippet and pull command; both are OpenAI-compatible,
  so `docker` tests with `kind: openai` and no key). Each mode keeps its
  own sub-object so switching back restores it. Test connection fills a
  `datalist` with the models. The saved key is only reused while the
  provider and base URL match the ones it was saved with; otherwise the
  field asks for a new key and Save removes the old one, so a key is
  never sent to a different endpoint.
- **Not wired yet:** the suggestions still come from the rules above in
  every mode. The settings are stored and testable so a model-backed
  suggester can read them (`loadPluginConfig('wingman')` plus the
  `api_key` secret server-side) without another settings change.
- Hidden below the `wide` breakpoint. While mounted it sets
  `toasts.shifted`, which moves the toast stack left of the ghoul.
