<script lang="ts">
  import { api, errorMessage, withParams } from '../lib/api';
  import Avatar from '../lib/components/Avatar.svelte';
  import Button from '../lib/components/Button.svelte';
  import Icon from '../lib/components/Icon.svelte';
  import Pill from '../lib/components/Pill.svelte';
  import ProgressBar from '../lib/components/ProgressBar.svelte';
  import Segmented from '../lib/components/Segmented.svelte';
  import { formatBytes, formatNumber, formatShortDate, percent } from '../lib/format';
  import { router } from '../lib/router.svelte';
  import { session } from '../lib/session.svelte';
  import { runningSync, syncStatus } from '../lib/sync';
  import { toasts } from '../lib/toasts.svelte';
  import type { Category, Group, Overview } from '../lib/types';
  import { unsubscribeBadge } from '../lib/unsubscribe';

  let { accountId }: { accountId: number } = $props();

  const PILES = 5;
  const MIX: { category: Category; label: string }[] = [
    { category: 'newsletters', label: 'Newsletters' },
    { category: 'notifications', label: 'Notifications' },
    { category: 'promotions', label: 'Promotions' },
    { category: 'receipts', label: 'Receipts' },
    { category: 'social', label: 'Social' },
    { category: 'people', label: 'People' },
  ];
  const MIX_CENTER = 80;
  const MIX_ORBIT = 50;
  const PETAL_HALF = 24;

  const toHalfUnit = (units: number) => Math.round(units * 2) / 2;

  let overview = $state<Overview | null>(null);
  let piles = $state<Group[]>([]);
  let pileBy = $state<'count' | 'size'>('count');

  const account = $derived(session.current);
  const status = $derived(account ? syncStatus(account, runningSync(session.jobs, account.id)) : null);
  const neverOpenedPct = $derived(overview ? percent(overview.never_opened.messages, overview.total) : 0);
  const mixTotal = $derived(overview?.categories.reduce((sum, entry) => sum + entry.count, 0) ?? 0);
  const petals = $derived(
    MIX.map((entry, index) => {
      const angle = ((index * 360) / MIX.length - 90) * (Math.PI / 180);
      const count = overview?.categories.find((candidate) => candidate.category === entry.category)?.count ?? 0;
      return {
        ...entry,
        pct: percent(count, mixTotal),
        left: toHalfUnit(MIX_CENTER + MIX_ORBIT * Math.cos(angle) - PETAL_HALF),
        top: toHalfUnit(MIX_CENTER + MIX_ORBIT * Math.sin(angle) - PETAL_HALF),
      };
    }),
  );

  $effect(() => {
    const id = accountId;
    void session.dataVersion;
    api
      .get<Overview>(`/accounts/${id}/overview`)
      .then((result) => (overview = result))
      .catch((error) => toasts.show(errorMessage(error), 'error'));
  });

  $effect(() => {
    const id = accountId;
    const sort = pileBy;
    void session.dataVersion;
    api
      .get<Group[]>(withParams(`/accounts/${id}/groups`, { group: 'sender', sort, limit: PILES }))
      .then((result) => (piles = result))
      .catch((error) => toasts.show(errorMessage(error), 'error'));
  });

  const stats = $derived(
    overview
      ? [
          { label: 'Unread', value: formatNumber(overview.unread), icon: '↑', iconTone: 'bg-bad', note: `+${formatNumber(overview.unread_week)} this week`, noteTone: 'text-bad' },
          { label: 'From mailing lists', value: formatNumber(overview.list_messages), icon: '≡', iconTone: 'bg-ink3', note: `${percent(overview.list_messages, overview.total)}% of your mail`, noteTone: 'text-ink2' },
          { label: 'Space you can recover', value: formatBytes(overview.never_opened.bytes), icon: '↓', iconTone: 'bg-good', note: `${formatBytes(overview.freed_week_bytes)} freed this week`, noteTone: 'text-good' },
        ]
      : [],
  );
</script>

