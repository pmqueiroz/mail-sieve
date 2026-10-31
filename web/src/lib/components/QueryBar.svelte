<script lang="ts">
  import { type Snippet, untrack } from 'svelte';
  import { liveTotals } from '../count.svelte';
  import { formatBytes, formatNumber } from '../format';
  import { QUERY_HELP } from '../query';
  import type { Totals } from '../types';
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';

  let {
    value = $bindable(''),
    accountId,
    placeholder,
    large = false,
    showCount = true,
    totals = $bindable<Totals | null>(null),
    trailing,
  }: {
    value?: string;
    accountId: number;
    placeholder: string;
    large?: boolean;
    showCount?: boolean;
    totals?: Totals | null;
    trailing?: Snippet;
  } = $props();

  const DEBOUNCE_MS = 250;

  let draft = $state(value);
  let help = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;
  const live = liveTotals(
    () => accountId,
    () => value,
  );

  $effect(() => {
    totals = live.totals;
  });

  $effect(() => {
    const next = value;
    untrack(() => {
      if (next !== draft.trim()) draft = next;
    });
  });

  function commit() {
    clearTimeout(timer);
    value = draft.trim();
  }

  function schedule() {
    clearTimeout(timer);
    timer = setTimeout(commit, DEBOUNCE_MS);
  }

  function add(term: string) {
    draft = `${draft} ${term}`.trim();
    help = false;
    commit();
  }
</script>

{#snippet count()}
  {#if live.error}
    <span class="text-bad">{live.error}</span>
  {:else if live.totals}
    <span><b class="text-ink">{formatNumber(live.totals.count)}</b> messages, {formatBytes(live.totals.bytes)}</span>
  {/if}
{/snippet}

<div class="flex flex-col gap-7">
  <div class="relative box-content flex items-center gap-5 rounded-full border bg-surface {live.error ? 'border-bad' : 'border-line'} {large ? 'h-24 pr-4 pl-8' : 'h-22 px-8'}">
    <Icon name="search" size={16} class="shrink-0 text-ink3" />
    <input
      type="search"
      spellcheck="false"
      autocomplete="off"
      aria-label="Search"
      class="min-w-0 flex-1 border-0 bg-transparent font-mono outline-0 {large ? 'text-base' : 'text-sm'}"
      {placeholder}
      bind:value={draft}
      oninput={schedule}
      onkeydown={(event) => event.key === 'Enter' && commit()} />
    {#if showCount}
      <span class="hidden whitespace-nowrap text-sm text-ink2 wide:inline" aria-live="polite">{@render count()}</span>
    {/if}
    {@render trailing?.()}
    {#if large}
      <button
        type="button"
        title="Search syntax"
        aria-expanded={help}
        class="size-16 shrink-0 rounded-full border border-line font-bold {help ? 'bg-chip' : 'bg-transparent'}"
        onclick={() => (help = !help)}>?</button>
      <Popover bind:open={help} class="top-28 right-0 box-content flex w-[min(calc(var(--spacing)*210),90vw)] flex-col gap-2 p-8">
        <div class="mb-3 font-semibold">Every term must match. Tap one to add it.</div>
        {#each QUERY_HELP as [term, meaning] (term)}
          <button
            type="button"
            class="grid grid-cols-[--spacing(75)_1fr] items-center gap-6 rounded-sm border-0 bg-transparent px-4 py-3 text-left hover:bg-chip"
            onclick={() => add(term)}>
            <code class="font-mono text-xs">{term}</code>
            <span class="text-sm text-ink2">{meaning}</span>
          </button>
        {/each}
      </Popover>
    {/if}
  </div>
  {#if showCount}
    <div class="pl-2 text-sm text-ink2 wide:hidden">{@render count()}</div>
  {/if}
</div>
