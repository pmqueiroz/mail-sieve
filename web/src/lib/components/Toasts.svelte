<script lang="ts">
  import { toasts } from '../toasts.svelte';
</script>

<div class="toasts" aria-live="polite">
  {#each toasts.items as toast (toast.id)}
    <div class="toast {toast.kind}" role="status">
      <span>{toast.message}</span>
      <button aria-label="Dismiss" onclick={() => toasts.dismiss(toast.id)}>×</button>
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    right: var(--space-4);
    bottom: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    z-index: 100;
    max-width: min(420px, calc(100vw - 32px));
  }
  .toast {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    box-shadow: var(--shadow-overlay);
    line-height: 1.4;
  }
  .toast span {
    flex: 1;
  }
  .success {
    border-left: 4px solid var(--color-success);
  }
  .error {
    border-left: 4px solid var(--color-danger);
  }
  .info {
    border-left: 4px solid var(--color-accent);
  }
  button {
    border: none;
    background: none;
    cursor: pointer;
    font-size: 18px;
    line-height: 1;
    color: var(--color-text-muted);
  }
</style>
