<script lang="ts">
  import { errorMessage } from './lib/api';
  import Button from './lib/components/Button.svelte';
  import ConnectDialog from './lib/components/ConnectDialog.svelte';
  import EmptyState from './lib/components/EmptyState.svelte';
  import NavRail from './lib/components/NavRail.svelte';
  import Toasts from './lib/components/Toasts.svelte';
  import TopBar from './lib/components/TopBar.svelte';
  import { router } from './lib/router.svelte';
  import { session } from './lib/session.svelte';
  import { ui } from './lib/ui.svelte';
  import Accounts from './routes/Accounts.svelte';
  import Activity from './routes/Activity.svelte';
  import Login from './routes/Login.svelte';
  import Messages from './routes/Messages.svelte';
  import Overview from './routes/Overview.svelte';
  import Rules from './routes/Rules.svelte';
  import Senders from './routes/Senders.svelte';

  let bootError = $state<string | null>(null);

  $effect(() => {
    session.boot().catch((error) => (bootError = errorMessage(error)));
  });
</script>

{#if bootError}
  <EmptyState title="mailsieve is unreachable"><p class="m-0">{bootError}</p></EmptyState>
{:else if !session.status}
  <div class="min-h-screen" aria-busy="true"></div>
{:else if !session.status.authenticated}
  <Login configured={session.status.configured} />
{:else}
  <div class="min-h-screen bg-surface text-ink">
    <div class="relative flex min-h-screen flex-col wide:flex-row">
      <div class="hidden wide:contents"><NavRail /></div>
      <div class="flex min-w-0 flex-1 flex-col">
        <TopBar />
        <main class="min-w-0 flex-1 p-8 wide:px-18 wide:pt-4 wide:pb-18">
          {#if router.route === 'accounts'}
            <Accounts />
          {:else if router.route === 'activity'}
            <Activity />
          {:else if session.accountId === null}
            <EmptyState title="No mailbox connected">
              <p class="m-0">Connect an IMAP mailbox to start sorting your mail.</p>
              <Button size="2xl" variant="primary" onclick={() => (ui.connectOpen = true)}>Connect a mailbox</Button>
            </EmptyState>
          {:else}
            {#key session.accountId}
              {#if router.route === 'overview'}
                <Overview accountId={session.accountId} />
              {:else if router.route === 'senders'}
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
    </div>
  </div>
  <ConnectDialog />
{/if}

<Toasts />
