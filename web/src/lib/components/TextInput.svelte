<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';

  let { value = $bindable(), mono = false, ...rest }: HTMLInputAttributes & { mono?: boolean } = $props();
</script>

<input
  class="box-content h-21 min-w-0 rounded-lg border border-line bg-transparent px-6 font-normal outline-0 focus:border-ink3 {mono ? 'font-mono text-sm' : 'text-base'}"
  bind:value
  {...rest} />
