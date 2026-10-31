<script lang="ts">
  import { api, errorMessage } from '../../lib/api';
  import Button from '../../lib/components/Button.svelte';
  import { liveTotals } from '../../lib/count.svelte';
  import { describeRule } from '../../lib/describe';
  import { formatNumber } from '../../lib/format';
  import { skippedNote } from '../../lib/pins';
  import { toasts } from '../../lib/toasts.svelte';
  import type { Recipe } from './recipes';

  let { accountId, recipe, onadded }: { accountId: number; recipe: Recipe; onadded: () => void } = $props();

  let adding = $state(false);

  const live = liveTotals(
    () => accountId,
    () => recipe.query,
  );

  async function add() {
    adding = true;
    try {
      await api.post('/rules', { account_id: accountId, name: recipe.name, query: recipe.query, action: recipe.action, enabled: true, run_after_sync: true });
      toasts.show(`Rule ${recipe.name} added. It runs after every sync.`, 'success');
      onadded();
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    } finally {
      adding = false;
    }
  }
</script>

<article class="flex flex-col gap-5 rounded-3xl border border-dashed border-line p-9">
  <span class="text-sm font-semibold">{recipe.name}</span>
  <span class="flex-1 text-pretty leading-row text-ink2">{describeRule({ ...recipe, run_after_sync: true, interval_minutes: null })}</span>
  <span class="text-xs text-ink3">
    {#if live.error}{live.error}{:else}Matches {formatNumber(live.totals?.count ?? 0)} messages now{#if live.totals?.pinned}, {skippedNote(live.totals.pinned)}{/if}{/if}
  </span>
  <Button class="self-start" busy={adding} onclick={add}>Add rule</Button>
</article>
