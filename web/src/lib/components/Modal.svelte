<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    open = $bindable(false),
    title,
    wide = false,
    children,
    footer,
  }: { open?: boolean; title: string; wide?: boolean; children: Snippet; footer?: Snippet } = $props();

  let dialog: HTMLDialogElement | undefined = $state();

  $effect(() => {
    if (!dialog) return;
    if (open && !dialog.open) dialog.showModal();
    if (!open && dialog.open) dialog.close();
  });
</script>

<dialog
  bind:this={dialog}
  onclose={() => (open = false)}
  onclick={(event) => event.target === dialog && (open = false)}
  class="m-auto max-h-[90vh] w-[min(var(--modal-width),calc(100vw-var(--spacing)*16))] overflow-auto rounded-4xl border-0 bg-surface p-12 text-ink shadow-pop backdrop:bg-overlay {wide ? '[--modal-width:calc(var(--spacing)*300)]' : '[--modal-width:calc(var(--spacing)*240)]'}">
  {#if open}
    <div class="flex flex-col gap-8">
      <h2 class="m-0 text-xl font-semibold">{title}</h2>
      {@render children()}
      {#if footer}<div class="flex flex-wrap justify-end gap-4">{@render footer()}</div>{/if}
    </div>
  {/if}
</dialog>
