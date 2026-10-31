<script lang="ts">
  import { untrack } from 'svelte';
  import { runAction } from '../lib/actions';
  import { api, errorMessage, withParams } from '../lib/api';
  import Avatar from '../lib/components/Avatar.svelte';
  import BulkBar from '../lib/components/BulkBar.svelte';
  import Button from '../lib/components/Button.svelte';
  import Checkbox from '../lib/components/Checkbox.svelte';
  import Chip from '../lib/components/Chip.svelte';
  import Pill from '../lib/components/Pill.svelte';
  import PinButton from '../lib/components/PinButton.svelte';
  import PinnedConfirm from '../lib/components/PinnedConfirm.svelte';
  import PluginSlot from '../lib/plugins/PluginSlot.svelte';
  import QueryBar from '../lib/components/QueryBar.svelte';
  import Segmented from '../lib/components/Segmented.svelte';
  import UnsubscribeDialog from '../lib/components/UnsubscribeDialog.svelte';
  import VirtualList from '../lib/components/VirtualList.svelte';
  import { formatBytes, formatNumber, formatShortDate } from '../lib/format';
  import { combine, groupQuery, hasTerm, quote, toggleTerm } from '../lib/query';
  import { router } from '../lib/router.svelte';
  import { session } from '../lib/session.svelte';
  import { toasts } from '../lib/toasts.svelte';
  import type { Group, GroupBy, SortBy } from '../lib/types';
  import { unsubscribeBadge } from '../lib/unsubscribe';

  const PAGE = 500;
  const ROW_CONTENT_HEIGHT = 52;
  const HAIRLINE = 1;
  const ROW_HEIGHT = ROW_CONTENT_HEIGHT + HAIRLINE;
  const COLUMNS = 'grid-cols-[--spacing(9)_minmax(0,1fr)_--spacing(32)_--spacing(32)] wide:grid-cols-[--spacing(9)_minmax(0,2.4fr)_--spacing(42)_--spacing(36)_--spacing(38)_--spacing(35)_--spacing(70)_--spacing(30)]';
  const CHIPS: [string, string][] = [
    ['Unread', 'is:unread'],
    ['Older than a year', 'older_than:1y'],
    ['Newsletters', 'has:list'],
    ['Large', 'larger:5mb'],
    ['Can unsubscribe', 'has:unsubscribe'],
    ['Pinned', 'is:pinned'],
  ];
  const SORTS: [SortBy, string][] = [
    ['count', 'Most messages'],
    ['size', 'Largest'],
    ['unread', 'Most unread'],
    ['recent', 'Most recent'],
  ];
  const GROUP_LABELS: Record<GroupBy, string> = { sender: 'Sender', domain: 'Domain', list: 'Mailing list' };
  const GROUP_PLURALS: Record<GroupBy, string> = { sender: 'senders', domain: 'domains', list: 'lists' };

  let { accountId }: { accountId: number } = $props();

  let groupBy = $state<GroupBy>('sender');
  let sortBy = $state<SortBy>('count');
  let filter = $state(router.params.get('q') ?? '');
  let groups = $state<Group[]>([]);
  let loading = $state(false);
  let exhausted = $state(false);
  let selected = $state<Record<string, Group>>({});
  let unsubscribeOpen = $state(false);
  let pending = $state<{ action: string; confirm: boolean } | null>(null);
  let pinnedConfirmOpen = $state(false);
  let generation = 0;

  const selection = $derived(Object.values(selected));
  const selectedMessages = $derived(selection.reduce((sum, group) => sum + group.message_count, 0));
  const selectedBytes = $derived(selection.reduce((sum, group) => sum + group.total_size, 0));
  const who = $derived(selection.length === 1 ? (selection[0]!.name ?? selection[0]!.key) : `${formatNumber(selection.length)} ${GROUP_PLURALS[groupBy]}`);
  const singlePinned = $derived(selection.length === 1 && selection[0]!.pinned_count > 0);
  const skipped = $derived(singlePinned ? 0 : selection.reduce((sum, group) => sum + group.pinned_count, 0));
  const resetKey = $derived(`${accountId}|${groupBy}|${sortBy}|${filter}`);

  $effect(() => {
    const fromRoute = router.params.get('q') ?? '';
    untrack(() => {
      if (fromRoute !== filter) filter = fromRoute;
    });
  });

  $effect(() => {
    const key = router.params.get('select');
    if (!key) return;
    untrack(() => preselect(key));
  });

  async function preselect(key: string) {
    try {
      const [group] = await api.get<Group[]>(withParams(`/accounts/${accountId}/groups`, { q: `from:${quote(key)}`, group: 'sender', limit: 1 }));
      groupBy = 'sender';
      if (group) selected = { [group.key]: group };
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }

  async function load(reset: boolean) {
    if (loading && !reset) return;
    const current = ++generation;
    loading = true;
    try {
      const offset = reset ? 0 : groups.length;
      const page = await api.get<Group[]>(withParams(`/accounts/${accountId}/groups`, { q: filter, group: groupBy, sort: sortBy, limit: PAGE, offset }));
      if (current !== generation) return;
      groups = reset ? page : [...groups, ...page];
      exhausted = page.length < PAGE;
    } catch (error) {
      if (current === generation) toasts.show(errorMessage(error), 'error');
    } finally {
      if (current === generation) loading = false;
    }
  }

  $effect(() => {
    void resetKey;
    untrack(() => {
      if (!router.params.get('select')) selected = {};
      load(true);
    });
  });

  $effect(() => {
    if (session.dataVersion > 0) untrack(() => load(true));
  });

  function toggle(group: Group) {
    if (selected[group.key]) delete selected[group.key];
    else selected[group.key] = group;
  }

  function toggleAll() {
    if (selection.length > 0 && selection.length === groups.length) selected = {};
    else selected = Object.fromEntries(groups.map((group) => [group.key, group]));
  }

  async function run(action: string, confirm: boolean) {
    if (singlePinned) {
      pending = { action, confirm };
      pinnedConfirmOpen = true;
      return;
    }
    await execute(action, confirm, false);
  }

  async function execute(action: string, confirm: boolean, includePinned: boolean) {
    const queries = selection.map((group) => combine(filter, groupQuery(groupBy, group.key)));
    await runAction(accountId, { queries }, action, confirm, { count: selectedMessages - skipped, who }, includePinned);
    selected = {};
  }

  async function runPending() {
    if (!pending) return;
    try {
      await execute(pending.action, pending.confirm, true);
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    } finally {
      pending = null;
    }
  }

  function pinChanged() {
    selected = {};
    load(true);
  }
</script>

<div class="flex flex-col gap-7 pt-2">
  <QueryBar {accountId} bind:value={filter} large placeholder="Search senders, or try older_than:1y is:unread" />
  <PluginSlot name="search" context={{ accountId, query: filter, setQuery: (next: string) => (filter = next) }} />
  <div class="flex flex-wrap items-center gap-4">
    {#each CHIPS as [label, term] (term)}
      <Chip active={hasTerm(filter, term)} surface onclick={() => (filter = toggleTerm(filter, term))}>{label}</Chip>
    {/each}
    <div class="flex-1"></div>
    <Segmented
      label="Group by"
      dense
      bind:value={groupBy}
      options={[
        { value: 'sender', label: 'Sender' },
        { value: 'domain', label: 'Domain' },
        { value: 'list', label: 'Mailing list' },
      ]} />
  </div>
  <div class="flex flex-wrap items-center gap-5 text-sm text-ink2" role="radiogroup" aria-label="Sort by">
    <span>Sort by</span>
    {#each SORTS as [value, label] (value)}
      <button
        type="button"
        role="radio"
        aria-checked={sortBy === value}
        class="border-0 border-b-2 bg-transparent px-1 py-2 text-sm font-semibold {sortBy === value ? 'border-ink text-ink' : 'border-transparent text-ink3'}"
        onclick={() => (sortBy = value)}>{label}</button>
    {/each}
  </div>
  <div class="border-t border-line">
    <div class="box-content grid h-20 items-center gap-7 border-b border-line text-xs font-medium text-ink3 {COLUMNS}">
      <Checkbox label="Select all" checked={groups.length > 0 && selection.length === groups.length} partial={selection.length > 0} onchange={toggleAll} />
      <span>{GROUP_LABELS[groupBy]}</span>
      <span class="text-right">Messages</span>
      <span class="hidden text-right wide:inline">Unread</span>
      <span class="text-right">Size</span>
      <span class="hidden wide:inline">Latest</span>
      <span class="hidden wide:inline">Unsubscribe</span>
      <span class="hidden wide:inline"></span>
    </div>
    <VirtualList items={groups} key={(group) => group.key} rowHeight={ROW_HEIGHT} {resetKey} onEndReached={() => !exhausted && load(false)}>
      {#snippet row(group: Group)}
        {@const on = !!selected[group.key]}
        {@const badge = groupBy === 'domain' ? null : unsubscribeBadge(group)}
        {@const name = groupBy === 'sender' ? (group.name ?? group.key) : group.key}
        <div
          role="row"
          tabindex="-1"
          aria-selected={on}
          class="box-content grid h-26 cursor-pointer items-center gap-7 border-b border-line {COLUMNS} {on ? 'bg-surface2' : 'bg-transparent'}"
          onclick={() => toggle(group)}
          onkeydown={(event) => event.key === ' ' && (event.preventDefault(), toggle(group))}>
          <Checkbox label="Select {name}" checked={on} />
          <div class="flex min-w-0 items-center gap-5">
            <Avatar label={name} seed={group.key} size="sm" />
            <span class="flex min-w-0 flex-col leading-snug">
              <span class="truncate font-semibold">{name}</span>
              <span class="truncate text-xs text-ink3">
                {#if groupBy === 'sender'}{group.key}{:else if groupBy === 'list'}list: {group.list_id ?? group.key}{:else}{formatNumber(group.sender_count)} {group.sender_count === 1 ? 'sender' : 'senders'}{/if}
              </span>
            </span>
            {#if groupBy !== 'list'}
              <PinButton {accountId} kind={groupBy} key={group.key} pin={group.pin} label={name} onchange={pinChanged} />
            {/if}
          </div>
          <span class="text-right font-semibold tabular-nums">{formatNumber(group.message_count)}</span>
          <span class="hidden text-right text-ink2 tabular-nums wide:inline">{formatNumber(group.unread_count)}</span>
          <span class="text-right text-ink2 tabular-nums">{formatBytes(group.total_size)}</span>
          <span class="hidden text-ink2 wide:inline">{formatShortDate(group.last_date)}</span>
          <span class="hidden wide:inline">{#if badge}<Pill tone={badge.tone} size="badge">{badge.label}</Pill>{/if}</span>
          <span class="hidden wide:grid">
            <Button size="xs" class="bg-surface" onclick={(event) => (event.stopPropagation(), router.go('messages', { q: combine(filter, groupQuery(groupBy, group.key)) }))}>View</Button>
          </span>
        </div>
      {/snippet}
    </VirtualList>
    {#if groups.length === 0 && !loading}
      <div class="py-24 text-center text-ink2">
        {session.current?.last_synced_at ? 'Nothing matches this search. Try removing a filter.' : 'The first sync is still running. Senders appear as messages arrive.'}
      </div>
    {/if}
    <div class="h-48"></div>
  </div>
</div>

<BulkBar
  {accountId}
  count={selectedMessages}
  bytes={selectedBytes}
  {who}
  {skipped}
  onrun={run}
  onunsubscribe={groupBy === 'sender' ? () => (unsubscribeOpen = true) : undefined}
  onclear={() => (selected = {})} />

<UnsubscribeDialog bind:open={unsubscribeOpen} {accountId} senders={selection} {who} count={selectedMessages} ondone={() => ((selected = {}), load(true))} />

<PinnedConfirm bind:open={pinnedConfirmOpen} {who} action={pending?.action ?? ''} count={selectedMessages} onconfirm={runPending} />
