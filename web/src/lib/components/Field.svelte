<script lang="ts">
  import type { Snippet } from 'svelte';

  let { label, hint, children }: { label: string; hint?: Snippet | string; children: Snippet } = $props();
</script>

<label class="flex min-w-0 flex-col gap-3 text-sm font-semibold">
  {label}
  {@render children()}
  {#if typeof hint === 'string'}
    <span class="font-normal text-ink2">{hint}</span>
  {:else if hint}
    <span class="font-normal text-ink2">{@render hint()}</span>
  {/if}
</label>
