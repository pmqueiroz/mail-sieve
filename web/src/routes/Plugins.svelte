<script lang="ts">
  import { errorMessage } from '../lib/api';
  import Brand from '../lib/components/Brand.svelte';
  import BrandText from '../lib/components/BrandText.svelte';
  import Button from '../lib/components/Button.svelte';
  import Checkbox from '../lib/components/Checkbox.svelte';
  import EmptyState from '../lib/components/EmptyState.svelte';
  import Modal from '../lib/components/Modal.svelte';
  import PluginAvatar from '../lib/plugins/PluginAvatar.svelte';
  import PluginContent from '../lib/plugins/PluginContent.svelte';
  import { plugins } from '../lib/plugins/registry.svelte';
  import type { PluginEntry, PluginSource } from '../lib/plugins/types';
  import { toasts } from '../lib/toasts.svelte';

  const SECTIONS: { source: PluginSource; title: string; note: string }[] = [
    { source: 'core', title: 'Core', note: 'Ship with mailsieve' },
    { source: 'community', title: 'Community', note: 'Installed from the plugins directory' },
  ];

  let pending = $state<Record<string, boolean>>({});
  let configuring = $state<PluginEntry | null>(null);
  let configOpen = $state(false);

  const configure = $derived(configuring?.definition?.configure ?? null);

  async function toggle(entry: PluginEntry) {
    if (pending[entry.id]) return;
    pending[entry.id] = true;
    try {
      await plugins.setEnabled(entry.id, !entry.enabled);
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    } finally {
      pending[entry.id] = false;
    }
  }

  function openConfig(entry: PluginEntry) {
    configuring = entry;
    configOpen = true;
  }
</script>

{#snippet card(entry: PluginEntry)}
  {@const meta = [entry.author && `by ${entry.author}`, entry.version && `v${entry.version}`].filter(Boolean).join(' · ')}
  <article class="flex flex-col gap-7 rounded-3xl border border-line p-10">
    <div class="flex min-w-0 items-center gap-6">
      <PluginAvatar avatar={entry.avatar} id={entry.id} name={entry.name} />
      <div class="flex min-w-0 flex-col leading-row">
        <span class="font-semibold">{entry.name}</span>
        {#if entry.source === 'community' && meta}<span class="truncate text-xs text-ink3">{meta}</span>{/if}
      </div>
    </div>
    {#if entry.description}<p class="m-0 text-pretty text-sm leading-normal text-ink2">{entry.description}</p>{/if}
    {#if entry.error}
      <div class="rounded-xl bg-bad-bg p-7 text-sm text-pretty text-bad">Could not load: {entry.error}</div>
    {/if}
    <div class="mt-auto flex items-center gap-5">
      <Checkbox checked={entry.enabled} label="Enable {entry.name}" onchange={() => toggle(entry)} />
      <span class="flex-1 text-sm text-ink2">{entry.enabled ? 'Enabled' : 'Disabled'}</span>
      {#if entry.definition?.configure}
        <Button size="md" disabled={!entry.enabled || !!entry.error} title={entry.enabled ? undefined : 'Enable the plugin to configure it'} onclick={() => openConfig(entry)}>Configure</Button>
      {/if}
    </div>
  </article>
{/snippet}

{#if !plugins.loaded}
  <div class="min-h-60" aria-busy="true"></div>
{:else}
  <div class="flex flex-col gap-14 pt-2">
    <p class="m-0 max-w-300 text-pretty leading-normal text-ink2">
      Plugins add to <Brand /> or change how it behaves. Core plugins ship with the app. Community plugins are folders you place in the
      <code class="font-mono text-xs text-ink">plugins</code> directory inside the data directory; they run with the same access to your mailboxes as you do, so only enable code you trust.
    </p>
    {#each SECTIONS as section (section.source)}
      {@const entries = plugins.entries.filter((entry) => entry.source === section.source)}
      <section class="flex flex-col gap-8" aria-label="{section.title} plugins">
        <div class="flex items-baseline gap-6">
          <h2 class="m-0 text-lg font-semibold">{section.title}</h2>
          <span class="text-sm text-ink3"><BrandText text={section.note} /></span>
        </div>
        {#if entries.length > 0}
          <div class="grid grid-cols-[repeat(auto-fill,minmax(--spacing(160),1fr))] gap-8">
            {#each entries as entry (entry.id)}
              {@render card(entry)}
            {/each}
          </div>
        {:else}
          <div class="rounded-3xl border-check border-dashed border-line">
            <EmptyState title="No community plugins yet">
              <p class="m-0 leading-normal">
                Add a folder with a <code class="font-mono text-xs text-ink">plugin.json</code> and an entry module to
                <code class="font-mono text-xs text-ink">plugins</code> in the data directory, then reload this page. It shows up here switched off.
              </p>
            </EmptyState>
          </div>
        {/if}
      </section>
    {/each}
  </div>
{/if}

<Modal bind:open={configOpen} title="Configure {configuring?.name ?? ''}" wide>
  {#if configuring && configure}
    <PluginContent id={configuring.id} content={configure} context={{ close: () => (configOpen = false) }} />
  {/if}
</Modal>
