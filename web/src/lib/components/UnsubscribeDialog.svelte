<script lang="ts">
  import { api, errorMessage } from '../api';
  import type { Group, UnsubscribeOutcome } from '../types';
  import Badge from './Badge.svelte';
  import Button from './Button.svelte';
  import Modal from './Modal.svelte';

  let {
    open = $bindable(false),
    accountId,
    senders,
    ondone,
  }: { open?: boolean; accountId: number; senders: Group[]; ondone?: () => void } = $props();

  type Result = { sender: string; outcome: UnsubscribeOutcome };

  let followUp = $state<'' | 'archive' | 'trash'>('archive');
  let running = $state(false);
  let results = $state<Result[]>([]);

  const eligible = $derived(senders.filter((sender) => sender.unsub_http || sender.unsub_mailto));
  const skipped = $derived(senders.length - eligible.length);

  $effect(() => {
    if (open) results = [];
  });

  async function run() {
    running = true;
    results = [];
    for (const sender of eligible) {
      try {
        const response = await api.post<{ outcome: UnsubscribeOutcome }>(`/accounts/${accountId}/unsubscribe`, {
          from_addr: sender.key,
          then: followUp || null,
        });
        results.push({ sender: sender.key, outcome: response.outcome });
      } catch (error) {
        results.push({ sender: sender.key, outcome: { status: 'failed', error: errorMessage(error) } });
      }
    }
    running = false;
    ondone?.();
  }
</script>

<Modal bind:open title="Unsubscribe" width={600}>
  {#if results.length === 0}
    <p>
      Unsubscribe from <strong>{eligible.length}</strong> sender{eligible.length === 1 ? '' : 's'}.
      One-click (RFC 8058) requests are sent by the server; mailto requests are sent through the account's SMTP
      server; other senders give you a link to open.
    </p>
    {#if skipped > 0}<p class="muted">{skipped} selected sender{skipped === 1 ? '' : 's'} offer no unsubscribe option.</p>{/if}
    <fieldset>
      <legend>Afterwards</legend>
      <label><input type="radio" bind:group={followUp} value="archive" /> Archive their existing messages</label>
      <label><input type="radio" bind:group={followUp} value="trash" /> Move their existing messages to trash</label>
      <label><input type="radio" bind:group={followUp} value="" /> Keep existing messages</label>
    </fieldset>
  {:else}
    <ul class="results">
      {#each results as result (result.sender)}
        <li>
          <span class="truncate">{result.sender}</span>
          {#if result.outcome.status === 'completed'}
            <Badge tone="success" title={result.outcome.detail}>unsubscribed</Badge>
          {:else if result.outcome.status === 'open_link'}
            <a href={result.outcome.url} target="_blank" rel="noopener noreferrer">Open unsubscribe page</a>
          {:else}
            <Badge tone="danger" title={result.outcome.error}>failed</Badge>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
  {#snippet footer()}
    {#if results.length === 0}
      <Button variant="ghost" onclick={() => (open = false)}>Cancel</Button>
      <Button variant="primary" busy={running} disabled={eligible.length === 0} onclick={run}>Unsubscribe</Button>
    {:else}
      <Button variant="primary" onclick={() => (open = false)}>Close</Button>
    {/if}
  {/snippet}
</Modal>

<style>
  p {
    margin: 0;
    line-height: 1.5;
  }
  fieldset {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3);
  }
  legend {
    font-weight: 600;
    padding: 0 var(--space-1);
  }
  label {
    display: flex;
    gap: var(--space-2);
    align-items: center;
  }
  .results {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    max-height: 50vh;
    overflow: auto;
  }
  .results li {
    display: flex;
    justify-content: space-between;
    gap: var(--space-3);
    align-items: center;
  }
</style>
