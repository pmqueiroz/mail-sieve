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

<Modal bind:open {title} width={440}>
  <p class="message">{message}</p>
  {#snippet footer()}
    <Button variant="ghost" onclick={() => (open = false)}>Cancel</Button>
    <Button variant={danger ? 'danger' : 'primary'} {busy} onclick={confirm}>{confirmLabel}</Button>
  {/snippet}
</Modal>

<style>
  .message {
    margin: 0;
    line-height: 1.5;
  }
</style>
