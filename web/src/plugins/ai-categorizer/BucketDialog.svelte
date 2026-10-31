<script lang="ts">
  import { runAction } from '../../lib/actions';
  import { errorMessage } from '../../lib/api';
  import Avatar from '../../lib/components/Avatar.svelte';
  import Button from '../../lib/components/Button.svelte';
  import Modal from '../../lib/components/Modal.svelte';
  import { formatNumber } from '../../lib/format';
  import { groupQuery } from '../../lib/query';
  import { router } from '../../lib/router.svelte';
  import { toasts } from '../../lib/toasts.svelte';
  import type { Bucket } from './categorize';

  let { open = $bindable(false), accountId, bucket }: { open?: boolean; accountId: number; bucket: Bucket | null } = $props();

  let running = $state<string | null>(null);

  async function run(action: string) {
    if (!bucket) return;
    running = action;
    try {
      const queries = bucket.senders.map((sender) => groupQuery('sender', sender.key));
      await runAction(accountId, { queries }, action, false, { count: bucket.messages, who: `${bucket.category} senders` });
      open = false;
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    } finally {
      running = null;
    }
  }

  function view(key: string) {
    open = false;
    router.go('messages', { q: groupQuery('sender', key) });
  }
</script>

<Modal bind:open title={bucket?.category ?? ''}>
  {#if bucket}
    <span class="text-sm text-ink2">{formatNumber(bucket.senders.length)} {bucket.senders.length === 1 ? 'sender' : 'senders'} · {formatNumber(bucket.messages)} messages. Pinned senders are skipped.</span>
    <div class="flex max-h-[50vh] flex-col overflow-auto">
      {#each bucket.senders as sender (sender.key)}
        <div class="box-content flex min-h-24 items-center gap-6 border-b border-line">
          <Avatar label={sender.name ?? sender.key} seed={sender.key} size="sm" />
          <span class="flex min-w-0 flex-1 flex-col leading-snug">
            <span class="truncate font-semibold">{sender.name ?? sender.key}</span>
            <span class="truncate text-xs text-ink3">{sender.key}</span>
          </span>
          <span class="text-sm text-ink2 tabular-nums">{formatNumber(sender.message_count)}</span>
          <Button size="xs" onclick={() => view(sender.key)}>View</Button>
        </div>
      {/each}
    </div>
  {/if}
  {#snippet footer()}
    <Button size="dialog" onclick={() => (open = false)}>Close</Button>
    <Button size="dialog" busy={running === 'mark_read'} disabled={running !== null} onclick={() => run('mark_read')}>Mark all read</Button>
    <Button size="dialog" variant="primary" busy={running === 'archive'} disabled={running !== null} onclick={() => run('archive')}>Archive all</Button>
  {/snippet}
</Modal>
