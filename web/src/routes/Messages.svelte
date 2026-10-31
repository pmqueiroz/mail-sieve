<script lang="ts">
  import { untrack } from 'svelte';
  import { api, errorMessage, withParams } from '../lib/api';
  import ActionBar from '../lib/components/ActionBar.svelte';
  import Badge from '../lib/components/Badge.svelte';
  import BulkActions from '../lib/components/BulkActions.svelte';
  import EmptyState from '../lib/components/EmptyState.svelte';
  import QueryBar from '../lib/components/QueryBar.svelte';
  import VirtualList from '../lib/components/VirtualList.svelte';
  import { actionLabel, formatBytes, formatDate, formatDateTime, formatNumber } from '../lib/format';
  import { router } from '../lib/router.svelte';
  import { session } from '../lib/session.svelte';
  import { toasts } from '../lib/toasts.svelte';
  import type { MessagePage, MessageRow, Preview, Totals } from '../lib/types';

  const PAGE = 300;

  let { accountId }: { accountId: number } = $props();

  let query = $state(router.params.get('q') ?? '');
  let totals = $state<Totals | null>(null);
  let messages = $state<MessageRow[]>([]);
  let next = $state<MessagePage['next']>(null);
  let loading = $state(false);
  let selected = $state<Record<number, MessageRow>>({});
  let allMatching = $state(false);
  let active = $state<MessageRow | null>(null);
  let preview = $state<Preview | null>(null);
  let previewError = $state<string | null>(null);
  let generation = 0;

  const selection = $derived(Object.values(selected));
  const selectedCount = $derived(allMatching ? (totals?.count ?? 0) : selection.length);
  const selectedBytes = $derived(allMatching ? (totals?.bytes ?? 0) : selection.reduce((sum, m) => sum + m.size, 0));
  const resetKey = $derived(`${accountId}|${query}`);

  $effect(() => {
    const fromRoute = router.params.get('q') ?? '';
    untrack(() => {
      if (fromRoute !== query) query = fromRoute;
    });
  });

  $effect(() => {
    const current = query;
    if ((router.params.get('q') ?? '') !== current) router.go('messages', current ? { q: current } : {});
  });

  async function load(reset: boolean) {
    if (!reset && (loading || !next)) return;
    const current = ++generation;
    loading = true;
    try {
      const cursor = reset ? null : next;
      const page = await api.get<MessagePage>(
        withParams(`/accounts/${accountId}/messages`, {
          q: query,
          limit: PAGE,
          before_date: cursor?.before_date,
          before_id: cursor?.before_id,
        }),
      );
      if (current !== generation) return;
      messages = reset ? page.messages : [...messages, ...page.messages];
      next = page.next;
    } catch (error) {
      if (current === generation) {
        messages = reset ? [] : messages;
        next = null;
        toasts.show(errorMessage(error), 'error');
      }
    } finally {
      if (current === generation) loading = false;
    }
  }

  $effect(() => {
    void resetKey;
    untrack(() => {
      selected = {};
      allMatching = false;
      load(true);
    });
  });

  $effect(() => {
    if (session.dataVersion > 0) untrack(() => load(true));
  });

  function toggle(message: MessageRow) {
    allMatching = false;
    if (selected[message.id]) delete selected[message.id];
    else selected[message.id] = message;
  }

  function toggleAll() {
    allMatching = false;
    if (selection.length > 0) selected = {};
    else selected = Object.fromEntries(messages.map((message) => [message.id, message]));
  }

  function clear() {
    selected = {};
    allMatching = false;
  }

  async function run(action: string, confirm: boolean) {
    const body = allMatching
      ? { query, action, confirm: confirm || query === '' }
      : { ids: selection.map((message) => message.id), action, confirm };
    const response = await api.post<{ job_id: number | null }>(`/accounts/${accountId}/actions`, body);
    if (response.job_id === null) toasts.show('Nothing to do, no messages matched.');
    else toasts.show(`${actionLabel(action)} queued for ${formatNumber(selectedCount)} messages.`, 'success');
    clear();
  }

  async function open(message: MessageRow) {
    active = message;
    preview = null;
    previewError = null;
    try {
      const loaded = await api.get<Preview>(`/messages/${message.id}/preview`);
      if (active?.id === message.id) preview = loaded;
    } catch (error) {
      if (active?.id === message.id) previewError = errorMessage(error);
    }
  }
</script>

