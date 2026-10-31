<script lang="ts">
  import { resolveAvatar, type PluginAvatarSpec } from './avatar';

  let { avatar, id, name }: { avatar: PluginAvatarSpec | null; id: string; name: string } = $props();

  const resolved = $derived(resolveAvatar(avatar, id, name));
</script>

<span class="plugin-solid grid size-20 shrink-0 place-items-center rounded-lg" style:--plugin-hue={resolved.hue} style:--plugin-chroma={resolved.chroma} aria-hidden="true">
  {#if resolved.paths}
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round">
      {#each resolved.paths as d, index (index)}<path {d} />{/each}
    </svg>
  {:else}
    <span class="text-base font-bold tracking-tight">{resolved.monogram}</span>
  {/if}
</span>
