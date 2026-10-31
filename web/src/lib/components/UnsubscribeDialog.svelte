<script lang="ts">
  import { api, errorMessage } from '../api';
  import { formatNumber } from '../format';
  import { toasts } from '../toasts.svelte';
  import type { ActionResponse, Group, UnsubscribeOutcome } from '../types';
  import Button from './Button.svelte';
  import Modal from './Modal.svelte';
  import Pill, { type PillTone } from './Pill.svelte';

  let {
    open = $bindable(false),
    accountId,
    senders,
    who,
    count,
    ondone,
  }: { open?: boolean; accountId: number; senders: Group[]; who: string; count: number; ondone?: () => void } = $props();

  type FollowUp = 'archive' | 'trash' | 'keep';
  type Result = { name: string; label: string; note: string; tone: PillTone; url: string | null };

  const OPTIONS: { value: FollowUp; label: string; detail: string }[] = [
    { value: 'archive', label: 'Archive them', detail: 'Out of the inbox, still searchable' },
    { value: 'trash', label: 'Move to trash', detail: 'Recoverable from Trash' },
    { value: 'keep', label: 'Keep them', detail: 'Only stop new mail' },
  ];

  let followUp = $state<FollowUp>('archive');
  let running = $state(false);
  let results = $state<Result[] | null>(null);

  $effect(() => {
    if (open) results = null;
  });

  function describe(name: string, outcome: UnsubscribeOutcome): Result {
    if (outcome.status === 'completed') {
      const note = outcome.method === 'one_click' ? 'Done with one click' : 'Unsubscribe email sent';
      return { name, label: 'Unsubscribed', note, tone: 'good', url: null };
    }
    if (outcome.status === 'open_link') {
      return { name, label: 'Needs you', note: "Finish on the sender's site", tone: 'warn', url: outcome.url };
    }
    return { name, label: 'Failed', note: outcome.error, tone: 'bad', url: null };
  }

  async function run() {
    running = true;
    const collected: Result[] = [];
    let queued = 0;
    for (const sender of senders) {
      const name = sender.name ?? sender.key;
      if (!sender.unsub_http && !sender.unsub_mailto) {
        collected.push({ name, label: 'Skipped', note: 'This sender offers no way to unsubscribe', tone: 'muted', url: null });
        continue;
      }
      try {
        const response = await api.post<ActionResponse & { outcome: UnsubscribeOutcome }>(`/accounts/${accountId}/unsubscribe`, {
          from_addr: sender.key,
          then: followUp === 'keep' ? null : followUp,
        });
        if (response.job_id !== null) queued += 1;
        const result = describe(name, response.outcome);
        if (response.skipped_pinned > 0) result.note = `${result.note}. Pinned, so its messages stay where they are`;
        collected.push(result);
      } catch (error) {
        collected.push({ name, label: 'Failed', note: errorMessage(error), tone: 'bad', url: null });
      }
    }
    if (queued > 0) toasts.show(`${followUp === 'archive' ? 'Archiving' : 'Moving to trash'} messages from ${formatNumber(queued)} sender${queued === 1 ? '' : 's'}.`, 'success');
    results = collected;
    running = false;
    ondone?.();
  }
</script>

<Modal bind:open title={results ? 'Unsubscribe results' : `Unsubscribe from ${who}`}>
  {#if results}
    <div class="flex flex-col">
      {#each results as result (result.name)}
        <div class="box-content flex min-h-26 items-center gap-6 border-b border-line">
          <span class="flex min-w-0 flex-1 flex-col leading-row">
            <span class="font-semibold">{result.name}</span>
            <span class="text-xs text-ink2">{result.note}</span>
          </span>
          <Pill tone={result.tone}>{result.label}</Pill>
          {#if result.url}<a href={result.url} target="_blank" rel="noopener noreferrer" class="text-sm font-semibold">Open</a>{/if}
        </div>
      {/each}
    </div>
    <div class="text-sm text-ink2">Links you haven't opened stay in Activity until you mark them done.</div>
  {:else}
    <div class="leading-normal text-ink2">We try one-click first, then email the sender, then give you their link. What should happen to the {formatNumber(count)} messages they already sent?</div>
    <div class="flex flex-col gap-4" role="radiogroup" aria-label="Existing messages">
      {#each OPTIONS as option (option.value)}
        {@const selected = followUp === option.value}
        <button
          type="button"
          role="radio"
          aria-checked={selected}
          class="flex items-center gap-6 rounded-xl border-check bg-transparent p-7 text-left {selected ? 'border-ink' : 'border-line'}"
          onclick={() => (followUp = option.value)}>
          <span class="grid size-9 shrink-0 place-items-center rounded-full border-check {selected ? 'border-ink' : 'border-line'}">
            <span class="size-4 rounded-full bg-ink {selected ? 'opacity-100' : 'opacity-0'}"></span>
          </span>
          <span class="flex flex-col leading-row">
            <span class="font-semibold">{option.label}</span>
            <span class="text-xs text-ink2">{option.detail}</span>
          </span>
        </button>
      {/each}
    </div>
  {/if}
  {#snippet footer()}
    {#if results}
      <Button size="dialog" variant="primary" onclick={() => (open = false)}>Done</Button>
    {:else}
      <Button size="dialog" onclick={() => (open = false)}>Cancel</Button>
      <Button size="dialog" variant="primary" busy={running} onclick={run}>Unsubscribe</Button>
    {/if}
  {/snippet}
</Modal>
