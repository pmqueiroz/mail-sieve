<script lang="ts">
  import { untrack } from 'svelte';
  import { errorMessage } from '../../lib/api';
  import Button from '../../lib/components/Button.svelte';
  import { session } from '../../lib/session.svelte';
  import { toasts } from '../../lib/toasts.svelte';
  import Balloon from './Balloon.svelte';
  import Ghoul, { type Look } from './Ghoul.svelte';
  import { wingman } from './wingman.svelte';

  const LOOK_RANGE = 180;
  const LOOK_X = 4;
  const LOOK_Y = 3.2;

  let ghoul = $state<HTMLElement | null>(null);
  let look = $state<Look>({ x: 0, y: 0 });
  let frame = 0;

  const sleeping = $derived(wingman.phase === 'sleep');

  $effect(() => {
    toasts.shifted = true;
    untrack(() => wingman.start());
    return () => {
      toasts.shifted = false;
      wingman.stop();
    };
  });

  $effect(() => {
    void session.accountId;
    void session.dataVersion;
    untrack(() => wingman.refresh().catch((error) => toasts.show(errorMessage(error), 'error')));
  });

  $effect(() => {
    const tracked = wingman.job;
    if (!tracked) return;
    const job = session.jobs.find((candidate) => candidate.id === tracked.id);
    void job?.state;
    untrack(() => wingman.watch(job));
  });

  $effect(() => {
    if (sleeping) look = { x: 0, y: 0 };
  });

  function track(event: PointerEvent) {
    if (sleeping || frame) return;
    const { clientX, clientY } = event;
    frame = requestAnimationFrame(() => {
      frame = 0;
      if (!ghoul) return;
      const rect = ghoul.getBoundingClientRect();
      const dx = clientX - (rect.left + rect.width / 2);
      const dy = clientY - (rect.top + rect.height * 0.42);
      const distance = Math.hypot(dx, dy) || 1;
      const reach = Math.min(1, distance / LOOK_RANGE);
      look = { x: Math.round((dx / distance) * LOOK_X * reach * 10) / 10, y: Math.round((dy / distance) * LOOK_Y * reach * 10) / 10 };
    });
  }
</script>

<svelte:window onpointermove={track} onpointerleave={() => (look = { x: 0, y: 0 })} />

<div class="pointer-events-none fixed right-14 bottom-11 z-30 hidden wide:block" aria-live="polite">
  {#if wingman.thinking}
    <div class="absolute right-61 bottom-67 animate-balloon-fade motion-reduce:animate-none" role="status">
      <div class="flex items-center gap-5 rounded-full border border-line bg-surface px-9 py-6 shadow-pop">
        <span class="flex gap-2.5" aria-hidden="true">
          {#each [0, 0.15, 0.3] as delay (delay)}
            <span class="size-3.5 animate-ghoul-dot rounded-full bg-ink2 motion-reduce:animate-none" style:animation-delay="{delay}s"></span>
          {/each}
        </span>
        <span class="text-sm whitespace-nowrap text-ink2">{wingman.thinking}</span>
      </div>
      <div class="absolute -right-6 -bottom-7 size-6 rounded-full border border-line bg-surface"></div>
      <div class="absolute -right-11 -bottom-13 size-3.5 rounded-full border border-line bg-surface"></div>
    </div>
  {:else if wingman.phase === 'suggest' && wingman.current}
    {@const current = wingman.current}
    <Balloon alert={current.alert} class="flex w-170 flex-col gap-6 px-9 pt-8 pb-9">
      <div class="flex items-start gap-4">
        <p class="m-0 flex-1 text-pretty text-control leading-normal">{current.lead}<b class="font-semibold">{current.strong}</b>{current.tail}</p>
        <button
          type="button"
          title="Send to nap"
          aria-label="Send the wingman to nap"
          class="grid size-13 flex-none place-items-center rounded-full border-0 bg-transparent p-0 text-ink3 hover:bg-chip"
          onclick={() => wingman.nap()}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
        </button>
      </div>
      <div class="flex items-center gap-3">
        <Button variant="primary" size="2xl" busy={wingman.busy} onclick={() => wingman.accept()}>{current.primary}</Button>
        <Button variant="muted-text" size="2xl" class="hover:bg-chip" disabled={wingman.busy} onclick={() => wingman.decline()}>{current.secondary}</Button>
      </div>
    </Balloon>
  {:else if wingman.phase === 'done'}
    <Balloon class="w-160 overflow-hidden">
      <div class="flex items-center gap-6 px-8 py-7">
        <span class="grid size-14 flex-none place-items-center rounded-full bg-good-bg text-good" aria-hidden="true">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="m5 12.5 4.5 4.5L19 7.5" /></svg>
        </span>
        <span class="flex-1 text-pretty leading-row">{wingman.message}</span>
        {#if wingman.undoable}
          <button type="button" class="border-0 bg-transparent p-2 text-sm font-semibold underline underline-offset-3" onclick={() => wingman.undo()}>Undo</button>
        {/if}
      </div>
      <div class="h-1.5 bg-track"><div class="h-full animate-countdown bg-ring"></div></div>
    </Balloon>
  {:else if wingman.phase === 'ack'}
    <Balloon class="w-max max-w-150 px-9 py-7 text-pretty leading-row">{wingman.message}</Balloon>
  {/if}

  <button
    type="button"
    class="pointer-events-auto block border-0 bg-transparent p-0"
    title={sleeping ? 'Wake wingman' : 'Poke'}
    aria-label={sleeping ? 'Wake the wingman' : 'Poke the wingman'}
    onclick={() => wingman.poke()}>
    <Ghoul mood={wingman.mood} size={sleeping ? 92 : 120} {look} bind:element={ghoul} />
  </button>
</div>
