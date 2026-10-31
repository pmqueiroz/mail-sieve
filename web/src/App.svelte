<script lang="ts">
  import { errorMessage } from './lib/api';
  import Button from './lib/components/Button.svelte';
  import EmptyState from './lib/components/EmptyState.svelte';
  import ProgressBar from './lib/components/ProgressBar.svelte';
  import Toasts from './lib/components/Toasts.svelte';
  import { formatNumber, formatRelative } from './lib/format';
  import { type Route, router } from './lib/router.svelte';
  import { session } from './lib/session.svelte';
  import Accounts from './routes/Accounts.svelte';
  import Activity from './routes/Activity.svelte';
  import Login from './routes/Login.svelte';
  import Messages from './routes/Messages.svelte';
  import Rules from './routes/Rules.svelte';
  import Senders from './routes/Senders.svelte';

  let bootError = $state<string | null>(null);

  $effect(() => {
    session.boot().catch((error) => (bootError = errorMessage(error)));
  });

  const nav: { route: Route; label: string }[] = [
    { route: 'senders', label: 'Senders' },
    { route: 'messages', label: 'Messages' },
    { route: 'rules', label: 'Rules' },
    { route: 'activity', label: 'Activity' },
    { route: 'accounts', label: 'Accounts' },
  ];

  const currentSync = $derived(
    session.activeJobs.find((job) => job.kind === 'sync' && job.account_id === session.accountId) ?? null,
  );
</script>

{#if bootError}
  <EmptyState title="mail-sieve is unreachable"><p>{bootError}</p></EmptyState>
{:else if !session.status}
  <div class="loading" aria-busy="true"></div>
{:else if !session.status.authenticated}
  <Login configured={session.status.configured} />
{:else}
  <div class="shell">
    <aside class="sidebar">
      <div class="brand">
        <img src="/favicon.svg" alt="" width="26" height="26" />
        <span>mail-sieve</span>
      </div>

      {#if session.accounts.length > 0}
        <label class="account-picker">
          <span class="muted small">Account</span>
          <select value={session.accountId} onchange={(event) => session.selectAccount(Number(event.currentTarget.value))}>
            {#each session.accounts as account (account.id)}
              <option value={account.id}>{account.label}</option>
            {/each}
          </select>
        </label>
      {/if}

      <nav>
        {#each nav as item (item.route)}
          <a href="#/{item.route}" class:active={router.route === item.route} aria-current={router.route === item.route ? 'page' : undefined}>
            {item.label}
            {#if item.route === 'activity' && session.activeJobs.length > 0}
              <span class="count">{session.activeJobs.length}</span>
            {/if}
          </a>
        {/each}
      </nav>

      <div class="footer">
        {#if session.current}
          <div class="sync-status">
            {#if currentSync}
              <span class="small">Syncing {formatNumber(currentSync.done)}{currentSync.total ? ` / ${formatNumber(currentSync.total)}` : ''}</span>
              <ProgressBar done={currentSync.done} total={currentSync.total} indeterminate />
            {:else}
              <span class="small muted">{formatNumber(session.current.message_count)} messages · synced {formatRelative(session.current.last_synced_at)}</span>
            {/if}
          </div>
        {/if}
        <Button size="sm" variant="ghost" onclick={() => session.logout()}>Sign out</Button>
      </div>
    </aside>

    <main class="content">
      {#if router.route === 'accounts' || router.route === 'activity'}
        {#if router.route === 'accounts'}<Accounts />{:else}<Activity />{/if}
      {:else if session.accountId === null}
        <EmptyState title="No account connected">
          <p>Add an IMAP account to start sorting your mail.</p>
          <Button variant="primary" onclick={() => router.go('accounts')}>Add account</Button>
        </EmptyState>
      {:else}
        {#key session.accountId}
          {#if router.route === 'senders'}
            <Senders accountId={session.accountId} />
          {:else if router.route === 'messages'}
            <Messages accountId={session.accountId} />
          {:else}
            <Rules accountId={session.accountId} />
          {/if}
        {/key}
      {/if}
    </main>
  </div>
{/if}

<Toasts />

<style>
  .loading {
    height: 100%;
  }
  .shell {
    display: grid;
    grid-template-columns: var(--sidebar-width) 1fr;
    height: 100%;
  }
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4);
    border-right: 1px solid var(--color-border);
    background: var(--color-surface);
    min-height: 0;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-weight: 700;
    font-size: var(--text-lg);
  }
  .account-picker {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  nav a {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    text-decoration: none;
    font-weight: 550;
  }
  nav a:hover {
    background: var(--color-surface-muted);
  }
  nav a.active {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }
  .count {
    min-width: 20px;
    padding: 0 6px;
    border-radius: 999px;
    background: var(--color-accent);
    color: var(--color-accent-text);
    font-size: var(--text-xs);
    text-align: center;
  }
  .footer {
    margin-top: auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    align-items: stretch;
  }
  .sync-status {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }
  .small {
    font-size: var(--text-xs);
  }
  .content {
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }
  @media (max-width: 760px) {
    .shell {
      grid-template-columns: 1fr;
      grid-template-rows: auto 1fr;
    }
    .sidebar {
      flex-direction: row;
      flex-wrap: wrap;
      align-items: center;
      border-right: none;
      border-bottom: 1px solid var(--color-border);
    }
    nav {
      flex-direction: row;
      flex-wrap: wrap;
    }
    .footer {
      margin-top: 0;
    }
  }
</style>
