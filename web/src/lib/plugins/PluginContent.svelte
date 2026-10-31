<script lang="ts" generics="P extends ContentContext">
  import MountSlot from './MountSlot.svelte';
  import { plugins } from './registry.svelte';
  import type { Content, ContentContext } from './types';

  let { id, content, context }: { id: string; content: Content<P>; context: P } = $props();
</script>

<svelte:boundary onerror={(error) => plugins.fail(id, error)}>
  {#if typeof content === 'function'}
    {@const Body = content}
    <Body {...context} />
  {:else}
    <MountSlot {id} mount={content.mount} {context} />
  {/if}
</svelte:boundary>
