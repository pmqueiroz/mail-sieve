---
title: Plugins
description: Turn core plugins like the ghoul wingman, undo history or shared blocklists on or off, and install community plugins that add features and UI to mailsieve.
lead: Plugins add to mailsieve or change how it behaves.
group: Using mailsieve
order: 5
---

Open **Plugins** in the navigation to see every installed plugin, grouped into Core and Community, and switch each one on or off. The choice is saved on the server, so it follows you to every browser. A plugin with settings has a **Configure** button that opens its own settings window; switch the plugin on first.

There are two kinds:

| Kind | Where it comes from |
| --- | --- |
| Core | Ships with :mailsieve. Most are on by default; the AI categorizer and shared blocklists start off. |
| Community | A folder you place in the `plugins` directory. Off until you enable it. |

## Ghoul wingman

The wingman is a small ghoul that floats in the bottom-right corner on wide screens. It reads the same index as the rest of the app and offers one suggestion at a time:

- **A sender you never open.** When every message from a sender is unread and there are at least 20 of them, it offers to move them to trash.
- **A mailing list you never read.** When every message from a list is unread and there are at least 10, it offers to create a rule that archives the list after every sync.
- **A mailbox that stopped syncing.** It offers to start a sync again.

Accepting runs the same action as the rest of the app, so [pinned senders](../senders/#pinned-senders) are still skipped. After a trash or a new rule you get a few seconds to **Undo** right in the balloon; trash can also be undone later from Activity.

Saying no snoozes that suggestion for a week (a sync problem comes back the next day). The **×** sends the ghoul to nap until you click it again, and clicking it while it is awake just pokes it.

:::note
The wingman never deletes permanently and never acts without a click.
:::

### Connecting an AI model

**Configure** on the wingman card picks where its intelligence comes from:

| Option | What it uses |
| --- | --- |
| Built-in rules | The rules above. Nothing leaves your server. This is the default. |
| AI provider | Anthropic, or any OpenAI-compatible API, with your API key. |
| Local with Docker | A model running next to :mailsieve in Ollama or Docker Model Runner. |

With a model connected, the wingman sends it a table of your noisiest senders and mailing lists (names, addresses, message and unread counts and the date of the latest message, never subjects or bodies) and shows the model's picks with its reasons. A pick can only be one of the actions the wingman already knows: move a sender's mail to trash, mark it read, or create a rule that archives a list. The answer is reused for 30 minutes. If the model cannot be reached, the wingman says so once and falls back to the built-in rules.

**Test connection** asks the endpoint for its models and lists them under Model. API keys are encrypted on the server with the same key as your mailbox passwords and are never shown again. If you change the provider or the base URL, the saved key is removed on save and you enter a new one, so a key is never sent somewhere it was not meant for.

For Ollama, add it to the same compose file as :mailsieve and pull a model:

```yaml
services:
  ollama:
    image: ollama/ollama
    volumes:
      - ollama:/root/.ollama

volumes:
  ollama:
```

```text
docker compose exec ollama ollama pull llama3.2
```

The base URL is then `http://ollama:11434/v1`. For Docker Model Runner, turn it on in Docker, run `docker model pull ai/llama3.2`, and use `http://model-runner.docker.internal/engines/v1`.

## Undo history

An **Undo** button in the top bar shows how many recent actions and rule runs can still be undone. Open it for the latest eight, each with its own Undo, long after its toast is gone. Permanent deletes never appear because they cannot be undone.

## Weekly digest

A **This week** section on Overview sums up the last 7 days: how much mail arrived and how much of it is still unread, how many messages you cleaned up and the space freed, which category grew compared with the rest of your mailbox, and the three busiest senders. Click a sender to review it on Senders.

## Rule recipes

**Starter rules** on the Rules screen offer ready-made rules, such as archiving receipts older than 90 days or moving promotions older than 30 days to trash. Each card shows how many messages it would match right now. **Add rule** creates it to run after every sync; you can edit or pause it like any other rule. A recipe disappears once you have the same rule.

## Query builder

Under the search bar on Senders and Messages, every search term shows up as a chip. Click a chip to exclude its matches (it turns red), or its **×** to remove it. **+ Filter** adds a term without typing the [syntax](../search/): pick a field such as From, Category or Older than, then a value.

## Unsubscribe audit

Some senders keep mailing after you unsubscribe. Two days after an unsubscribe completes, Activity lists any sender that still sent you mail, with how many messages arrived since. **View** opens them, **Trash after every sync** adds a rule that moves their mail to trash, and **Dismiss** hides the entry in this browser.

## AI categorizer

Off by default. It splits the People category on Overview into your own groups, such as Work, Finance or Family and friends. **Configure** picks an AI provider or a local Docker model, exactly like the wingman, and the list of categories.

**Sort with AI** sends the names, addresses, message counts and up to three subjects of your top 150 People senders to that model. Message bodies are never sent. Each group then shows its senders and message count; open one to see the senders, or to **Mark all read** or **Archive all** in one go. The result is saved on the server. **Sort again** after new senders arrive or after you change the categories.

## Shared blocklists

Off by default. A **Blocklists** section on the Rules screen checks a public list of spam or tracker domains against your mail. Paste the link to the list and choose **Add and check**. Plain lists of domains, hosts files (`0.0.0.0 ads.example.com`) and adblock lists (`||ads.example.com^`) all work, and a listed domain also covers its subdomains.

The result shows the listed domains that sent you mail. Untick any you want to keep, then:

| Button | What it does |
| --- | --- |
| Trash N messages | Moves their mail to trash now. Undo works as usual. |
| Add N trash rules | Adds a rule per domain that moves new mail to trash after every sync. |
| Pin N domains | Pins the domains, for lists of senders you trust. |

:::note
The server only downloads lists over https from public addresses, never from your local network, and refuses files over 8 MB.
:::

## Community plugins

A community plugin is a folder inside `plugins/` in the data directory (`/data/plugins` in the container). The folder name is the plugin id: lowercase letters, digits and dashes.

```text
/data/plugins/hello-world/
  plugin.json
  index.js
```

`plugin.json` describes it. Only `name` is required; `entry` defaults to `index.js`.

```json
{
  "name": "Hello world",
  "version": "0.1.0",
  "description": "Shows the current page in a badge.",
  "author": "you",
  "entry": "index.js"
}
```

The entry is an ES module whose default export is the plugin. A slot receives a DOM element to render into, the host API and a context, and returns a cleanup function. `configure`, if present, has the same shape and fills the Configure window; its third argument has `close()`. `setup` runs once when the plugin is enabled, for behavior without UI.

```js
export default {
  slots: {
    overlay: {
      mount(target, host) {
        const badge = document.createElement('div');
        target.append(badge);
        const stop = host.subscribe((state) => {
          badge.textContent = `You are on ${state.route}`;
        });
        return () => { stop(); badge.remove(); };
      },
    },
  },
  configure: {
    mount(target, host, { close }) {
      const button = document.createElement('button');
      button.textContent = 'Save greeting';
      button.onclick = async () => {
        await host.config.save('hello-world', { greeting: 'Hi' });
        close();
      };
      target.append(button);
    },
  },
  setup(host) {
    host.toast('Hello world is on');
  },
};
```

These slots are available:

| Slot | Where it renders and its context |
| --- | --- |
| `overlay` | Once, floating over the app. No context. |
| `topbar` | In the top bar, before the account menu. No context. |
| `overview` | On Overview, under the stats. Context: `accountId`. |
| `rules` | On Rules, above your rules. Context: `accountId`, `rules`, `reload()`. |
| `search` | Under the search bar on Senders and Messages. Context: `accountId`, `query`, `setQuery(query)`. |
| `activity` | At the top of Activity. Context: `accountId`. |

Context values are read live, so `context.query` is always the current search.

The host API gives a plugin the same tools the app uses:

| Member | What it does |
| --- | --- |
| `host.api` | `get`, `post`, `put`, `patch`, `delete` against the mailsieve API |
| `host.runAction` | Runs a bulk action with the usual progress and undo toast |
| `host.toast(message, kind)` | Shows a toast (`info`, `success` or `error`) |
| `host.go(route, params)` | Navigates to a screen |
| `host.route`, `host.accountId`, `host.account`, `host.accounts`, `host.jobs` | The current state |
| `host.config.load(id)`, `host.config.save(id, settings, secrets)` | Reads or stores the plugin's settings; `secrets` values are encrypted and only their names come back |
| `host.ai.listModels({ kind, baseUrl, apiKey, plugin })` | Lists the models of an Anthropic (`anthropic`) or OpenAI-compatible (`openai`) endpoint |
| `host.ai.complete({ kind, baseUrl, model, plugin, system, prompt, maxTokens })` | Sends one prompt to that endpoint and returns the answer text; `plugin` uses that plugin's saved `api_key` |
| `host.subscribe(listener)` | Calls the listener now and whenever the route, account or data changes; returns a stop function |

Reload the page after adding or changing files. A plugin that fails to load or throws shows its error on the Plugins screen and is switched off for that session.

:::warning
A community plugin runs inside the app with your session, so it can read and change everything you can. Only install code you have read or trust.
:::
