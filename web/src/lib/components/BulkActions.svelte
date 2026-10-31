<script lang="ts">
  import { api, errorMessage } from '../api';
  import { actionLabel, formatNumber } from '../format';
  import { toasts } from '../toasts.svelte';
  import type { Mailbox } from '../types';
  import Button from './Button.svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';

  let {
    accountId,
    count,
    onrun,
  }: { accountId: number; count: number; onrun: (action: string, confirm: boolean) => Promise<void> } = $props();

  let busy = $state<string | null>(null);
  let more = $state('');
  let mailboxes = $state<Mailbox[]>([]);
  let confirmDelete = $state(false);

  $effect(() => {
    const id = accountId;
    api
      .get<Mailbox[]>(`/accounts/${id}/mailboxes`)
      .then((list) => (mailboxes = list))
      .catch(() => (mailboxes = []));
  });

  async function run(action: string, confirm = false) {
    busy = action;
    try {
      await onrun(action, confirm);
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    } finally {
      busy = null;
    }
  }

  function chooseMore() {
    const action = more;
    more = '';
    if (!action) return;
    if (action === 'delete') confirmDelete = true;
    else run(action);
  }
</script>

<Button size="sm" variant="primary" busy={busy === 'archive'} onclick={() => run('archive')}>Archive</Button>
<Button size="sm" busy={busy === 'trash'} onclick={() => run('trash')}>Move to trash</Button>
<Button size="sm" busy={busy === 'mark_read'} onclick={() => run('mark_read')}>Mark read</Button>
<select class="more" aria-label="More actions" bind:value={more} onchange={chooseMore} disabled={busy !== null}>
  <option value="">More…</option>
  <option value="mark_unread">Mark unread</option>
  <option value="flag">Flag</option>
  <option value="unflag">Unflag</option>
  {#if mailboxes.length > 0}
    <optgroup label="Move to folder">
      {#each mailboxes as mailbox (mailbox.id)}
        <option value="move:{mailbox.name}">{mailbox.display_name}</option>
      {/each}
    </optgroup>
  {/if}
  <option value="delete">Delete permanently…</option>
</select>

<ConfirmDialog
  bind:open={confirmDelete}
  title="Delete permanently?"
  message="This removes {formatNumber(count)} messages from the server and cannot be undone. Gmail accounts move them to Trash instead."
  confirmLabel={actionLabel('delete')}
  danger
  onconfirm={() => run('delete', true)} />

<style>
  .more {
    width: auto;
    padding: var(--space-1) var(--space-2);
    font-size: var(--text-sm);
  }
</style>
