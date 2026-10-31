<script lang="ts">
  import type { Rule } from '../../lib/types';
  import RecipeCard from './RecipeCard.svelte';
  import { available } from './recipes';

  let { accountId, rules, reload }: { accountId: number; rules: Rule[]; reload: () => void } = $props();

  const recipes = $derived(available(rules));
</script>

{#if recipes.length > 0}
  <section class="mb-12 flex flex-col gap-6" aria-label="Starter rules">
    <div class="flex flex-wrap items-baseline gap-6">
      <h2 class="m-0 text-lg font-semibold">Starter rules</h2>
      <span class="text-sm text-ink2">One click adds a rule that runs after every sync. Pinned senders are always skipped.</span>
    </div>
    <div class="grid grid-cols-[repeat(auto-fill,minmax(--spacing(130),1fr))] gap-6">
      {#each recipes as recipe (recipe.name)}
        <RecipeCard {accountId} {recipe} onadded={reload} />
      {/each}
    </div>
  </section>
{/if}
