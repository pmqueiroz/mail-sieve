<script lang="ts">
  import { api, errorMessage, withParams } from '../api';
  import { formatBytes, formatNumber } from '../format';
  import { QUERY_HELP } from '../query';
  import { session } from '../session.svelte';
  import type { Totals } from '../types';

  let {
    value = $bindable(''),
    accountId,
    placeholder = 'Filter: from:*@shop.com older_than:1y is:unread',
    showCount = true,
    totals = $bindable<Totals | null>(null),
  }: {
    value?: string;
    accountId: number;
    placeholder?: string;
    showCount?: boolean;
    totals?: Totals | null;
  } = $props();

  let draft = $state('');
  let error = $state<string | null>(null);
  let help = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    draft = value;
  });

  function schedule() {
    clearTimeout(timer);
    timer = setTimeout(() => (value = draft.trim()), 250);
  }

  $effect(() => {
    const query = value;
    const id = accountId;
    void session.dataVersion;
    let cancelled = false;
    api
      .get<Totals>(withParams(`/accounts/${id}/count`, { q: query }))
      .then((result) => {
        if (cancelled) return;
        totals = result;
        error = null;
      })
      .catch((reason) => {
        if (cancelled) return;
        totals = null;
        error = errorMessage(reason);
      });
    return () => (cancelled = true);
  });
</script>

<div class="query">
  <div class="input-row">
    <input
      type="search"
      spellcheck="false"
      autocomplete="off"
      aria-label="Query"
      class:invalid={error}
      {placeholder}
      bind:value={draft}
      oninput={schedule}
      onkeydown={(event) => event.key === 'Enter' && (clearTimeout(timer), (value = draft.trim()))} />
    <button class="help-toggle" aria-expanded={help} onclick={() => (help = !help)} title="Query syntax">?</button>
  </div>
  {#if showCount}
    <div class="status" aria-live="polite">
      {#if error}
        <span class="error">{error}</span>
      {:else if totals}
        <span><strong>{formatNumber(totals.count)}</strong> messages · {formatBytes(totals.bytes)}</span>
      {/if}
    </div>
  {/if}
  {#if help}
    <div class="help card">
      {#each QUERY_HELP as [example, meaning] (example)}
        <button class="example mono" onclick={() => ((draft = `${draft} ${example.split('  ')[0]}`.trim()), schedule())}>{example}</button>
        <span class="muted">{meaning}</span>
      {/each}
    </div>
  {/if}
</div>

<style>
  .query {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    flex: 1;
    min-width: 280px;
    position: relative;
  }
  .input-row {
    display: flex;
    gap: var(--space-2);
  }
  .input-row input {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
  }
  .invalid {
    border-color: var(--color-danger) !important;
  }
  .help-toggle {
    width: 34px;
    flex-shrink: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    cursor: pointer;
    font-weight: 700;
  }
  .status {
    font-size: var(--text-sm);
    min-height: 1.2em;
    color: var(--color-text-muted);
  }
  .error {
    color: var(--color-danger);
  }
  .help {
    position: absolute;
    top: 44px;
    left: 0;
    right: 0;
    z-index: 20;
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: var(--space-1) var(--space-4);
    padding: var(--space-3) var(--space-4);
    box-shadow: var(--shadow-overlay);
  }
  .example {
    text-align: left;
    border: none;
    background: none;
    cursor: pointer;
    color: var(--color-accent);
    padding: 0;
  }
</style>
