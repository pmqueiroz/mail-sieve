<script lang="ts">
  import { api, errorMessage } from '../api';
  import { router, routes } from '../router.svelte';
  import { session } from '../session.svelte';
  import { runningSync, syncDot, syncStatus } from '../sync';
  import { toasts } from '../toasts.svelte';
  import { ui } from '../ui.svelte';
  import PluginSlot from '../plugins/PluginSlot.svelte';
  import Avatar from './Avatar.svelte';
  import Button from './Button.svelte';
  import Chip from './Chip.svelte';
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';

  let menu = $state(false);

  const title = $derived(routes.find((entry) => entry.route === router.route)?.label ?? '');
  const account = $derived(session.current);
  const status = $derived(account ? syncStatus(account, runningSync(session.jobs, account.id)) : null);

  async function syncNow() {
    if (!account) return;
    try {
      const { job_id } = await api.post<{ job_id: number | null }>(`/accounts/${account.id}/sync`);
      if (job_id === null) toasts.show('A sync is already running.');
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }
</script>

<header class="flex flex-wrap items-center gap-8 px-8 py-7 wide:px-18 wide:pt-12 wide:pb-4">
  <div class="grid size-16 place-items-center rounded-logo-sm bg-btn text-btn-ink wide:hidden"><Icon name="logo" size={18} /></div>
  <h1 class="m-0 text-xl font-semibold tracking-tight wide:text-2xl">{title}</h1>
  <div class="flex-1"></div>
  {#if status}
    <button
      type="button"
      class="flex h-18 items-center gap-4 rounded-full border border-line bg-surface pr-7 pl-5 text-sm text-ink2"
      onclick={() => router.go('accounts')}>
      <span class="size-4 rounded-full {syncDot[status.tone]}"></span>
      <span class="wide:hidden">{status.state === 'syncing' ? status.label : status.short}</span>
      <span class="hidden wide:inline">{status.label}</span>
    </button>
    <span class="hidden wide:contents"><Button size="2xl" variant="primary" onclick={syncNow}>Sync now</Button></span>
  {/if}
  <PluginSlot name="topbar" context={{}} />
  <div class="relative">
    <button type="button" class="flex items-center gap-4 border-0 bg-transparent p-0" aria-expanded={menu} onclick={() => (menu = !menu)}>
      <Avatar label={account?.label ?? '+'} seed={account?.id ?? 0} size="xl" round />
      {#if account}
        <span class="hidden flex-col items-start leading-tight wide:flex">
          <span class="text-sm font-semibold">{account.label}</span>
          <span class="text-xs text-ink3">{account.email}</span>
        </span>
      {/if}
      <Icon name="chevron" size={14} class="text-ink2" />
    </button>
    <Popover bind:open={menu} class="top-24 right-0 box-content w-140">
      {#each session.accounts as candidate (candidate.id)}
        {@const candidateStatus = syncStatus(candidate, runningSync(session.jobs, candidate.id))}
        <button
          type="button"
          class="flex w-full items-center gap-5 rounded-md border-0 p-5 text-left {candidate.id === session.accountId ? 'bg-chip' : 'bg-transparent'}"
          onclick={() => (session.selectAccount(candidate.id), (menu = false))}>
          <Avatar label={candidate.label} seed={candidate.id} size="md" />
          <span class="flex min-w-0 flex-1 flex-col leading-snug">
            <span class="text-sm font-semibold">{candidate.label}</span>
            <span class="truncate text-xs text-ink3">{candidate.email}</span>
          </span>
          {#if candidateStatus.state !== 'synced'}
            <span class="text-tiny font-semibold {candidateStatus.state === 'error' ? 'text-bad' : 'text-ink2'}">{candidateStatus.short}</span>
          {/if}
        </button>
      {/each}
      {#if session.accounts.length > 0}<div class="mx-2 my-3 h-px bg-line"></div>{/if}
      <button type="button" class="w-full rounded-md border-0 bg-transparent p-5 text-left text-sm font-semibold" onclick={() => ((menu = false), (ui.connectOpen = true))}>+ Connect a mailbox</button>
    </Popover>
  </div>
</header>

<nav class="flex gap-3 overflow-x-auto border-b border-line px-8 pb-6 wide:hidden">
  {#each routes as entry (entry.route)}
    <Chip active={router.route === entry.route} tall onclick={() => router.go(entry.route)}>{entry.label}</Chip>
  {/each}
</nav>
