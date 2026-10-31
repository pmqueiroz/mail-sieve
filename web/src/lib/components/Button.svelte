<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  type Variant = 'primary' | 'secondary' | 'ghost' | 'danger';

  let {
    variant = 'secondary',
    size = 'md',
    busy = false,
    disabled = false,
    children,
    ...rest
  }: HTMLButtonAttributes & { variant?: Variant; size?: 'sm' | 'md'; busy?: boolean; children: Snippet } = $props();
</script>

<button class="button {variant} {size}" disabled={disabled || busy} aria-busy={busy} {...rest}>
  {#if busy}<span class="spinner" aria-hidden="true"></span>{/if}
  {@render children()}
</button>

<style>
  .button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    padding: var(--space-2) var(--space-3);
    font-weight: 550;
    line-height: 1.2;
    cursor: pointer;
    white-space: nowrap;
    transition: background 120ms, border-color 120ms;
  }
  .button.sm {
    padding: var(--space-1) var(--space-2);
    font-size: var(--text-sm);
  }
  .button:hover:not(:disabled) {
    background: var(--color-surface-muted);
  }
  .button:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
  .button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }
  .primary {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: var(--color-accent-text);
  }
  .primary:hover:not(:disabled) {
    background: var(--color-accent);
    filter: brightness(1.08);
  }
  .ghost {
    border-color: transparent;
    background: transparent;
  }
  .danger {
    color: var(--color-danger);
    border-color: var(--color-danger-soft);
  }
  .danger:hover:not(:disabled) {
    background: var(--color-danger-soft);
  }
  .spinner {
    width: 12px;
    height: 12px;
    border: 2px solid currentColor;
    border-right-color: transparent;
    border-radius: 50%;
    animation: spin 700ms linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
