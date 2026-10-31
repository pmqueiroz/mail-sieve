<script lang="ts">
  import type { Snippet } from 'svelte';
  import { formatBytes, formatNumber } from '../format';

  let {
    count,
    bytes,
    onclear,
    children,
  }: { count: number; bytes?: number; onclear: () => void; children: Snippet } = $props();
</script>

{#if count > 0}
  <div class="action-bar" role="toolbar" aria-label="Bulk actions">
    <span class="summary">
      <strong>{formatNumber(count)}</strong> selected{#if bytes !== undefined}&nbsp;· {formatBytes(bytes)}{/if}
    </span>
    <div class="actions">{@render children()}</div>
    <button class="clear" onclick={onclear}>Clear</button>
  </div>
{/if}

<style>
  .action-bar {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    background: var(--color-accent-soft);
    border: 1px solid var(--color-accent);
    flex-wrap: wrap;
  }
  .summary {
    white-space: nowrap;
  }
  .actions {
    display: flex;
    gap: var(--space-2);
    flex-wrap: wrap;
    flex: 1;
  }
  .clear {
    border: none;
    background: none;
    color: var(--color-accent);
    cursor: pointer;
    font-weight: 600;
  }
</style>
