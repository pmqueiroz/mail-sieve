<script lang="ts" module>
  export type Mood = 'idle' | 'think' | 'suggest' | 'happy' | 'alert' | 'sleep';

  export interface Look {
    x: number;
    y: number;
  }

  const THOUGHTS = [
    [98, 14, 3],
    [108, 4, 3.8],
    [120, -7, 4.6],
  ] as const;

  const SPARKS = [
    [4, 22, 0],
    [116, 30, 0.35],
    [-4, 62, 0.7],
    [112, 2, 1],
  ] as const;

  const MOUTHS: Record<Mood, { d: string; filled: boolean; width: number }> = {
    idle: { d: 'M55 75 Q60 79.5 65 75', filled: false, width: 2.6 },
    suggest: { d: 'M53 74 Q60 84 67 74 Z', filled: true, width: 1.5 },
    happy: { d: 'M51 73 Q60 89 69 73 Z', filled: true, width: 1.5 },
    think: { d: 'M61.6 77 a2.4 2.4 0 1 0 4.8 0 a2.4 2.4 0 1 0 -4.8 0', filled: true, width: 0 },
    alert: { d: 'M56.6 78 a3.4 4.4 0 1 0 6.8 0 a3.4 4.4 0 1 0 -6.8 0', filled: true, width: 0 },
    sleep: { d: 'M56 77 Q60 79 64 77', filled: false, width: 2.4 },
  };

  function sparkPath(x: number, y: number) {
    return `M${x} ${y - 6} L${x + 2} ${y - 2} L${x + 6} ${y} L${x + 2} ${y + 2} L${x} ${y + 6} L${x - 2} ${y + 2} L${x - 6} ${y} L${x - 2} ${y - 2} Z`;
  }
</script>

<script lang="ts">
  let {
    mood,
    size,
    look = { x: 0, y: 0 },
    element = $bindable(null),
  }: { mood: Mood; size: number; look?: Look; element?: HTMLElement | null } = $props();

  const uid = $props.id();
  const gradient = `ghoul-${uid}`;
  const waving = $derived(mood === 'suggest' || mood === 'happy');
  const openEyes = $derived(mood !== 'happy' && mood !== 'sleep');
  const eyeShift = $derived(mood === 'think' ? { x: 2.5, y: -3 } : { x: 0, y: 0 });
  const eyeRadius = $derived(mood === 'alert' ? { x: 6.5, y: 9 } : { x: 5.5, y: 7.5 });
  const mouth = $derived(MOUTHS[mood]);
  const body = $derived(
    mood === 'happy' ? 'animate-ghoul-bounce' : mood === 'alert' ? 'animate-ghoul-jitter' : mood === 'sleep' ? 'translate-y-3 -rotate-8' : '',
  );
</script>

