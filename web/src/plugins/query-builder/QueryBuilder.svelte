<script lang="ts">
  import Popover from '../../lib/components/Popover.svelte';
  import { formatTerm, parseTerm, splitTerms, type Term } from '../../lib/query';
  import AddTerm from './AddTerm.svelte';
  import { fieldLabel } from './fields';

  let { query, setQuery }: { query: string; setQuery: (query: string) => void } = $props();

  let adding = $state(false);

  const terms = $derived(splitTerms(query).map(parseTerm));

  function update(next: Term[]) {
    setQuery(next.map(formatTerm).join(' '));
  }

  function toggle(index: number) {
    update(terms.map((term, position) => (position === index ? { ...term, negated: !term.negated } : term)));
  }

  function remove(index: number) {
    update(terms.filter((_, position) => position !== index));
  }

  function add(term: Term) {
    update([...terms, term]);
    adding = false;
  }
</script>

<div class="flex flex-wrap items-center gap-3" aria-label="Search filters">
  {#each terms as term, index (index)}
    <span class="inline-flex h-17 items-center rounded-full border text-sm {term.negated ? 'border-bad text-bad' : 'border-line bg-surface'}">
      <button
        type="button"
        title={term.negated ? 'Include matches' : 'Exclude matches'}
        class="flex h-full items-center gap-3 rounded-l-full border-0 bg-transparent pr-3 pl-7 text-inherit"
        onclick={() => toggle(index)}>
        <span class="text-ink3">{term.negated ? 'not ' : ''}{fieldLabel(term.field)}</span>
        <span class="font-mono text-xs font-semibold {term.negated ? 'line-through' : ''}">{term.value}</span>
      </button>
      <button
        type="button"
        aria-label="Remove {fieldLabel(term.field)} {term.value}"
        class="grid h-full place-items-center rounded-r-full border-0 bg-transparent pr-6 pl-2 text-ink3 hover:text-ink"
        onclick={() => remove(index)}>
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
      </button>
    </span>
  {/each}
  <div class="relative">
    <button
      type="button"
      aria-expanded={adding}
      class="h-17 rounded-full border border-dashed border-line bg-transparent px-7 text-sm font-semibold text-ink2"
      onclick={() => (adding = !adding)}>+ Filter</button>
    <Popover bind:open={adding} class="top-20 left-0 box-content w-[min(calc(var(--spacing)*200),85vw)] p-8">
      <AddTerm onadd={add} />
    </Popover>
  </div>
</div>
