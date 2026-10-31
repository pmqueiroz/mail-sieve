<script lang="ts" generics="T">
  import { type Snippet, untrack } from 'svelte';

  let {
    items,
    key,
    row,
    rowHeight = 44,
    overscan = 10,
    resetKey = '',
    onEndReached,
  }: {
    items: T[];
    key: (item: T) => string | number;
    row: Snippet<[T, number]>;
    rowHeight?: number;
    overscan?: number;
    resetKey?: string;
    onEndReached?: () => void;
  } = $props();

  let viewport: HTMLDivElement | undefined = $state();
  let scrollTop = $state(0);
  let height = $state(0);

  const start = $derived(Math.max(0, Math.floor(scrollTop / rowHeight) - overscan));
  const end = $derived(Math.min(items.length, Math.ceil((scrollTop + height) / rowHeight) + overscan));
  const visible = $derived(items.slice(start, end));

  $effect(() => {
    void resetKey;
    untrack(() => {
      if (viewport) viewport.scrollTop = 0;
      scrollTop = 0;
    });
  });

  $effect(() => {
    if (onEndReached && items.length > 0 && end >= items.length - overscan) untrack(() => onEndReached());
  });
</script>

<div class="viewport" bind:this={viewport} bind:clientHeight={height} onscroll={() => (scrollTop = viewport?.scrollTop ?? 0)}>
  <div class="spacer" style:height="{items.length * rowHeight}px">
    <div class="window" style:transform="translateY({start * rowHeight}px)">
      {#each visible as item, index (key(item))}
        <div class="row" style:height="{rowHeight}px">{@render row(item, start + index)}</div>
      {/each}
    </div>
  </div>
</div>

<style>
  .viewport {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    contain: strict;
  }
  .spacer {
    position: relative;
  }
  .window {
    position: absolute;
    inset: 0 0 auto 0;
    will-change: transform;
  }
  .row {
    display: flex;
    align-items: stretch;
  }
</style>
