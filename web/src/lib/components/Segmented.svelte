<script lang="ts" generics="T extends string">
  let { value = $bindable(), options, label }: { value: T; options: { value: T; label: string }[]; label: string } = $props();
</script>

<div class="segmented" role="radiogroup" aria-label={label}>
  {#each options as option (option.value)}
    <button
      role="radio"
      aria-checked={value === option.value}
      class:active={value === option.value}
      onclick={() => (value = option.value)}>{option.label}</button>
  {/each}
</div>

<style>
  .segmented {
    display: inline-flex;
    padding: 2px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-muted);
  }
  button {
    border: none;
    background: none;
    padding: var(--space-1) var(--space-3);
    border-radius: 4px;
    cursor: pointer;
    font-weight: 550;
    color: var(--color-text-muted);
  }
  .active {
    background: var(--color-surface);
    color: var(--color-text);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
  }
</style>
