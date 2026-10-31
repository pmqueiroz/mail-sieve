<script lang="ts">
  import { api, errorMessage } from '../lib/api';
  import Avatar from '../lib/components/Avatar.svelte';
  import Button from '../lib/components/Button.svelte';
  import Checkbox from '../lib/components/Checkbox.svelte';
  import ConfirmDialog from '../lib/components/ConfirmDialog.svelte';
  import { providerName } from '../lib/components/ConnectDialog.svelte';
  import Field from '../lib/components/Field.svelte';
  import Modal from '../lib/components/Modal.svelte';
  import Pill, { type PillTone } from '../lib/components/Pill.svelte';
  import ProgressBar from '../lib/components/ProgressBar.svelte';
  import TextInput from '../lib/components/TextInput.svelte';
  import { formatBytes, formatNumber, formatRelative } from '../lib/format';
  import { isAuthenticationFailure } from '../lib/guides';
  import { router } from '../lib/router.svelte';
  import { session } from '../lib/session.svelte';
  import { runningSync, syncStatus, type SyncStatus } from '../lib/sync';
  import { toasts } from '../lib/toasts.svelte';
  import type { Account, Mailbox } from '../lib/types';
  import { ui } from '../lib/ui.svelte';

  const STATE_PILLS: Record<SyncStatus['state'], [PillTone, string]> = {
    synced: ['good', 'Synced'],
    syncing: ['strong', 'Syncing'],
    error: ['bad', 'Error'],
  };

  let folderAccount = $state<Account | null>(null);
  let folders = $state<Mailbox[]>([]);
  let foldersOpen = $state(false);
  let settingsAccount = $state<Account | null>(null);
  let settings = $state({ label: '', max_connections: 4, sync_interval_minutes: 15, smtp_host: '', smtp_port: 587, password: '' });
  let settingsOpen = $state(false);
  let removing = $state<Account | null>(null);
  let confirmRemove = $state(false);
  let technical = $state<Record<number, boolean>>({});

  $effect(() => {
    const added = router.params.get('added');
    const failure = router.params.get('error');
    if (added) toasts.show('Account connected. The first sync has started.', 'success');
    if (failure) toasts.show(`Sign-in failed: ${failure}`, 'error', 12000);
    if (added || failure) {
      session.refreshAccounts();
      router.go('accounts');
    }
  });

  function explain(account: Account, error: string): { title: string; body: string } {
    const provider = providerName(account.imap_host);
    const named = provider === 'IMAP' ? 'the' : `${provider}'s`;
    if (isAuthenticationFailure(error)) {
      return {
        title: `${provider === 'IMAP' ? 'The server' : provider} rejected the password`,
        body: 'The email and password were not accepted. Providers that need an app password reject the normal one. Open Settings to enter a new password, then try Sync now.',
      };
    }
    if (/refused|unreachable|timed out|dns|resolve|connect/i.test(error)) {
      return {
        title: `Can't reach ${named} mail server`,
        body: `The server refused the connection. Usually the server address or port is wrong, or ${provider === 'IMAP' ? 'the server' : provider} is briefly down. Check Settings, then try Sync now.`,
      };
    }
    return { title: 'The last sync failed', body: 'Something went wrong while syncing. Try Sync now, and check Settings if it keeps failing.' };
  }

  async function sync(account: Account) {
    try {
      const { job_id } = await api.post<{ job_id: number | null }>(`/accounts/${account.id}/sync`);
      toasts.show(job_id === null ? 'A sync is already running.' : 'Sync started.');
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }

  async function openFolders(account: Account) {
    folderAccount = account;
    folders = await api.get<Mailbox[]>(`/accounts/${account.id}/mailboxes`);
    foldersOpen = true;
  }

  async function toggleFolder(folder: Mailbox) {
    try {
      await api.patch(`/mailboxes/${folder.id}`, { sync_enabled: !folder.sync_enabled });
      if (folderAccount) folders = await api.get<Mailbox[]>(`/accounts/${folderAccount.id}/mailboxes`);
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }

  function openSettings(account: Account) {
    settingsAccount = account;
    settings = {
      label: account.label,
      max_connections: account.max_connections,
      sync_interval_minutes: account.sync_interval_minutes,
      smtp_host: account.smtp_host ?? '',
      smtp_port: account.smtp_port ?? 587,
      password: '',
    };
    settingsOpen = true;
  }

  async function saveSettings() {
    if (!settingsAccount) return;
    try {
      await api.patch(`/accounts/${settingsAccount.id}`, {
        ...settings,
        max_connections: Number(settings.max_connections),
        sync_interval_minutes: Number(settings.sync_interval_minutes),
        smtp_port: Number(settings.smtp_port),
        password: settings.password || null,
      });
      settingsOpen = false;
      await session.refreshAccounts();
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }

  async function remove() {
    if (!removing) return;
    await api.delete(`/accounts/${removing.id}`);
    removing = null;
    await session.refreshAccounts();
  }
</script>

<div class="grid grid-cols-[repeat(auto-fill,minmax(--spacing(160),1fr))] gap-8 pt-2">
  {#each session.accounts as account (account.id)}
    {@const status = syncStatus(account, runningSync(session.jobs, account.id))}
    {@const [tone, label] = STATE_PILLS[status.state]}
    <article class="flex flex-col gap-8 rounded-3xl border border-line p-10">
      <div class="flex items-center gap-6">
        <Avatar label={account.label} seed={account.id} size="2xl" />
        <span class="flex min-w-0 flex-1 flex-col leading-row">
          <span class="font-semibold">{account.label}</span>
          <span class="truncate text-xs text-ink3">{account.email}</span>
        </span>
        <Pill {tone}>{label}</Pill>
      </div>
      <div class="grid grid-cols-3 gap-4">
        <div><div class="text-xs text-ink3">Indexed</div><div class="text-xl font-semibold tabular-nums">{formatNumber(account.message_count)}</div></div>
        <div><div class="text-xs text-ink3">Size</div><div class="text-xl font-semibold">{formatBytes(account.total_size)}</div></div>
        <div><div class="text-xs text-ink3">Last sync</div><div class="mt-2 text-base font-semibold">{status.state === 'syncing' ? 'Syncing' : formatRelative(account.last_synced_at)}</div></div>
      </div>
      <div class="text-xs text-ink3">{account.provider === 'gmail' ? 'Gmail' : providerName(account.imap_host)} · {account.imap_host}:{account.imap_port}</div>
      {#if status.state === 'syncing'}
        <div class="flex flex-col gap-4">
          <div class="flex text-sm"><span class="flex-1 font-semibold">{status.first ? 'First sync' : 'Syncing'}</span><span class="text-ink2">{status.pct}%</span></div>
          <ProgressBar value={status.pct} tone="sieve" />
          {#if status.first}<div class="text-pretty text-xs text-ink2">Counts keep rising until every header is copied. That is expected; senders are already browsable.</div>{/if}
        </div>
      {/if}
      {#if account.last_error && status.state !== 'syncing'}
        {@const explained = explain(account, account.last_error)}
        <div class="flex flex-col gap-4 rounded-xl bg-bad-bg p-7">
          <div class="font-semibold text-bad">{explained.title}</div>
          <div class="text-pretty text-sm text-ink">{explained.body}</div>
          <button
            type="button"
            class="self-start border-0 bg-transparent p-0 text-xs font-semibold text-ink2 underline"
            onclick={() => (technical[account.id] = !technical[account.id])}>{technical[account.id] ? 'Hide technical details' : 'Show technical details'}</button>
          {#if technical[account.id]}<code class="font-mono text-xs break-words text-ink2">{account.last_error}</code>{/if}
        </div>
      {/if}
      <div class="flex flex-wrap gap-4">
        <Button variant="primary" busy={status.state === 'syncing'} onclick={() => sync(account)}>Sync now</Button>
        <Button onclick={() => openFolders(account)}>Folders</Button>
        <Button onclick={() => openSettings(account)}>Settings</Button>
        <Button variant="muted-text" onclick={() => ((removing = account), (confirmRemove = true))}>Remove</Button>
      </div>
    </article>
  {/each}
  <button
    type="button"
    class="flex min-h-110 flex-col items-center justify-center gap-3 rounded-3xl border-check border-dashed border-line bg-transparent text-ink2"
    onclick={() => (ui.connectOpen = true)}>
    <span class="text-3xl font-light">+</span>
    <span class="font-semibold text-ink">Connect a mailbox</span>
    <span class="text-sm">Gmail, Outlook, iCloud, Yahoo, Fastmail or any IMAP server</span>
  </button>
</div>

<Modal bind:open={foldersOpen} title="Folders for {folderAccount?.label ?? ''}">
  <p class="m-0 leading-normal text-ink2">Only synced folders are indexed. Gmail accounts sync All Mail, which already contains every label, so other Gmail folders are off by default to avoid counting messages twice.</p>
  <ul class="m-0 flex max-h-[55vh] list-none flex-col overflow-auto p-0">
    {#each folders as folder (folder.id)}
      <li class="flex min-h-26 items-center gap-6 border-b border-line">
        <Checkbox label="Sync {folder.display_name}" checked={folder.sync_enabled} onchange={() => toggleFolder(folder)} />
        <span class="min-w-0 flex-1 truncate font-semibold">{folder.display_name}</span>
        {#if folder.special_use}<Pill size="tag">{folder.special_use.replace('\\', '')}</Pill>{/if}
        <span class="text-xs text-ink3">{folder.sync_enabled ? `${formatNumber(folder.message_count)} indexed` : 'not synced'}</span>
      </li>
    {/each}
  </ul>
  {#snippet footer()}
    <Button size="dialog" variant="primary" onclick={() => (foldersOpen = false)}>Done</Button>
  {/snippet}
</Modal>

<Modal bind:open={settingsOpen} title="Settings for {settingsAccount?.label ?? ''}">
  <Field label="Label"><TextInput bind:value={settings.label} /></Field>
  <Field label="Parallel connections" hint="More connections sync faster. Gmail allows up to 15 per account.">
    <TextInput type="number" min="1" max="15" bind:value={settings.max_connections} />
  </Field>
  <Field label="Sync every (minutes)" hint="IDLE push also triggers a sync when new mail arrives.">
    <TextInput type="number" min="1" bind:value={settings.sync_interval_minutes} />
  </Field>
  <div class="grid grid-cols-[1fr_--spacing(48)] gap-6">
    <Field label="SMTP server"><TextInput bind:value={settings.smtp_host} /></Field>
    <Field label="Port"><TextInput type="number" bind:value={settings.smtp_port} /></Field>
  </div>
  {#if settingsAccount?.auth_kind === 'password'}
    <Field label="New password" hint="Leave empty to keep the current one"><TextInput type="password" autocomplete="new-password" bind:value={settings.password} /></Field>
  {/if}
  {#snippet footer()}
    <Button size="dialog" onclick={() => (settingsOpen = false)}>Cancel</Button>
    <Button size="dialog" variant="primary" onclick={saveSettings}>Save</Button>
  {/snippet}
</Modal>

<ConfirmDialog
  bind:open={confirmRemove}
  title="Remove account?"
  message="This removes {removing?.email ?? ''} and its local index from mailsieve. Nothing is deleted on the mail server."
  confirmLabel="Remove account"
  danger
  onconfirm={remove} />
