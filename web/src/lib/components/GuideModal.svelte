<script lang="ts">
  import type { ProviderGuide } from '../guides';
  import Button from './Button.svelte';
  import Modal from './Modal.svelte';

  let {
    open = $bindable(false),
    guide,
    notice = null,
  }: { open?: boolean; guide: ProviderGuide; notice?: string | null } = $props();
</script>

<Modal bind:open title={guide.title} wide>
  {#if notice}<div class="rounded-xl bg-bad-bg p-7 leading-normal text-bad" role="alert">{notice}</div>{/if}
  <p class="m-0 leading-normal text-ink2">{guide.intro}</p>
  <ol class="m-0 flex list-none flex-col gap-8 p-0">
    {#each guide.steps as step, index (step.title)}
      <li class="flex gap-6">
        <span class="grid size-14 shrink-0 place-items-center rounded-full bg-chip font-bold" aria-hidden="true">{index + 1}</span>
        <div class="flex flex-col gap-2 pt-1">
          <strong class="font-semibold">{step.title}</strong>
          <p class="m-0 leading-normal text-ink2">{step.body}</p>
          {#if step.link}
            <a href={step.link.url} target="_blank" rel="noopener noreferrer" class="w-fit text-sm font-semibold">{step.link.label} ↗</a>
          {/if}
        </div>
      </li>
    {/each}
  </ol>
  {#if guide.troubleshooting.length > 0}
    <details class="border-t border-line pt-6">
      <summary class="cursor-pointer font-semibold">Still not working?</summary>
      <ul class="mt-4 mb-0 flex flex-col gap-4 pl-12 leading-normal text-ink2">
        {#each guide.troubleshooting as tip (tip)}<li>{tip}</li>{/each}
      </ul>
    </details>
  {/if}
  {#snippet footer()}
    <Button size="dialog" variant="primary" onclick={() => (open = false)}>Got it</Button>
  {/snippet}
</Modal>
