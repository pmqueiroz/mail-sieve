<script lang="ts" generics="K extends SlotName">
  import PluginContent from './PluginContent.svelte';
  import { plugins } from './registry.svelte';
  import type { SlotContexts, SlotName, SlotProps } from './types';

  let { name, context }: { name: K; context: SlotContexts[K] } = $props();

  const contents = $derived(plugins.slot(name));
  const slotContext = $derived({ close: () => {}, ...context } as SlotProps<K>);
</script>

{#each contents as entry (entry.id)}
  <PluginContent id={entry.id} content={entry.content} context={slotContext} />
{/each}
