<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    active,
    tall = false,
    muted = false,
    surface = false,
    onclick,
    children,
  }: { active: boolean; tall?: boolean; muted?: boolean; surface?: boolean; onclick: () => void; children: Snippet } = $props();

  const idle = $derived(`${surface ? 'bg-surface' : 'bg-transparent'} ${muted ? 'text-ink2' : 'text-ink'} border-line`);
</script>

<button
  type="button"
  aria-pressed={active}
  class="shrink-0 whitespace-nowrap rounded-full border px-7 text-sm font-semibold {tall ? 'h-18' : 'h-17'} {active ? `bg-btn text-btn-ink ${surface ? 'border-btn' : 'border-line'}` : idle}"
  {onclick}>{@render children()}</button>