<section class="page">
  <div class="page-header">
    <h1>Messages</h1>
  </div>

  <div class="toolbar">
    <QueryBar {accountId} bind:value={query} bind:totals placeholder="Search: from:*@linkedin.com older_than:1y" />
  </div>

  {#if selection.length > 0 && selection.length === messages.length && totals && totals.count > messages.length && !allMatching}
    <div class="select-all">
      All {formatNumber(messages.length)} loaded messages are selected.
      <button onclick={() => (allMatching = true)}>Select all {formatNumber(totals.count)} matching messages</button>
    </div>
  {/if}

  <ActionBar count={selectedCount} bytes={selectedBytes} onclear={clear}>
    <BulkActions {accountId} count={selectedCount} onrun={run} />
  </ActionBar>

  <div class="split">
    <div class="table card">
      <div class="head">
        <span class="check"><input type="checkbox" aria-label="Select loaded" checked={allMatching || (selection.length > 0 && selection.length === messages.length)} indeterminate={!allMatching && selection.length > 0 && selection.length < messages.length} onchange={toggleAll} /></span>
        <span>From</span>
        <span>Subject</span>
        <span class="right">Size</span>
        <span class="right">Date</span>
      </div>
      {#if messages.length === 0 && !loading}
        <EmptyState title="No messages match" />
      {:else}
        <VirtualList items={messages} key={(message) => message.id} {resetKey} onEndReached={() => load(false)}>
          {#snippet row(message: MessageRow)}
            <div class="line" class:selected={allMatching || !!selected[message.id]} class:active={active?.id === message.id} class:unread={message.unread}>
              <span class="check"><input type="checkbox" aria-label="Select message" checked={allMatching || !!selected[message.id]} onchange={() => toggle(message)} /></span>
              <button class="cell from truncate" onclick={() => open(message)}>{message.from_name ?? message.from_addr}</button>
              <button class="cell subject" onclick={() => open(message)}>
                <span class="truncate">{message.subject ?? '(no subject)'}</span>
                {#if message.flagged}<Badge tone="warning">flagged</Badge>{/if}
                {#if message.mailbox !== 'INBOX'}<Badge>{message.mailbox}</Badge>{/if}
              </button>
              <span class="right muted">{formatBytes(message.size)}</span>
              <span class="right muted">{formatDate(message.date)}</span>
            </div>
          {/snippet}
        </VirtualList>
      {/if}
    </div>

    {#if active}
      <aside class="preview card">
        <header>
          <h2>{active.subject ?? '(no subject)'}</h2>
          <button class="close" aria-label="Close preview" onclick={() => (active = null)}>×</button>
        </header>
        <dl>
          <dt>From</dt>
          <dd>{active.from_name ? `${active.from_name} <${active.from_addr}>` : active.from_addr}</dd>
          <dt>Date</dt>
          <dd>{formatDateTime(active.date)}</dd>
          <dt>Folder</dt>
          <dd>{active.mailbox}</dd>
        </dl>
        {#if previewError}
          <p class="error">{previewError}</p>
        {:else if !preview}
          <p class="muted">Loading message from the server…</p>
        {:else}
          {#if preview.attachments.length > 0}
            <p class="muted">Attachments: {preview.attachments.join(', ')}</p>
          {/if}
          <pre class="body">{preview.text || '(this message has no text part)'}</pre>
        {/if}
      </aside>
    {/if}
  </div>
</section>

<style>
  .select-all {
    font-size: var(--text-sm);
    text-align: center;
  }
  .select-all button {
    border: none;
    background: none;
    color: var(--color-accent);
    font-weight: 600;
    cursor: pointer;
  }
  .split {
    flex: 1;
    min-height: 0;
    display: flex;
    gap: var(--space-4);
  }
  .table {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .head,
  .line {
    display: grid;
    grid-template-columns: 40px minmax(110px, 220px) minmax(140px, 1fr) 70px 100px;
    align-items: center;
    width: 100%;
    padding: 0 var(--space-3);
    gap: var(--space-2);
  }
  .head {
    height: 38px;
    border-bottom: 1px solid var(--color-border);
    font-size: var(--text-xs);
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--color-text-muted);
  }
  .line {
    border-bottom: 1px solid var(--color-border);
  }
  .line:hover {
    background: var(--color-surface-muted);
  }
  .line.selected {
    background: var(--color-selected);
  }
  .line.active {
    box-shadow: inset 3px 0 0 var(--color-accent);
  }
  .line.unread .from,
  .line.unread .subject {
    font-weight: 650;
  }
  .cell {
    border: none;
    background: none;
    padding: 0;
    text-align: left;
    cursor: pointer;
    min-width: 0;
    height: 100%;
  }
  .subject {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .right {
    text-align: right;
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
  }
  .preview {
    width: min(46%, 640px);
    display: flex;
    flex-direction: column;
    padding: var(--space-4) var(--space-5);
    overflow: auto;
    gap: var(--space-3);
  }
  .preview header {
    display: flex;
    justify-content: space-between;
    gap: var(--space-3);
  }
  .preview h2 {
    margin: 0;
    font-size: var(--text-lg);
  }
  .close {
    border: none;
    background: none;
    font-size: 22px;
    cursor: pointer;
    color: var(--color-text-muted);
  }
  dl {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: var(--space-1) var(--space-3);
    margin: 0;
    font-size: var(--text-sm);
  }
  dt {
    color: var(--color-text-muted);
  }
  dd {
    margin: 0;
    word-break: break-word;
  }
  .body {
    white-space: pre-wrap;
    word-break: break-word;
    font-family: var(--font-sans);
    line-height: 1.5;
    margin: 0;
  }
  .error {
    color: var(--color-danger);
  }
</style>
