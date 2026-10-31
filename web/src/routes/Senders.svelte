<script lang="ts">
  import { untrack } from 'svelte';
  import { api, errorMessage, withParams } from '../lib/api';
  import ActionBar from '../lib/components/ActionBar.svelte';
  import Badge from '../lib/components/Badge.svelte';
  import BulkActions from '../lib/components/BulkActions.svelte';
  import Button from '../lib/components/Button.svelte';
  import EmptyState from '../lib/components/EmptyState.svelte';
  import QueryBar from '../lib/components/QueryBar.svelte';
  import Segmented from '../lib/components/Segmented.svelte';
  import UnsubscribeDialog from '../lib/components/UnsubscribeDialog.svelte';
  import VirtualList from '../lib/components/VirtualList.svelte';
  import { actionLabel, formatBytes, formatDate, formatNumber } from '../lib/format';
  import { combine, groupQuery } from '../lib/query';
  import { router } from '../lib/router.svelte';
  import { session } from '../lib/session.svelte';
  import { toasts } from '../lib/toasts.svelte';
  import type { Group, GroupBy, SortBy } from '../lib/types';

  const PAGE = 500;

  let { accountId }: { accountId: number } = $props();

  let groupBy = $state<GroupBy>('sender');
  let sortBy = $state<SortBy>('count');
  let filter = $state('');
  let groups = $state<Group[]>([]);
  let loading = $state(false);
  let exhausted = $state(false);
  let selected = $state<Record<string, Group>>({});
  let unsubscribeOpen = $state(false);
  let generation = 0;

  const selection = $derived(Object.values(selected));
  const selectedMessages = $derived(selection.reduce((sum, group) => sum + group.message_count, 0));
  const selectedBytes = $derived(selection.reduce((sum, group) => sum + group.total_size, 0));
  const resetKey = $derived(`${accountId}|${groupBy}|${sortBy}|${filter}`);

  async function load(reset: boolean) {
    if (loading && !reset) return;
    const current = ++generation;
    loading = true;
    try {
      const offset = reset ? 0 : groups.length;
      const page = await api.get<Group[]>(
        withParams(`/accounts/${accountId}/groups`, { q: filter, group: groupBy, sort: sortBy, limit: PAGE, offset }),
      );
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
      selected = {};
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
    if (selection.length > 0) selected = {};
    else selected = Object.fromEntries(groups.map((group) => [group.key, group]));
  }

  function queryFor(group: Group) {
    return combine(filter, groupQuery(groupBy, group.key));
  }

  async function run(action: string, confirm: boolean) {
    const queries = selection.map(queryFor);
    const response = await api.post<{ job_id: number | null }>(`/accounts/${accountId}/actions`, { queries, action, confirm });
    if (response.job_id === null) toasts.show('Nothing to do, no messages matched.');
    else toasts.show(`${actionLabel(action)} queued for ${formatNumber(selectedMessages)} messages.`, 'success');
    selected = {};
  }

  function unsubscribeLabel(group: Group): { tone: 'success' | 'warning' | 'danger' | 'accent'; text: string } | null {
    if (group.unsubscribe_status === 'completed') return { tone: 'success', text: 'unsubscribed' };
    if (group.unsubscribe_status === 'link_opened') return { tone: 'warning', text: 'link opened' };
    if (group.unsubscribe_status === 'failed') return { tone: 'danger', text: 'unsubscribe failed' };
    if (group.unsub_one_click) return { tone: 'accent', text: 'one-click' };
    if (group.unsub_http || group.unsub_mailto) return { tone: 'accent', text: 'unsubscribe' };
    return null;
  }
</script>

<section class="page">
  <div class="page-header">
    <h1>Senders</h1>
    <div class="toolbar">
      <Segmented
        label="Group by"
        bind:value={groupBy}
        options={[
          { value: 'sender', label: 'Sender' },
          { value: 'domain', label: 'Domain' },
          { value: 'list', label: 'Mailing list' },
        ]} />
      <select class="sort" aria-label="Sort by" bind:value={sortBy}>
        <option value="count">Most messages</option>
        <option value="size">Largest</option>
        <option value="unread">Most unread</option>
        <option value="recent">Most recent</option>
      </select>
    </div>
  </div>

  <div class="toolbar">
    <QueryBar {accountId} bind:value={filter} placeholder="Narrow down: older_than:6m has:unsubscribe" />
  </div>

  <ActionBar count={selectedMessages} bytes={selectedBytes} onclear={() => (selected = {})}>
    <BulkActions {accountId} count={selectedMessages} onrun={run} />
    {#if groupBy === 'sender'}
      <Button size="sm" onclick={() => (unsubscribeOpen = true)}>Unsubscribe…</Button>
    {/if}
  </ActionBar>

  <div class="table card">
    <div class="head">
      <span class="check"><input type="checkbox" aria-label="Select all" checked={selection.length > 0 && selection.length === groups.length} indeterminate={selection.length > 0 && selection.length < groups.length} onchange={toggleAll} /></span>
      <span class="who">{groupBy === 'sender' ? 'Sender' : groupBy === 'domain' ? 'Domain' : 'List'}</span>
      <span class="num">Messages</span>
      <span class="num">Unread</span>
      <span class="num">Size</span>
      <span class="date">Latest</span>
      <span class="tags"></span>
      <span class="open"></span>
    </div>
    {#if groups.length === 0 && !loading}
      <EmptyState title="No senders match">
        <p>{session.current?.last_synced_at ? 'Try a different filter.' : 'The first sync is still running, results appear as messages arrive.'}</p>
      </EmptyState>
    {:else}
      <VirtualList items={groups} key={(group) => group.key} {resetKey} onEndReached={() => !exhausted && load(false)}>
        {#snippet row(group: Group)}
          {@const badge = unsubscribeLabel(group)}
          <div class="line" class:selected={!!selected[group.key]}>
            <span class="check"><input type="checkbox" aria-label="Select {group.key}" checked={!!selected[group.key]} onchange={() => toggle(group)} /></span>
            <span class="who">
              {#if groupBy === 'sender'}
                <span class="name truncate">{group.name ?? group.key}</span>
                <span class="key truncate muted">{group.key}</span>
              {:else}
                <span class="name truncate">{group.key}</span>
                <span class="key truncate muted">{group.sender_count === 1 ? (group.name ?? '1 sender') : `${formatNumber(group.sender_count)} senders`}</span>
              {/if}
            </span>
            <span class="num strong">{formatNumber(group.message_count)}</span>
            <span class="num">{group.unread_count ? formatNumber(group.unread_count) : ''}</span>
            <span class="num muted">{formatBytes(group.total_size)}</span>
            <span class="date muted">{formatDate(group.last_date)}</span>
            <span class="tags">{#if badge}<Badge tone={badge.tone}>{badge.text}</Badge>{/if}</span>
            <span class="open"><Button size="sm" variant="ghost" onclick={() => router.go('messages', { q: queryFor(group) })}>View</Button></span>
          </div>
        {/snippet}
      </VirtualList>
    {/if}
  </div>
</section>

<UnsubscribeDialog bind:open={unsubscribeOpen} {accountId} senders={selection} ondone={() => ((selected = {}), load(true))} />

<style>
  .sort {
    width: auto;
  }
  .table {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .head,
  .line {
    display: grid;
    grid-template-columns: 40px minmax(200px, 1fr) 90px 80px 90px 110px 130px 70px;
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
  .who {
    display: flex;
    flex-direction: column;
    min-width: 0;
    line-height: 1.25;
  }
  .name {
    font-weight: 550;
  }
  .key {
    font-size: var(--text-xs);
  }
  .num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .strong {
    font-weight: 600;
  }
  .date {
    text-align: right;
    font-size: var(--text-sm);
  }
  .tags {
    display: flex;
    justify-content: flex-end;
  }
  .open {
    display: flex;
    justify-content: flex-end;
  }
</style>