{#if status?.state === 'syncing' && status.first}
  <div class="my-4 flex flex-wrap items-center gap-7 rounded-xl border border-line bg-surface2 px-9 py-7">
    <span class="font-semibold">First sync {status.pct}% done</span>
    <span class="min-w-110 flex-1 text-pretty text-ink2">Counts below keep rising until every header is copied. That is expected; you can start cleaning now.</span>
    <ProgressBar value={status.pct} class="w-80" />
  </div>
{/if}

<section class="flex flex-wrap items-end gap-24 border-b border-line pt-12 pb-16">
  <div class="flex-none">
    <div class="text-hero-sm font-semibold tracking-hero wide:text-hero">{formatNumber(overview?.total ?? 0)}</div>
    <div class="mt-5 text-ink2">messages in your mailbox</div>
  </div>
  <div class="min-w-130 flex-1">
    <div class="mb-7 flex flex-wrap items-baseline gap-4">
      <span class="text-xl font-bold text-good">{neverOpenedPct}%</span>
      <span class="text-ink2">comes from senders you never open. Clear it in a few clicks.</span>
    </div>
    <div class="relative h-22">
      <div class="absolute top-9 left-0 h-17 rounded-md bg-sieve opacity-55 blur-glow" style:width="{neverOpenedPct}%"></div>
      <div class="absolute inset-0 rounded-md bg-ticks opacity-90"></div>
      <div class="absolute inset-y-0 left-0 flex items-center rounded-md bg-sieve pl-6 text-xs font-semibold text-avatar-ink" style:width="{neverOpenedPct}%">
        {formatNumber(overview?.never_opened.messages ?? 0)}
      </div>
      <div class="absolute -inset-y-2 ml-1 w-1.5 rounded-full bg-ink" style:left="{neverOpenedPct}%"></div>
    </div>
    <div class="mt-6 flex justify-between text-xs text-ink3"><span>Never opened</span><span>Opened at least once</span></div>
  </div>
</section>

<section class="grid grid-cols-[repeat(auto-fit,minmax(--spacing(110),1fr))] border-b border-line">
  {#each stats as stat (stat.label)}
    <div class="flex flex-col gap-6 py-12 pr-12">
      <span class="font-medium">{stat.label}</span>
      <div class="flex flex-wrap items-center gap-5">
        <span class="grid size-10 place-items-center rounded-check text-xs font-bold text-white {stat.iconTone}">{stat.icon}</span>
        <span class="text-3xl font-semibold tracking-tight">{stat.value}</span>
        <span class="ml-6 text-xs {stat.noteTone}">{stat.note}</span>
      </div>
    </div>
  {/each}
</section>

<section class="grid grid-cols-[minmax(0,1fr)] board:grid-cols-[minmax(0,1fr)_--spacing(190)]">
  <div class="min-w-0 pt-14 pr-14 pb-4 board:border-r board:border-line">
    <div class="mb-6 flex flex-wrap items-center gap-8">
      <h2 class="m-0 text-lg font-semibold">Biggest piles</h2>
      <span class="text-sm text-ink2"><b class="font-semibold text-ink">{formatNumber(overview?.sender_count ?? 0)} senders</b>, start at the top</span>
      <div class="flex-1"></div>
      <Segmented
        label="Sort piles"
        bind:value={pileBy}
        options={[
          { value: 'count', label: 'By count' },
          { value: 'size', label: 'By size' },
        ]} />
    </div>
    {#each piles as pile (pile.key)}
      {@const badge = unsubscribeBadge(pile)}
      {@const unreadPct = percent(pile.unread_count, pile.message_count)}
      <div class="grid h-32 grid-cols-[minmax(0,1fr)_auto_auto] items-center gap-8 wide:grid-cols-[minmax(--spacing(80),1.6fr)_--spacing(65)_--spacing(45)_--spacing(60)_auto]">
        <div class="flex min-w-0 items-center gap-6">
          <Avatar label={pile.name ?? pile.key} seed={pile.key} size="xl" />
          <span class="flex min-w-0 flex-col leading-row">
            <span class="truncate font-semibold">{pile.name ?? pile.key}</span>
            <span class="text-xs text-ink3">{formatNumber(pile.message_count)} messages, {formatBytes(pile.total_size)}</span>
          </span>
        </div>
        <span class="hidden justify-self-start wide:block {badge ? '' : 'invisible'}">
          <Pill tone={badge?.tone} size="badge-lg"><Icon name="bars" size={12} />{badge?.label ?? 'None'}</Pill>
        </span>
        <span class="hidden items-center gap-4 text-sm text-ink2 wide:flex"><Icon name="tile" size={14} class="text-ink3" />{formatShortDate(pile.last_date)}</span>
        <span class="flex items-center gap-5 text-sm font-medium">
          <span class="grid size-12 place-items-center rounded-full" style:background="conic-gradient(var(--color-ring) {unreadPct}%, var(--color-track) 0)">
            <span class="size-8 rounded-full bg-surface"></span>
          </span>
          {unreadPct}% unread
        </span>
        <Button size="md" onclick={() => router.go('senders', { select: pile.key })}>Review</Button>
      </div>
    {/each}
  </div>
  <div class="min-w-0 pt-14 pb-12 pl-14">
    <div class="mb-4 flex items-center">
      <h2 class="m-0 flex-1 text-lg font-semibold">Inbox mix</h2>
      <span class="text-sm text-ink3">Last 12 months</span>
    </div>
    <div class="relative mx-auto size-160 max-w-full">
      <div class="absolute top-35 left-35 size-90 rounded-full bg-sieve-bloom opacity-85 blur-bloom"></div>
      {#each petals as petal (petal.category)}
        <button
          type="button"
          class="absolute flex size-48 flex-col items-center justify-center gap-1 rounded-petal border border-line bg-surface2 opacity-92 backdrop-blur-glass"
          style:left="calc(var(--spacing) * {petal.left})"
          style:top="calc(var(--spacing) * {petal.top})"
          onclick={() => router.go('senders', { q: `category:${petal.category}` })}>
          <span class="text-md font-bold">{petal.pct}%</span>
          <span class="text-tiny text-ink2">{petal.label}</span>
        </button>
      {/each}
      <div class="absolute top-71 left-71 grid size-18 place-items-center rounded-full bg-surface shadow-knob"><Icon name="envelope" size={16} class="text-ink" /></div>
    </div>
    <div class="mt-4 text-center text-xs text-ink3">Tap a category to see its senders</div>
  </div>
</section>
