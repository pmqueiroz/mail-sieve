<script lang="ts">
  import Brand from './Brand.svelte';

  let { text }: { text: string } = $props();
  const parts = $derived(text.split('mailsieve'));
</script>

{#each parts as part, index (index)}{#if index > 0}<Brand />{/if}{part}{/each}
