<script lang="ts">
  import { untrack } from 'svelte';
  import { runAction } from '../lib/actions';
  import { api, errorMessage, withParams } from '../lib/api';
  import Avatar from '../lib/components/Avatar.svelte';
  import BulkBar from '../lib/components/BulkBar.svelte';
  import Button from '../lib/components/Button.svelte';
  import Checkbox from '../lib/components/Checkbox.svelte';
  import Icon from '../lib/components/Icon.svelte';
  import Pill from '../lib/components/Pill.svelte';
  import PinButton from '../lib/components/PinButton.svelte';
  import PinnedConfirm from '../lib/components/PinnedConfirm.svelte';
  import QueryBar from '../lib/components/QueryBar.svelte';
  import PluginSlot from '../lib/plugins/PluginSlot.svelte';
  import VirtualList from '../lib/components/VirtualList.svelte';
  import { formatBytes, formatNumber, formatShortDate } from '../lib/format';
  import { pinDescription } from '../lib/pins';
  import { router } from '../lib/router.svelte';
  import { session } from '../lib/session.svelte';
  import { toasts } from '../lib/toasts.svelte';
  import type { MessagePage, MessageRow, Pin, Preview, Totals } from '../lib/types';

  const PAGE = 300;
  const ROW_CONTENT_HEIGHT = 56;
  const HAIRLINE = 1;
  const ROW_HEIGHT = ROW_CONTENT_HEIGHT + HAIRLINE;
  const INBOX = 'INBOX';

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
  let pendingOne = $state<string | null>(null);
  let pinnedConfirmOpen = $state(false);
  let generation = 0;

  const gmail = $derived(session.current?.provider === 'gmail');
  const selection = $derived(Object.values(selected));
  const allLoaded = $derived(messages.length > 0 && selection.length === messages.length);
  const selectedCount = $derived(allMatching ? (totals?.count ?? 0) : selection.length);
  const selectedBytes = $derived(allMatching ? (totals?.bytes ?? 0) : selection.reduce((sum, m) => sum + m.size, 0));
  const skipped = $derived(allMatching ? (totals?.pinned ?? 0) : selection.filter((message) => message.pin).length);
  const who = $derived.by(() => {
    if (allMatching) return 'this search';
    const senders = new Set(selection.map((message) => message.from_addr));
    if (senders.size === 1) return selection[0]!.from_name ?? selection[0]!.from_addr;
    return `${formatNumber(senders.size)} senders`;
  });
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
        withParams(`/accounts/${accountId}/messages`, { q: query, limit: PAGE, before_date: cursor?.before_date, before_id: cursor?.before_id }),
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
      clear();
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
    if (allLoaded) selected = {};
    else selected = Object.fromEntries(messages.map((message) => [message.id, message]));
  }

  function clear() {
    selected = {};
    allMatching = false;
  }

  async function run(action: string, confirm: boolean) {
    const target = allMatching ? { query } : { ids: selection.map((message) => message.id) };
    await runAction(accountId, target, action, confirm || (allMatching && query === ''), { count: selectedCount - skipped, who });
    clear();
  }

  function runOne(action: string) {
    if (!active) return;
    if (active.pin) {
      pendingOne = action;
      pinnedConfirmOpen = true;
      return;
    }
    return executeOne(action, false);
  }

  async function executeOne(action: string, includePinned: boolean) {
    if (!active) return;
    try {
      await runAction(accountId, { ids: [active.id] }, action, false, { count: 1, who: active.from_name ?? active.from_addr }, includePinned);
      active = null;
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }

  async function pinChanged(pin: Pin | null) {
    if (active) active = { ...active, pin };
    await load(true);
    const current = active && messages.find((message) => message.id === active!.id);
    if (current) active = current;
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

  function folderTag(message: MessageRow): string | null {
    return gmail || message.mailbox === INBOX ? null : message.mailbox;
  }
</script>

<div class="grid grid-cols-[minmax(0,1fr)] gap-12 pt-2 split:grid-cols-[minmax(0,1fr)_minmax(0,--spacing(220))]">
  <div class="flex min-w-0 flex-col gap-6">
    <QueryBar {accountId} bind:value={query} bind:totals showCount={false} placeholder="Search messages">
      {#snippet trailing()}<span class="whitespace-nowrap text-xs text-ink3">Newest first</span>{/snippet}
    </QueryBar>
    <PluginSlot name="search" context={{ accountId, query, setQuery: (next: string) => (query = next) }} />
    {#if allLoaded}
      <div class="flex flex-wrap items-center gap-6 rounded-lg bg-chip px-7 py-5 text-sm">
        {#if allMatching}
          <span>All {formatNumber(totals?.count ?? 0)} matching messages are selected.</span>
          <button type="button" class="border-0 bg-transparent p-0 text-sm font-bold underline" onclick={clear}>Clear selection</button>
        {:else}
          <span>All {formatNumber(messages.length)} loaded messages are selected.</span>
          {#if totals && totals.count > messages.length}
            <button type="button" class="border-0 bg-transparent p-0 text-sm font-bold underline" onclick={() => (allMatching = true)}>Select all {formatNumber(totals.count)} matching messages</button>
          {/if}
        {/if}
      </div>
    {/if}
    <div class="border-t border-line">
      <div class="box-content flex h-18 items-center gap-6 border-b border-line text-xs text-ink3">
        <Checkbox label="Select loaded messages" checked={allLoaded} partial={selection.length > 0} onchange={toggleAll} />
        <span>{formatNumber(messages.length)} loaded</span>
      </div>
      <VirtualList items={messages} key={(message) => message.id} rowHeight={ROW_HEIGHT} {resetKey} onEndReached={() => load(false)}>
        {#snippet row(message: MessageRow)}
          {@const tag = folderTag(message)}
          <div
            role="button"
            tabindex="0"
            class="box-content flex h-28 cursor-pointer items-center gap-6 border-b border-line pr-4 {active?.id === message.id ? 'bg-chip' : 'bg-transparent'}"
            onclick={() => open(message)}
            onkeydown={(event) => event.key === 'Enter' && open(message)}>
            <Checkbox label="Select message" checked={allMatching || !!selected[message.id]} onchange={() => toggle(message)} />
            <span class="size-4 shrink-0 rounded-full bg-ring {message.unread ? 'opacity-100' : 'opacity-0'}"></span>
            <span class="flex min-w-0 flex-1 flex-col leading-row">
              <span class="flex items-center gap-4">
                <span class="truncate {message.unread ? 'font-bold' : 'font-medium'}">{message.from_name ?? message.from_addr}</span>
                {#if message.pin}<span class="shrink-0 text-ink3" title={pinDescription(message.pin)}><Icon name="pinned" size={12} /></span>{/if}
                {#if message.flagged}<Pill tone="warn" size="tag">Flagged</Pill>{/if}
                {#if tag}<Pill size="tag">{tag}</Pill>{/if}
              </span>
              <span class="truncate text-sm {message.unread ? 'font-semibold text-ink' : 'font-normal text-ink2'}">{message.subject ?? '(no subject)'}</span>
            </span>
            <span class="shrink-0 text-xs text-ink3">{formatShortDate(message.date)}</span>
          </div>
        {/snippet}
      </VirtualList>
      {#if messages.length === 0 && !loading}
        <div class="py-24 text-center text-ink2">No messages match this search.</div>
      {/if}
      <div class="h-48"></div>
    </div>
  </div>

  {#if active}
    <aside
      class="fixed inset-0 z-40 box-content flex flex-col overflow-hidden border border-line bg-surface wide:inset-y-12 wide:right-12 wide:left-auto wide:w-220 wide:rounded-3xl split:sticky split:top-8 split:z-auto split:h-[calc(100vh-var(--spacing)*80)] split:w-auto">
      <div class="flex flex-col gap-3 border-b border-line px-11 py-10">
        <div class="flex items-center gap-5">
          <Avatar label={active.from_name ?? active.from_addr} seed={active.from_addr} size="lg" />
          <span class="flex min-w-0 flex-1 flex-col leading-snug">
            <span class="truncate font-semibold">{active.from_name ?? active.from_addr}</span>
            <span class="truncate text-xs text-ink3">{active.from_addr}</span>
          </span>
          <PinButton {accountId} kind="sender" key={active.from_addr} pin={active.pin} label={active.from_name ?? active.from_addr} onchange={pinChanged} />
          <button type="button" aria-label="Close preview" class="size-16 shrink-0 rounded-full border border-line bg-transparent" onclick={() => (active = null)}>✕</button>
        </div>
        <div class="mt-4 text-pretty text-lg font-semibold tracking-snug">{active.subject ?? '(no subject)'}</div>
        <div class="text-xs text-ink3">{formatShortDate(active.date)} · {active.mailbox === INBOX ? 'Inbox' : active.mailbox} · {formatBytes(active.size)}</div>
      </div>
      <div class="flex-1 overflow-auto px-11 py-10 text-base leading-relaxed text-ink2">
        {#if previewError}
          <span class="text-bad">{previewError}</span>
        {:else if !preview}
          Loading message from the server…
        {:else}
          {#if preview.attachments.length > 0}<div class="mb-6 text-xs text-ink3">Attachments: {preview.attachments.join(', ')}</div>{/if}
          <pre class="m-0 font-sans break-words whitespace-pre-wrap">{preview.text || '(this message has no text part)'}</pre>
        {/if}
      </div>
      <div class="border-t border-line px-11 py-6 text-xs text-ink3">Plain-text preview. Images, links and formatting are never loaded.</div>
      <div class="flex flex-wrap gap-4 px-11 pt-6 pb-9">
        <Button size="xl-wide" variant="primary" onclick={() => runOne('archive')}>Archive</Button>
        <Button size="xl-wide" onclick={() => runOne('trash')}>Move to trash</Button>
        <Button size="xl-wide" onclick={() => router.go('senders', { select: active!.from_addr })}>All from this sender</Button>
      </div>
    </aside>
  {/if}
</div>

<BulkBar {accountId} count={selectedCount} bytes={selectedBytes} {who} {skipped} onrun={run} onclear={clear} />

<PinnedConfirm
  bind:open={pinnedConfirmOpen}
  who={active?.from_name ?? active?.from_addr ?? ''}
  action={pendingOne ?? ''}
  count={1}
  onconfirm={() => (pendingOne ? executeOne(pendingOne, true) : undefined)} />
