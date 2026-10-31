<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    open = $bindable(false),
    class: className = '',
    children,
  }: { open?: boolean; class?: string; children: Snippet } = $props();

  let panel: HTMLDivElement | undefined = $state();

  $effect(() => {
    if (!open) return;
    const close = (event: PointerEvent) => {
      const anchor = panel?.parentElement;
      if (anchor && !anchor.contains(event.target as Node)) open = false;
    };
    const escape = (event: KeyboardEvent) => event.key === 'Escape' && (open = false);
    window.addEventListener('pointerdown', close);
    window.addEventListener('keydown', escape);
    return () => {
      window.removeEventListener('pointerdown', close);
      window.removeEventListener('keydown', escape);
    };
  });
</script>

{#if open}
  <div bind:this={panel} class="absolute z-30 rounded-2xl border border-line bg-surface p-3 shadow-pop {className}">
    {@render children()}
  </div>
{/if}
