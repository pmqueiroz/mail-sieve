<script lang="ts">
  import type { Snippet } from 'svelte';

  let { title, children }: { title: string; children?: Snippet } = $props();
</script>

<div class="empty">
  <h2>{title}</h2>
  {#if children}<div class="body">{@render children()}</div>{/if}
</div>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-6);
    text-align: center;
    color: var(--color-text-muted);
    flex: 1;
  }
  h2 {
    margin: 0;
    font-size: var(--text-lg);
    color: var(--color-text);
  }
  .body {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-3);
    max-width: 440px;
  }
</style>
