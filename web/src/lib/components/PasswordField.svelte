<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';

  let {
    label,
    value = $bindable(''),
    error = null,
    hint,
    ...rest
  }: Omit<HTMLInputAttributes, 'type'> & { label: string; error?: string | null; hint?: string } = $props();

  let revealed = $state(false);
</script>

<label class="flex flex-col gap-4">
  <span class="text-sm font-semibold">{label}</span>
  <span class="box-content flex h-24 items-center rounded-xl border bg-surface2 pr-3 pl-8 {error ? 'border-bad' : 'border-line'}">
    <input
      class="h-full min-w-0 flex-1 border-0 bg-transparent text-control outline-0"
      type={revealed ? 'text' : 'password'}
      aria-invalid={error ? true : undefined}
      bind:value
      {...rest} />
    <button
      type="button"
      class="h-18 rounded-md border-0 bg-transparent px-6 text-xs font-semibold text-ink2 hover:bg-chip hover:text-ink"
      aria-pressed={revealed}
      onclick={() => (revealed = !revealed)}>
      {revealed ? 'Hide' : 'Show'}
    </button>
  </span>
  {#if error}
    <span class="text-xs text-bad" role="alert">{error}</span>
  {:else if hint}
    <span class="text-xs text-ink2">{hint}</span>
  {/if}
</label>
