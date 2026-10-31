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

<Modal bind:open title={guide.title} width={600}>
  {#if notice}<div class="notice" role="alert">{notice}</div>{/if}
  <p class="intro">{guide.intro}</p>
  <ol class="steps">
    {#each guide.steps as step, index (step.title)}
      <li>
        <span class="number" aria-hidden="true">{index + 1}</span>
        <div class="step">
          <strong>{step.title}</strong>
          <p>{step.body}</p>
          {#if step.link}
            <a href={step.link.url} target="_blank" rel="noopener noreferrer">{step.link.label} ↗</a>
          {/if}
        </div>
      </li>
    {/each}
  </ol>
  {#if guide.troubleshooting.length > 0}
    <details>
      <summary>Still not working?</summary>
      <ul>
        {#each guide.troubleshooting as tip (tip)}<li>{tip}</li>{/each}
      </ul>
    </details>
  {/if}
  {#snippet footer()}
    <Button variant="primary" onclick={() => (open = false)}>Got it</Button>
  {/snippet}
</Modal>

<style>
  .notice {
    padding: var(--space-3);
    border-radius: var(--radius-sm);
    background: var(--color-danger-soft);
    color: var(--color-danger);
    line-height: 1.5;
  }
  .intro {
    margin: 0;
    line-height: 1.5;
  }
  .steps {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }
  .steps li {
    display: flex;
    gap: var(--space-3);
  }
  .number {
    flex-shrink: 0;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--color-accent-soft);
    color: var(--color-accent);
    font-weight: 700;
  }
  .step {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding-top: 3px;
  }
  .step p {
    margin: 0;
    line-height: 1.5;
    color: var(--color-text-muted);
  }
  .step a {
    font-weight: 600;
    width: fit-content;
  }
  details {
    border-top: 1px solid var(--color-border);
    padding-top: var(--space-3);
  }
  summary {
    cursor: pointer;
    font-weight: 600;
  }
  details ul {
    margin: var(--space-2) 0 0;
    padding-left: var(--space-5);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    line-height: 1.5;
    color: var(--color-text-muted);
  }
</style>
