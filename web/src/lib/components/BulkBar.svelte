<script lang="ts">
  import { api, errorMessage } from '../api';
  import { formatBytes, formatNumber } from '../format';
  import { skippedNote } from '../pins';
  import { session } from '../session.svelte';
  import { toasts } from '../toasts.svelte';
  import type { Mailbox } from '../types';
  import Button from './Button.svelte';
  import Modal from './Modal.svelte';
  import Popover from './Popover.svelte';

  let {
    accountId,
    count,
    bytes,
    who,
    skipped = 0,
    onrun,
    onunsubscribe,
    onclear,
  }: {
    accountId: number;
    count: number;
    bytes: number;
    who: string;
    skipped?: number;
    onrun: (action: string, confirm: boolean) => Promise<void>;
    onunsubscribe?: () => void;
    onclear: () => void;
  } = $props();

  const MORE: [string, string][] = [
    ['mark_unread', 'Mark unread'],
    ['flag', 'Flag'],
    ['unflag', 'Unflag'],
  ];

  let menu = $state(false);
  let folders = $state(false);
  let confirmDelete = $state(false);
  let busy = $state<string | null>(null);
  let mailboxes = $state<Mailbox[]>([]);

  const gmail = $derived(session.current?.provider === 'gmail');

  $effect(() => {
    const id = accountId;
    api
      .get<Mailbox[]>(`/accounts/${id}/mailboxes`)
      .then((list) => (mailboxes = list.filter((mailbox) => !mailbox.special_use)))
      .catch(() => (mailboxes = []));
  });

  $effect(() => {
    toasts.raised = count > 0;
    return () => (toasts.raised = false);
  });

  async function run(action: string, confirm = false) {
    busy = action;
    menu = false;
    folders = false;
    try {
      await onrun(action, confirm);
      confirmDelete = false;
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    } finally {
      busy = null;
    }
  }
</script>

{#snippet menuItem(label: string, onclick: () => void)}
  <button type="button" class="rounded-md border-0 bg-transparent px-6 py-5 text-left text-sm font-medium hover:bg-chip" {onclick}>{label}</button>
{/snippet}

{#if count > 0}
  <div class="fixed bottom-12 left-1/2 z-35 w-max max-w-[calc(100vw-var(--spacing)*16)] -translate-x-1/2" role="toolbar" aria-label="Bulk actions">
    <Popover bind:open={menu} class="right-60 bottom-32 box-content flex w-120 flex-col">
      {#each MORE as [action, label] (action)}
        {@render menuItem(label, () => run(action))}
      {/each}
      {#if mailboxes.length > 0}
        <button type="button" class="flex rounded-md border-0 bg-transparent px-6 py-5 text-left text-sm font-medium hover:bg-chip" onclick={() => (folders = !folders)}>
          <span class="flex-1">Move to folder</span><span class="text-ink3">{folders ? '▾' : '▸'}</span>
        </button>
        {#if folders}
          <div class="flex max-h-120 flex-col overflow-auto pl-6">
            {#each mailboxes as mailbox (mailbox.id)}
              <button type="button" class="rounded-md border-0 bg-transparent px-6 py-4 text-left text-sm text-ink2 hover:bg-chip" onclick={() => run(`move:${mailbox.name}`)}>{mailbox.display_name}</button>
            {/each}
          </div>
        {/if}
      {/if}
      <div class="mx-3 my-2 h-px bg-line"></div>
      <button type="button" class="rounded-md border-0 bg-transparent px-6 py-5 text-left text-sm font-semibold text-bad hover:bg-bad-bg" onclick={() => ((menu = false), (confirmDelete = true))}>Delete permanently…</button>
    </Popover>
    <div class="flex flex-wrap items-center gap-3 rounded-full bg-btn py-4 pr-4 pl-10 text-btn-ink shadow-pop">
      <span class="mr-5 whitespace-nowrap text-sm"><b>{formatNumber(count)}</b> messages, {formatBytes(bytes)} <span class="opacity-60">from {who}</span>{#if skipped > 0}<span class="opacity-60">, {skippedNote(skipped)}</span>{/if}</span>
      <Button size="xl" variant="inverse" busy={busy === 'archive'} onclick={() => run('archive')}>Archive</Button>
      <Button size="xl" variant="on-dark" busy={busy === 'trash'} onclick={() => run('trash')}>Move to trash</Button>
      <Button size="xl" variant="on-dark" busy={busy === 'mark_read'} onclick={() => run('mark_read')}>Mark read</Button>
      <Button size="xl" variant="on-dark" aria-expanded={menu} onclick={() => ((menu = !menu), (folders = false))}>More ▾</Button>
      {#if onunsubscribe}
        <Button size="xl" variant="on-dark-outline" onclick={onunsubscribe}>Unsubscribe</Button>
      {/if}
      <button type="button" title="Clear selection" aria-label="Clear selection" class="size-18 rounded-full border-0 bg-transparent text-btn-ink opacity-60" onclick={onclear}>✕</button>
    </div>
  </div>
{/if}

<Modal bind:open={confirmDelete} title="Delete {formatNumber(count)} messages permanently?">
  <p class="m-0 text-pretty leading-normal text-ink2">
    {#if gmail}
      {formatBytes(bytes)} from {who}. Gmail moves them to its Trash and empties it after 30 days. Move to trash does the same and can be undone.
    {:else}
      {formatBytes(bytes)} from {who}. They are removed from the server right away and cannot be recovered. Move to trash keeps them recoverable instead.
    {/if}
  </p>
  {#snippet footer()}
    <Button size="dialog" onclick={() => (confirmDelete = false)}>Cancel</Button>
    <Button size="dialog" busy={busy === 'trash'} onclick={() => run('trash')}>Move to trash</Button>
    <Button size="dialog" variant="danger" busy={busy === 'delete'} onclick={() => run('delete', true)}>Delete permanently</Button>
  {/snippet}
</Modal>
