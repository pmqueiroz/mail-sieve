<script lang="ts" generics="T extends string">
  let {
    value = $bindable(),
    options,
    label,
    dense = false,
  }: { value: T; options: { value: T; label: string }[]; label: string; dense?: boolean } = $props();

  let buttons: HTMLButtonElement[] = $state([]);
  let thumb = $state<{ left: number; width: number } | null>(null);
  let animate = $state(false);

  $effect(() => {
    const active = buttons[options.findIndex((option) => option.value === value)];
    if (!active) return;
    const measure = () => (thumb = { left: active.offsetLeft, width: active.offsetWidth });
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(active);
    requestAnimationFrame(() => (animate = true));
    return () => observer.disconnect();
  });
</script>

<div class="relative flex gap-1 rounded-full border border-line p-1" role="radiogroup" aria-label={label}>
  {#if thumb}
    <span
      class="absolute inset-y-1 rounded-full bg-btn motion-reduce:transition-none {animate ? 'transition-[left,width] duration-200 ease-out' : ''}"
      style:left="{thumb.left}px"
      style:width="{thumb.width}px"
      aria-hidden="true"></span>
  {/if}
  {#each options as option, index (option.value)}
    <button
      bind:this={buttons[index]}
      role="radio"
      aria-checked={value === option.value}
      class="relative rounded-full border-0 bg-transparent py-3 text-sm font-medium transition-colors duration-200 {dense ? 'px-6' : 'px-7'} {value === option.value ? 'text-btn-ink' : 'text-ink2'}"
      onclick={() => (value = option.value)}>{option.label}</button>
  {/each}
</div>
