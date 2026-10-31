---
title: Plugins
description: Turn core plugins like the ghoul wingman on or off, and install community plugins that add features and UI to mailsieve.
lead: Plugins add to mailsieve or change how it behaves.
group: Using mailsieve
order: 5
---

Open **Plugins** in the navigation to see every installed plugin, grouped into Core and Community, and switch each one on or off. The choice is saved on the server, so it follows you to every browser. A plugin with settings has a **Configure** button that opens its own settings window; switch the plugin on first.

There are two kinds:

| Kind | Where it comes from |
| --- | --- |
| Core | Ships with :mailsieve. On by default. |
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

:::note
This release stores and tests the connection. Suggestions still come from the built-in rules in every mode.
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

The entry is an ES module whose default export is the plugin. A slot receives a DOM element to render into and the host API, and returns a cleanup function. `configure`, if present, has the same shape and fills the Configure window; its third argument has `close()`. `setup` runs once when the plugin is enabled, for behavior without UI.

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
| `host.subscribe(listener)` | Calls the listener now and whenever the route, account or data changes; returns a stop function |

Reload the page after adding or changing files. A plugin that fails to load or throws shows its error on the Plugins screen and is switched off for that session.

:::warning
A community plugin runs inside the app with your session, so it can read and change everything you can. Only install code you have read or trust.
:::
