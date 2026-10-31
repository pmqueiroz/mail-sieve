<script lang="ts" generics="P extends ContentContext">
  import { untrack } from 'svelte';
  import { host } from './host.svelte';
  import { plugins } from './registry.svelte';
  import type { ContentContext, MountFn } from './types';

  let { id, mount, context }: { id: string; mount: MountFn<P>; context: P } = $props();

  let target = $state<HTMLElement | null>(null);

  const live = new Proxy({} as P, { get: (_, key) => Reflect.get(context, key) });

  $effect(() => {
    const element = target;
    if (!element) return;
    return untrack(() => {
      try {
        return mount(element, host, live) ?? undefined;
      } catch (error) {
        plugins.fail(id, error);
      }
    });
  });
</script>

<div class="contents" bind:this={target}></div>
