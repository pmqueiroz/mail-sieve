<script lang="ts">
  import type { Snippet } from 'svelte';

  let { label, hint, children }: { label: string; hint?: string; children: Snippet } = $props();
</script>

<label class="field">
  <span class="label">{label}</span>
  {@render children()}
  {#if hint}<span class="hint">{hint}</span>{/if}
</label>

<style>
  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 0;
  }
  .label {
    font-size: var(--text-sm);
    font-weight: 600;
  }
  .hint {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }
</style>
