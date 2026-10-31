<script lang="ts">
  import { errorMessage } from '../../lib/api';
  import Avatar from '../../lib/components/Avatar.svelte';
  import { formatBytes, formatNumber } from '../../lib/format';
  import { router } from '../../lib/router.svelte';
  import { session } from '../../lib/session.svelte';
  import { CATEGORY_LABELS, type Digest, loadDigest, weeklyCleanup } from './digest';

  let { accountId }: { accountId: number } = $props();

  let digest = $state<Digest | null>(null);
  let error = $state<string | null>(null);

  const cleanup = $derived(weeklyCleanup(session.jobs, accountId));
  const pct = (value: number) => Math.round(value * 100);

  $effect(() => {
    const id = accountId;
    void session.dataVersion;
    let cancelled = false;
    loadDigest(id)
      .then((result) => {
        if (cancelled) return;
        digest = result;
        error = null;
      })
      .catch((reason) => !cancelled && (error = errorMessage(reason)));
    return () => (cancelled = true);
  });
</script>

<section class="border-b border-line py-12" aria-label="This week">
  <div class="mb-8 flex flex-wrap items-baseline gap-6">
    <h2 class="m-0 text-lg font-semibold">This week</h2>
    <span class="text-sm text-ink2">The last 7 days of your mailbox</span>
  </div>
  {#if error}
    <p class="m-0 text-sm text-bad">{error}</p>
  {:else if digest}
    <div class="grid grid-cols-[repeat(auto-fit,minmax(--spacing(100),1fr))] gap-x-12 gap-y-10">
      <div class="flex flex-col gap-3">
        <span class="text-sm text-ink2">Arrived</span>
        <span class="text-2xl font-semibold tracking-tight">{formatNumber(digest.arrived.count)}</span>
        <span class="text-xs text-ink3">{formatNumber(digest.unread)} still unread · {formatBytes(digest.arrived.bytes)}</span>
      </div>
      <div class="flex flex-col gap-3">
        <span class="text-sm text-ink2">Cleaned up</span>
        <span class="text-2xl font-semibold tracking-tight">{formatNumber(cleanup.messages)}</span>
        <span class="text-xs text-ink3">{formatNumber(cleanup.actions)} {cleanup.actions === 1 ? 'action' : 'actions'} · {formatBytes(digest.freedBytes)} freed</span>
      </div>
      <div class="flex flex-col gap-3">
        <span class="text-sm text-ink2">Mix shift</span>
        {#if digest.shift && digest.shift.week > digest.shift.overall}
          <span class="text-2xl font-semibold tracking-tight">{CATEGORY_LABELS[digest.shift.category]}</span>
          <span class="text-xs text-ink3">{pct(digest.shift.week)}% this week · {pct(digest.shift.overall)}% overall</span>
        {:else}
          <span class="text-2xl font-semibold tracking-tight">Steady</span>
          <span class="text-xs text-ink3">No category grew this week</span>
        {/if}
      </div>
      <div class="flex min-w-0 flex-col gap-3">
        <span class="text-sm text-ink2">Busiest senders</span>
        {#each digest.senders as sender (sender.key)}
          <button
            type="button"
            class="flex min-w-0 items-center gap-4 border-0 bg-transparent p-0 text-left text-sm"
            onclick={() => router.go('senders', { select: sender.key })}>
            <Avatar label={sender.name ?? sender.key} seed={sender.key} size="sm" />
            <span class="min-w-0 flex-1 truncate font-semibold">{sender.name ?? sender.key}</span>
            <span class="text-ink2 tabular-nums">{formatNumber(sender.message_count)}</span>
          </button>
        {:else}
          <span class="text-xs text-ink3">Nothing arrived this week</span>
        {/each}
      </div>
    </div>
  {/if}
</section>
