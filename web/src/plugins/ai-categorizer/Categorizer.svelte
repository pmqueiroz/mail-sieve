<script lang="ts">
  import { errorMessage } from '../../lib/api';
  import Button from '../../lib/components/Button.svelte';
  import { formatNumber, formatRelative } from '../../lib/format';
  import { aiTarget } from '../../lib/plugins/ai/settings';
  import { loadPluginConfig } from '../../lib/plugins/config';
  import { router } from '../../lib/router.svelte';
  import { session } from '../../lib/session.svelte';
  import { toasts } from '../../lib/toasts.svelte';
  import type { Group } from '../../lib/types';
  import BucketDialog from './BucketDialog.svelte';
  import { type Bucket, buckets, MAX_SENDERS, peopleSenders, sortPeople } from './categorize';
  import { type CategorizerSettings, categoryList, PLUGIN_ID, withCategorizerDefaults } from './settings';

  let { accountId }: { accountId: number } = $props();

  let settings = $state<CategorizerSettings | null>(null);
  let senders = $state<Group[]>([]);
  let sorting = $state(false);
  let opened = $state<Bucket | null>(null);
  let dialogOpen = $state(false);

  const target = $derived(settings ? aiTarget(settings, PLUGIN_ID) : null);
  const categories = $derived(settings ? categoryList(settings) : []);
  const sorted = $derived(settings?.sorted[accountId]);
  const list = $derived(buckets(senders, sorted, categories));
  const unsorted = $derived(sorted ? senders.filter((sender) => !sorted.senders[sender.key]).length : 0);

  $effect(() => {
    const id = accountId;
    void session.dataVersion;
    let cancelled = false;
    Promise.all([loadPluginConfig<Partial<CategorizerSettings>>(PLUGIN_ID), peopleSenders(id)])
      .then(([config, found]) => {
        if (cancelled) return;
        settings = withCategorizerDefaults(config.settings);
        senders = found;
      })
      .catch((error) => !cancelled && toasts.show(errorMessage(error), 'error'));
    return () => (cancelled = true);
  });

  async function sort() {
    if (!target || !settings) return;
    sorting = true;
    try {
      const result = await sortPeople(accountId, target, categories);
      settings.sorted[accountId] = result;
      senders = await peopleSenders(accountId);
    } catch (error) {
      toasts.show(`Sorting failed. ${errorMessage(error)}`, 'error', 10000);
    } finally {
      sorting = false;
    }
  }

  function show(bucket: Bucket) {
    opened = bucket;
    dialogOpen = true;
  }
</script>

{#if settings}
  <section class="border-b border-line py-12" aria-label="People, sorted">
    <div class="mb-8 flex flex-wrap items-center gap-6">
      <h2 class="m-0 text-lg font-semibold">People, sorted</h2>
      <span class="min-w-100 flex-1 text-sm text-ink2">
        {#if sorted}Sorted {formatRelative(Math.round(sorted.at / 1000))}{#if unsorted > 0} · {formatNumber(unsorted)} new {unsorted === 1 ? 'sender' : 'senders'} not sorted yet{/if}{:else}Your model splits the People category into your own groups{/if}
      </span>
      {#if target}
        <Button size="md" busy={sorting} disabled={senders.length === 0} onclick={sort}>{sorted ? 'Sort again' : 'Sort with AI'}</Button>
      {:else}
        <Button size="md" onclick={() => router.go('plugins')}>Choose a model</Button>
      {/if}
    </div>
    {#if list.length > 0}
      <div class="grid grid-cols-[repeat(auto-fill,minmax(--spacing(90),1fr))] gap-6">
        {#each list as bucket (bucket.category)}
          <button type="button" class="flex flex-col gap-2 rounded-2xl border border-line bg-surface p-8 text-left hover:bg-surface2" onclick={() => show(bucket)}>
            <span class="font-semibold">{bucket.category}</span>
            <span class="text-xs text-ink3">{formatNumber(bucket.senders.length)} {bucket.senders.length === 1 ? 'sender' : 'senders'} · {formatNumber(bucket.messages)} messages</span>
          </button>
        {/each}
      </div>
    {:else if !sorted}
      <p class="m-0 text-pretty text-sm text-ink2">
        Sorting sends the names, addresses, counts and a few subjects of your top {MAX_SENDERS} People senders to the model you chose. Message bodies are never sent.
      </p>
    {:else}
      <p class="m-0 text-sm text-ink2">No People senders to sort.</p>
    {/if}
  </section>
  <BucketDialog bind:open={dialogOpen} {accountId} bucket={opened} />
{/if}
