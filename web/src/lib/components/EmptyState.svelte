<script lang="ts">
  import type { Snippet } from 'svelte';
  import BrandText from './BrandText.svelte';

  let { title, children }: { title?: string; children?: Snippet } = $props();
</script>

<div class="flex flex-col items-center gap-6 py-24 text-center text-ink2">
  {#if title}<h2 class="m-0 text-lg font-semibold text-ink"><BrandText text={title} /></h2>{/if}
  {#if children}<div class="flex max-w-220 flex-col items-center gap-6 text-pretty">{@render children()}</div>{/if}
</div>