<div bind:this={element} class="flex select-none flex-col items-center transition-[width] duration-500 ease-out motion-reduce:transition-none" style:width="{size}px">
  <div class="animate-ghoul-float motion-reduce:animate-none">
    <div class="origin-[50%_90%] transition-transform duration-500 motion-reduce:animate-none {body}">
      <svg class="block overflow-visible drop-shadow-ghoul transition-[width,height] duration-500 ease-out motion-reduce:transition-none" style:width="{size}px" style:height="{(size * 150) / 140}px" viewBox="-10 -14 140 150" aria-hidden="true">
        <defs>
          <radialGradient id={gradient} cx="38%" cy="28%" r="85%">
            <stop offset="0%" class="[stop-color:var(--color-ghoul-hi)]" />
            <stop offset="62%" class="[stop-color:var(--color-ghoul-body)]" />
            <stop offset="100%" class="[stop-color:var(--color-ghoul-shade)]" />
          </radialGradient>
        </defs>
        <g class="origin-right [transform-box:fill-box] motion-reduce:animate-none {mood === 'happy' ? 'animate-ghoul-wave-back' : ''}">
          <ellipse cx="15" cy="86" rx="8" ry="5.5" transform={mood === 'sleep' ? 'rotate(-60 15 86)' : 'rotate(-25 15 86)'} class="fill-ghoul-body stroke-ghoul-stroke" stroke-width="1.5" />
        </g>
        <g class="origin-left [transform-box:fill-box] motion-reduce:animate-none {waving ? (mood === 'happy' ? 'animate-ghoul-wave-fast' : 'animate-ghoul-wave') : ''}">
          <ellipse
            cx="105"
            cy={waving ? 78 : 86}
            rx="8"
            ry="5.5"
            transform={waving ? 'rotate(-35 105 78)' : mood === 'sleep' ? 'rotate(60 105 86)' : 'rotate(25 105 86)'}
            class="fill-ghoul-body stroke-ghoul-stroke"
            stroke-width="1.5" />
        </g>
        <path
          d="M20 62 C20 39.9 37.9 22 60 22 C82.1 22 100 39.9 100 62 L100 116 Q93.3 128 86.7 116 Q80 104 73.3 116 Q66.7 128 60 116 Q53.3 104 46.7 116 Q40 128 33.3 116 Q26.7 104 20 116 Z"
          fill="url(#{gradient})"
          class="stroke-ghoul-stroke motion-reduce:animate-none {mood === 'sleep' ? 'animate-ghoul-tail-slow' : 'animate-ghoul-tail'}"
          stroke-width="1.5" />
        <g class="transition-transform duration-250 ease-out" style:transform="translate({look.x}px, {look.y}px)">
          {#if openEyes}
            <g class="origin-center animate-ghoul-blink [transform-box:fill-box] motion-reduce:animate-none">
              <ellipse cx={45 + eyeShift.x} cy={62 + eyeShift.y} rx={eyeRadius.x} ry={eyeRadius.y} class="fill-ghoul-eye" />
              <ellipse cx={75 + eyeShift.x} cy={62 + eyeShift.y} rx={eyeRadius.x} ry={eyeRadius.y} class="fill-ghoul-eye" />
              <circle cx={47 + eyeShift.x} cy={59 + eyeShift.y} r="1.9" class="fill-ghoul-hi" />
              <circle cx={77 + eyeShift.x} cy={59 + eyeShift.y} r="1.9" class="fill-ghoul-hi" />
            </g>
          {:else}
            {@const [left, right] = mood === 'happy' ? ['M39 64 Q45 55 51 64', 'M69 64 Q75 55 81 64'] : ['M39 62 Q45 67 51 62', 'M69 62 Q75 67 81 62']}
            <path d={left} fill="none" class="stroke-ghoul-eye" stroke-width="3.6" stroke-linecap="round" />
            <path d={right} fill="none" class="stroke-ghoul-eye" stroke-width="3.6" stroke-linecap="round" />
          {/if}
          <ellipse cx="36" cy="75" rx="6" ry="3.5" class="fill-sieve-coral" opacity={mood === 'happy' ? 0.6 : 0.38} />
          <ellipse cx="84" cy="75" rx="6" ry="3.5" class="fill-sieve-coral" opacity={mood === 'happy' ? 0.6 : 0.38} />
          <path
            d={mouth.d}
            class="stroke-ghoul-eye {mouth.filled ? 'fill-ghoul-eye' : 'fill-none'}"
            stroke-width={mouth.width}
            stroke-linecap="round"
            stroke-linejoin="round" />
        </g>
        {#if mood === 'think'}
          {#each THOUGHTS as [cx, cy, r], index (index)}
            <circle {cx} {cy} {r} class="fill-ghoul-accent animate-ghoul-thought motion-reduce:animate-none" style:animation-delay="{index * 0.18}s" />
          {/each}
        {:else if mood === 'sleep'}
          {#each [0, 1.1] as delay, index (index)}
            <text x="94" y="22" font-size={15 - index * 3} class="fill-ghoul-accent font-mono font-bold animate-ghoul-z [transform-box:fill-box] motion-reduce:animate-none" style:animation-delay="{delay}s">z</text>
          {/each}
        {:else if mood === 'alert'}
          <circle cx="102" cy="28" r="10" class="fill-ghoul-alert" />
          <text x="102" y="33" text-anchor="middle" font-size="14" class="fill-ghoul-eye font-sans font-bold">!</text>
        {:else if mood === 'happy'}
          {#each SPARKS as [x, y, delay], index (index)}
            <path d={sparkPath(x, y)} class="origin-center fill-sieve-amber animate-ghoul-spark [transform-box:fill-box] motion-reduce:animate-none" style:animation-delay="{delay}s" />
          {/each}
        {/if}
      </svg>
    </div>
  </div>
  <div
    class="animate-ghoul-ground rounded-full bg-ghoul-ground motion-reduce:animate-none"
    style:width="{size * 0.48}px"
    style:height="{Math.max(5, size * 0.07)}px"
    style:margin-top="{-size * 0.03}px"></div>
</div>
