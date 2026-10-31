<script lang="ts">
  import { untrack } from 'svelte';
  import { api, errorMessage } from '../../lib/api';
  import Button from '../../lib/components/Button.svelte';
  import TextInput from '../../lib/components/TextInput.svelte';
  import { loadPluginConfig, savePluginConfig } from '../../lib/plugins/config';
  import { toasts } from '../../lib/toasts.svelte';
  import type { BlocklistPreview, Rule } from '../../lib/types';
  import Matches from './Matches.svelte';
  import { type BlocklistSettings, PLUGIN_ID, withBlocklistDefaults } from './settings';

  let { accountId, rules, reload }: { accountId: number; rules: Rule[]; reload: () => void } = $props();

  let settings = $state<BlocklistSettings>(withBlocklistDefaults(null));
  let draft = $state('');
  let checking = $state<string | null>(null);
  let result = $state<{ url: string; preview: BlocklistPreview } | null>(null);

  $effect(() => {
    untrack(() =>
      loadPluginConfig<Partial<BlocklistSettings>>(PLUGIN_ID)
        .then((config) => (settings = withBlocklistDefaults(config.settings)))
        .catch((error) => toasts.show(errorMessage(error), 'error')),
    );
  });

  $effect(() => {
    void accountId;
    untrack(() => (result = null));
  });

  async function save(next: BlocklistSettings) {
    await savePluginConfig(PLUGIN_ID, next);
    settings = next;
  }

  async function check(url: string) {
    checking = url;
    try {
      const preview = await api.post<BlocklistPreview>(`/accounts/${accountId}/blocklists/preview`, { url });
      result = { url, preview };
    } catch (error) {
      toasts.show(errorMessage(error), 'error', 10000);
    } finally {
      checking = null;
    }
  }

  async function add() {
    const url = draft.trim();
    if (!url) return;
    try {
      if (!settings.lists.some((list) => list.url === url)) await save({ lists: [...settings.lists, { url }] });
      draft = '';
      await check(url);
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }

  async function remove(url: string) {
    try {
      await save({ lists: settings.lists.filter((list) => list.url !== url) });
      if (result?.url === url) result = null;
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }
</script>

<section class="mb-12 flex flex-col gap-6 rounded-3xl border border-line p-10" aria-label="Blocklists">
  <div class="flex flex-col gap-2">
    <h2 class="m-0 text-lg font-semibold">Blocklists</h2>
    <span class="text-pretty text-sm text-ink2">Check a public list of spam or tracker domains against your mail. Plain domain lists, hosts files and adblock lists work. Only https links to public servers are fetched.</span>
  </div>
  {#each settings.lists as list (list.url)}
    <div class="flex flex-wrap items-center gap-4">
      <code class="min-w-0 flex-1 truncate font-mono text-xs text-ink2" title={list.url}>{list.url}</code>
      <Button size="sm" busy={checking === list.url} disabled={checking !== null} onclick={() => check(list.url)}>{result?.url === list.url ? 'Check again' : 'Check'}</Button>
      <Button size="sm" variant="danger-text" onclick={() => remove(list.url)}>Remove</Button>
    </div>
  {/each}
  <div class="flex flex-wrap items-center gap-4">
    <span class="flex min-w-120 flex-1 flex-col"><TextInput mono type="url" spellcheck="false" aria-label="Blocklist URL" placeholder="https://example.com/blocklist.txt" bind:value={draft} onkeydown={(event) => event.key === 'Enter' && add()} /></span>
    <Button busy={checking !== null && checking === draft.trim()} disabled={!draft.trim() || checking !== null} onclick={add}>Add and check</Button>
  </div>
  {#if result}
    {#key result}
      <Matches {accountId} preview={result.preview} {rules} {reload} />
    {/key}
  {/if}
</section>
