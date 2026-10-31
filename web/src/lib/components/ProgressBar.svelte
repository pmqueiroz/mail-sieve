<script lang="ts" module>
  export type ProgressTone = 'btn' | 'good' | 'bad' | 'muted' | 'sieve';

  const tones: Record<ProgressTone, string> = {
    btn: 'bg-btn',
    good: 'bg-good',
    bad: 'bg-bad',
    muted: 'bg-ink3',
    sieve: 'bg-sieve',
  };
</script>

<script lang="ts">
  let {
    value,
    tone = 'btn',
    thin = false,
    class: className = '',
  }: { value: number; tone?: ProgressTone; thin?: boolean; class?: string } = $props();

  const width = $derived(Math.max(0, Math.min(100, value)));
</script>

<div
  class="overflow-hidden rounded-full bg-track {thin ? 'h-2' : 'h-3'} {className}"
  role="progressbar"
  aria-valuemin={0}
  aria-valuemax={100}
  aria-valuenow={Math.round(width)}>
  <div class="h-full transition-[width] duration-200 {tones[tone]}" style:width="{width}%"></div>
</div>
