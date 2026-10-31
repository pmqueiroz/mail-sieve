<script lang="ts" generics="T extends string">
  let {
    value = $bindable(),
    options,
    label,
    dense = false,
  }: { value: T; options: { value: T; label: string }[]; label: string; dense?: boolean } = $props();
</script>

<div class="flex gap-1 rounded-full border border-line p-1" role="radiogroup" aria-label={label}>
  {#each options as option (option.value)}
    <button
      role="radio"
      aria-checked={value === option.value}
      class="rounded-full border-0 py-3 text-sm font-medium {dense ? 'px-6' : 'px-7'} {value === option.value ? 'bg-btn text-btn-ink' : 'bg-transparent text-ink2'}"
      onclick={() => (value = option.value)}>{option.label}</button>
  {/each}
</div>
