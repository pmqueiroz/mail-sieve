<script lang="ts">
  import Button from './Button.svelte';
  import Modal from './Modal.svelte';

  let {
    open = $bindable(false),
    title,
    message,
    confirmLabel = 'Confirm',
    danger = false,
    onconfirm,
  }: {
    open?: boolean;
    title: string;
    message: string;
    confirmLabel?: string;
    danger?: boolean;
    onconfirm: () => void | Promise<void>;
  } = $props();

  let busy = $state(false);

  async function confirm() {
    busy = true;
    try {
      await onconfirm();
      open = false;
    } finally {
      busy = false;
    }
  }
</script>

<Modal bind:open {title}>
  <p class="m-0 text-pretty leading-normal text-ink2">{message}</p>
  {#snippet footer()}
    <Button size="dialog" onclick={() => (open = false)}>Cancel</Button>
    <Button size="dialog" variant={danger ? 'danger' : 'primary'} {busy} onclick={confirm}>{confirmLabel}</Button>
  {/snippet}
</Modal>
