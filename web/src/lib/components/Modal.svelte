<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    open = $bindable(false),
    title,
    width = 520,
    children,
    footer,
  }: { open?: boolean; title: string; width?: number; children: Snippet; footer?: Snippet } = $props();

  let dialog: HTMLDialogElement | undefined = $state();

  $effect(() => {
    if (!dialog) return;
    if (open && !dialog.open) dialog.showModal();
    if (!open && dialog.open) dialog.close();
  });
</script>

<dialog bind:this={dialog} onclose={() => (open = false)} style:width="min({width}px, calc(100vw - 32px))">
  <header>
    <h2>{title}</h2>
    <button class="close" aria-label="Close" onclick={() => (open = false)}>×</button>
  </header>
  <div class="content">{@render children()}</div>
  {#if footer}<footer>{@render footer()}</footer>{/if}
</dialog>

<style>
  dialog {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 0;
    background: var(--color-surface);
    color: var(--color-text);
    box-shadow: var(--shadow-overlay);
  }
  dialog::backdrop {
    background: rgba(10, 12, 16, 0.45);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4) var(--space-5) 0;
  }
  h2 {
    margin: 0;
    font-size: var(--text-lg);
  }
  .close {
    border: none;
    background: none;
    font-size: 22px;
    line-height: 1;
    cursor: pointer;
    color: var(--color-text-muted);
  }
  .content {
    padding: var(--space-4) var(--space-5);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }
  footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    padding: 0 var(--space-5) var(--space-4);
  }
</style>
