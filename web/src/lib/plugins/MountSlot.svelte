<script lang="ts">
  import { untrack } from 'svelte';
  import { host } from './host.svelte';
  import { plugins } from './registry.svelte';
  import type { ContentContext, MountFn } from './types';

  let { id, mount, context }: { id: string; mount: MountFn; context: ContentContext } = $props();

  let target = $state<HTMLElement | null>(null);

  $effect(() => {
    const element = target;
    if (!element) return;
    return untrack(() => {
      try {
        return mount(element, host, context) ?? undefined;
      } catch (error) {
        plugins.fail(id, error);
      }
    });
  });
</script>

<div class="contents" bind:this={target}></div>
