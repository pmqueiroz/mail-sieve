<script lang="ts">
  let {
    checked,
    partial = false,
    label,
    onchange,
  }: { checked: boolean; partial?: boolean; label: string; onchange?: () => void } = $props();

  const on = $derived(checked || partial);
</script>

{#snippet box()}
  <span class="grid size-9 place-items-center rounded-check border-check {on ? 'border-btn bg-btn' : 'border-ink3 bg-transparent'}">
    <svg class="text-btn-ink {checked ? 'opacity-100' : partial ? 'opacity-50' : 'opacity-0'}" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3.5" aria-hidden="true"><path d="M5 12.5l4.5 4.5L19 7.5" /></svg>
  </span>
{/snippet}

{#if onchange}
  <button
    type="button"
    role="checkbox"
    aria-checked={partial && !checked ? 'mixed' : checked}
    aria-label={label}
    class="grid border-0 bg-transparent p-0"
    onclick={(event) => (event.stopPropagation(), onchange())}>{@render box()}</button>
{:else}
  <span role="checkbox" aria-checked={checked} aria-label={label}>{@render box()}</span>
{/if}
