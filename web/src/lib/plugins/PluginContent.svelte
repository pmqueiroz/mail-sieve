<script lang="ts">
  import MountSlot from './MountSlot.svelte';
  import { plugins } from './registry.svelte';
  import type { ContentContext, SlotContent } from './types';

  let { id, content, context = { close: () => {} } }: { id: string; content: SlotContent; context?: ContentContext } = $props();
</script>

<svelte:boundary onerror={(error) => plugins.fail(id, error)}>
  {#if typeof content === 'function'}
    {@const Content = content}
    <Content {...context} />
  {:else}
    <MountSlot {id} mount={content.mount} {context} />
  {/if}
</svelte:boundary>
