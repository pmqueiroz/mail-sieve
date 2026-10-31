<script lang="ts">
  import type { Snippet } from 'svelte';

  let { tone = 'neutral', title, children }: { tone?: 'neutral' | 'accent' | 'success' | 'warning' | 'danger'; title?: string; children: Snippet } = $props();
</script>

<span class="badge {tone}" {title}>{@render children()}</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 1px 8px;
    border-radius: 999px;
    font-size: var(--text-xs);
    font-weight: 600;
    background: var(--color-surface-muted);
    color: var(--color-text-muted);
    white-space: nowrap;
  }
  .accent {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }
  .success {
    background: var(--color-success-soft);
    color: var(--color-success);
  }
  .warning {
    background: var(--color-warning-soft);
    color: var(--color-warning);
  }
  .danger {
    background: var(--color-danger-soft);
    color: var(--color-danger);
  }
</style>
