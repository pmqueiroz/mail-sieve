<script lang="ts" generics="T">
  import { type Snippet, untrack } from 'svelte';

  let {
    items,
    key,
    row,
    rowHeight,
    overscan = 10,
    resetKey = '',
    onEndReached,
  }: {
    items: T[];
    key: (item: T) => string | number;
    row: Snippet<[T, number]>;
    rowHeight: number;
    overscan?: number;
    resetKey?: string;
    onEndReached?: () => void;
  } = $props();

  let container: HTMLDivElement | undefined = $state();
  let offset = $state(0);
  let viewport = $state(window.innerHeight);

  const start = $derived(Math.max(0, Math.floor(-offset / rowHeight) - overscan));
  const end = $derived(Math.min(items.length, Math.ceil((viewport - offset) / rowHeight) + overscan));
  const visible = $derived(items.slice(start, end));

  function measure() {
    if (!container) return;
    offset = container.getBoundingClientRect().top;
    viewport = window.innerHeight;
  }

  $effect(() => {
    let frame = 0;
    const schedule = () => {
      if (frame) return;
      frame = requestAnimationFrame(() => {
        frame = 0;
        measure();
      });
    };
    measure();
    window.addEventListener('scroll', schedule, { passive: true });
    window.addEventListener('resize', schedule, { passive: true });
    return () => {
      cancelAnimationFrame(frame);
      window.removeEventListener('scroll', schedule);
      window.removeEventListener('resize', schedule);
    };
  });

  $effect(() => {
    void resetKey;
    untrack(() => {
      const top = container?.getBoundingClientRect().top ?? 0;
      if (top < 0) window.scrollTo({ top: window.scrollY + top });
      measure();
    });
  });

  $effect(() => {
    if (onEndReached && items.length > 0 && end >= items.length - overscan) untrack(() => onEndReached());
  });
</script>

<div bind:this={container} class="relative" style:height="{items.length * rowHeight}px">
  <div class="absolute inset-x-0 top-0 will-change-transform" style:transform="translateY({start * rowHeight}px)">
    {#each visible as item, index (key(item))}
      <div style:height="{rowHeight}px">{@render row(item, start + index)}</div>
    {/each}
  </div>
</div>
