<script lang="ts">
  import { api, errorMessage } from '../../lib/api';
  import Button from '../../lib/components/Button.svelte';
  import { formatNumber, formatShortDate } from '../../lib/format';
  import { groupQuery } from '../../lib/query';
  import { router } from '../../lib/router.svelte';
  import { session } from '../../lib/session.svelte';
  import { toasts } from '../../lib/toasts.svelte';
  import type { Rule, StillSending } from '../../lib/types';

  const STORAGE_KEY = 'mail-sieve.unsubscribe-audit';

  let { accountId }: { accountId: number | null } = $props();

  let senders = $state<StillSending[]>([]);
  let rules = $state<Rule[]>([]);
  let dismissed = $state<Record<string, number>>(readDismissed());
  let busy = $state<string | null>(null);

  const visible = $derived(senders.filter((sender) => dismissed[dismissKey(sender)] === undefined));

  function readDismissed(): Record<string, number> {
    try {
      return JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}') as Record<string, number>;
    } catch {
      return {};
    }
  }

  function dismissKey(sender: StillSending): string {
    return `${accountId}:${sender.from_addr}:${sender.unsubscribed_at}`;
  }

  $effect(() => {
    const id = accountId;
    void session.dataVersion;
    if (id === null) return;
    let cancelled = false;
    Promise.all([api.get<StillSending[]>(`/accounts/${id}/unsubscribes/still-sending`), api.get<Rule[]>('/rules')])
      .then(([found, all]) => {
        if (cancelled) return;
        senders = found;
        rules = all.filter((rule) => rule.account_id === id);
      })
      .catch((error) => !cancelled && toasts.show(errorMessage(error), 'error'));
    return () => (cancelled = true);
  });

  function hasRule(sender: StillSending): boolean {
    const query = groupQuery('sender', sender.from_addr);
    return rules.some((rule) => rule.query.split(/\s+/).includes(query));
  }

  async function addRule(sender: StillSending) {
    if (accountId === null) return;
    busy = sender.from_addr;
    try {
      const who = sender.name ?? sender.from_addr;
      const rule = await api.post<Rule>('/rules', {
        account_id: accountId,
        name: `Trash ${who} after unsubscribe`,
        query: groupQuery('sender', sender.from_addr),
        action: 'trash',
        run_after_sync: true,
      });
      rules = [...rules, rule];
      toasts.show(`Rule created. Mail from ${who} goes to trash after each sync.`, 'success');
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    } finally {
      busy = null;
    }
  }

  function dismiss(sender: StillSending) {
    dismissed[dismissKey(sender)] = Date.now();
    localStorage.setItem(STORAGE_KEY, JSON.stringify(dismissed));
  }
</script>

{#if visible.length > 0}
  <div class="mt-2 mb-10 flex flex-col gap-6 rounded-3xl border border-line px-10 py-9">
    <div class="flex flex-wrap items-center gap-5">
      <span class="grid size-10 place-items-center rounded-check bg-bad text-xs font-bold text-white">!</span>
      <span class="font-semibold">{visible.length} {visible.length === 1 ? 'sender still mails' : 'senders still mail'} you after unsubscribing</span>
      <span class="text-sm text-ink2">Counted from two days after the unsubscribe, the time senders get to process it.</span>
    </div>
    {#each visible as sender (sender.from_addr)}
      {@const ruled = hasRule(sender)}
      <div class="flex flex-wrap items-center gap-6">
        <span class="flex min-w-100 flex-1 flex-col leading-row">
          <b class="font-semibold">{sender.name ?? sender.from_addr}</b>
          <span class="text-xs text-ink3">
            Unsubscribed {formatShortDate(sender.unsubscribed_at)} · {formatNumber(sender.since_count)}
            {sender.since_count === 1 ? 'message' : 'messages'} since · latest {formatShortDate(sender.last_date)}
          </span>
        </span>
        <Button size="sm" onclick={() => router.go('messages', { q: groupQuery('sender', sender.from_addr) })}>View</Button>
        {#if ruled}
          <span class="text-xs text-ink3">Trash rule active</span>
        {:else}
          <Button size="sm" variant="primary" busy={busy === sender.from_addr} onclick={() => addRule(sender)}>Trash after every sync</Button>
        {/if}
        <Button size="sm" variant="muted-text" onclick={() => dismiss(sender)}>Dismiss</Button>
      </div>
    {/each}
  </div>
{/if}
