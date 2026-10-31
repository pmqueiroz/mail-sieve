<script lang="ts" module>
  export type PillTone = 'neutral' | 'muted' | 'strong' | 'good' | 'bad' | 'warn';
  export type PillSize = 'tag' | 'status' | 'badge' | 'badge-lg';

  const tones: Record<PillTone, string> = {
    neutral: 'bg-chip text-ink2',
    muted: 'bg-chip text-ink3',
    strong: 'bg-chip text-ink',
    good: 'bg-good-bg text-good',
    bad: 'bg-bad-bg text-bad',
    warn: 'bg-warn-bg text-warn',
  };

  const sizes: Record<PillSize, string> = {
    tag: 'h-8 px-4 text-tiny font-bold',
    status: 'h-11 px-5 text-xs font-semibold',
    badge: 'h-12 px-5 text-xs font-semibold',
    'badge-lg': 'h-14 gap-3 px-6 text-xs font-semibold',
  };
</script>

<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    tone = 'neutral',
    size = 'status',
    title,
    children,
  }: { tone?: PillTone; size?: PillSize; title?: string; children: Snippet } = $props();
</script>

<span class="inline-flex shrink-0 items-center whitespace-nowrap rounded-full {tones[tone]} {sizes[size]}" {title}>{@render children()}</span>
