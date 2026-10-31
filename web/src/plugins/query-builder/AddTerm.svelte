<script lang="ts">
  import Button from '../../lib/components/Button.svelte';
  import Chip from '../../lib/components/Chip.svelte';
  import TextInput from '../../lib/components/TextInput.svelte';
  import type { Term } from '../../lib/query';
  import { DURATION_UNITS, FIELDS, type FieldSpec, SIZE_UNITS } from './fields';

  let { onadd }: { onadd: (term: Term) => void } = $props();

  let spec = $state<FieldSpec>(FIELDS[0]!);
  let text = $state('');
  let amount = $state(1);
  let unit = $state('y');
  let negated = $state(false);

  const value = $derived.by(() => {
    if (spec.kind === 'duration') return amount > 0 ? `${amount}${unit}` : '';
    if (spec.kind === 'size') return amount > 0 ? `${amount}${unit}` : '';
    return text.trim();
  });

  function pick(next: FieldSpec) {
    spec = next;
    text = next.kind === 'choice' ? next.choices![0]! : '';
    amount = 1;
    unit = next.kind === 'size' ? 'mb' : 'y';
  }

  function add() {
    if (!value) return;
    onadd({ negated, field: spec.field, value });
    text = spec.kind === 'choice' ? text : '';
  }
</script>

<div class="flex flex-col gap-7">
  <div class="flex flex-wrap gap-3" role="radiogroup" aria-label="Field">
    {#each FIELDS as candidate (candidate.field)}
      <Chip active={spec.field === candidate.field} muted onclick={() => pick(candidate)}>{candidate.label}</Chip>
    {/each}
  </div>
  {#if spec.kind === 'choice'}
    <div class="flex flex-wrap gap-3">
      {#each spec.choices ?? [] as choice (choice)}
        <Chip active={text === choice} muted onclick={() => (text = choice)}>{choice}</Chip>
      {/each}
    </div>
  {:else if spec.kind === 'duration' || spec.kind === 'size'}
    <div class="flex flex-wrap items-center gap-4">
      <span class="flex w-36"><TextInput type="number" min="1" aria-label="Amount" bind:value={amount} /></span>
      {#each spec.kind === 'duration' ? DURATION_UNITS : SIZE_UNITS as [code, label] (code)}
        <Chip active={unit === code} muted onclick={() => (unit = code)}>{label}</Chip>
      {/each}
    </div>
  {:else if spec.kind === 'date'}
    <TextInput type="date" aria-label={spec.label} bind:value={text} />
  {:else}
    <TextInput mono spellcheck={false} aria-label={spec.label} placeholder={spec.placeholder} bind:value={text} onkeydown={(event) => event.key === 'Enter' && add()} />
  {/if}
  <div class="flex flex-wrap items-center gap-4">
    <Chip active={negated} muted onclick={() => (negated = !negated)}>Exclude matches</Chip>
    <div class="flex-1"></div>
    <Button variant="primary" disabled={!value} onclick={add}>Add filter</Button>
  </div>
</div>
