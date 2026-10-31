<script lang="ts">
  import PluginContent from './PluginContent.svelte';
  import { plugins } from './registry.svelte';
  import type { SlotName } from './types';

  let { name }: { name: SlotName } = $props();

  const contents = $derived(plugins.slot(name));
</script>

{#each contents as entry (entry.id)}
  <PluginContent id={entry.id} content={entry.content} />
{/each}
