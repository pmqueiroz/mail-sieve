<script lang="ts">
  import { api, errorMessage } from '../lib/api';
  import Badge from '../lib/components/Badge.svelte';
  import Button from '../lib/components/Button.svelte';
  import ConfirmDialog from '../lib/components/ConfirmDialog.svelte';
  import Field from '../lib/components/Field.svelte';
  import GuideModal from '../lib/components/GuideModal.svelte';
  import Modal from '../lib/components/Modal.svelte';
  import ProgressBar from '../lib/components/ProgressBar.svelte';
  import { formatBytes, formatNumber, formatRelative } from '../lib/format';
  import { guides, isAuthenticationFailure } from '../lib/guides';
  import { router } from '../lib/router.svelte';
  import { session } from '../lib/session.svelte';
  import { toasts } from '../lib/toasts.svelte';
  import type { Account, Mailbox } from '../lib/types';

  type Preset = { id: string; label: string; imap: string; smtp: string; smtpPort: number; appPassword: boolean };

  const presets: Preset[] = [
    { id: 'gmail', label: 'Gmail', imap: 'imap.gmail.com', smtp: 'smtp.gmail.com', smtpPort: 587, appPassword: true },
    { id: 'outlook', label: 'Outlook / Microsoft 365', imap: 'outlook.office365.com', smtp: 'smtp.office365.com', smtpPort: 587, appPassword: false },
    { id: 'icloud', label: 'iCloud', imap: 'imap.mail.me.com', smtp: 'smtp.mail.me.com', smtpPort: 587, appPassword: true },
    { id: 'yahoo', label: 'Yahoo', imap: 'imap.mail.yahoo.com', smtp: 'smtp.mail.yahoo.com', smtpPort: 465, appPassword: true },
    { id: 'fastmail', label: 'Fastmail', imap: 'imap.fastmail.com', smtp: 'smtp.fastmail.com', smtpPort: 465, appPassword: true },
    { id: 'custom', label: 'Other IMAP server', imap: '', smtp: '', smtpPort: 587, appPassword: false },
  ];

  const blankForm = () => ({
    preset: 'gmail',
    label: '',
    email: '',
    username: '',
    password: '',
    imap_host: 'imap.gmail.com',
    imap_port: 993,
    security: 'tls',
    smtp_host: 'smtp.gmail.com',
    smtp_port: 587,
  });

  let adding = $state(false);
  let form = $state(blankForm());
  let saving = $state(false);
  let folderAccount = $state<Account | null>(null);
  let folders = $state<Mailbox[]>([]);
  let settingsAccount = $state<Account | null>(null);
  let settings = $state({ label: '', max_connections: 4, sync_interval_minutes: 15, smtp_host: '', smtp_port: 587, password: '' });
  let removing = $state<Account | null>(null);
  let confirmRemove = $state(false);
  let foldersOpen = $state(false);
  let settingsOpen = $state(false);
  let guideOpen = $state(false);
  let guideNotice = $state<string | null>(null);

  const preset = $derived(presets.find((candidate) => candidate.id === form.preset));
  const guide = $derived(guides[form.preset] ?? guides.custom!);

  function openGuide(notice: string | null = null) {
    guideNotice = notice;
    guideOpen = true;
  }

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

  function applyPreset() {
    const chosen = presets.find((candidate) => candidate.id === form.preset);
    if (!chosen) return;
    form.imap_host = chosen.imap;
    form.smtp_host = chosen.smtp;
    form.smtp_port = chosen.smtpPort;
    form.imap_port = 993;
    form.security = 'tls';
  }

  async function add(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    try {
      const account = await api.post<Account>('/accounts', {
        label: form.label,
        email: form.email,
        username: form.username,
        password: preset?.appPassword ? form.password.replace(/\s+/g, '') : form.password,
        imap_host: form.imap_host,
        imap_port: Number(form.imap_port),
        security: form.security,
        smtp_host: form.smtp_host,
        smtp_port: Number(form.smtp_port),
      });
      adding = false;
      form = blankForm();
      await session.refreshAccounts();
      session.selectAccount(account.id);
      toasts.show(`Connected ${account.email}. The first sync has started.`, 'success');
    } catch (error) {
      const message = errorMessage(error);
      if (isAuthenticationFailure(message)) {
        openGuide(
          preset?.appPassword
            ? `${preset.label} rejected the email and password. This usually means the normal account password was used instead of an app password. Follow the steps below.`
            : 'The server rejected the email and password. Check them and follow the steps below.',
        );
      } else {
        toasts.show(message, 'error', 10000);
      }
    } finally {
      saving = false;
    }
  }

  async function signIn(provider: 'google' | 'microsoft') {
    try {
      const { url } = await api.get<{ url: string }>(`/oauth/${provider}/start`);
      window.location.href = url;
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
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

  function runningSync(account: Account) {
    return session.activeJobs.find((job) => job.kind === 'sync' && job.account_id === account.id);
  }
</script>

<section class="page">
  <div class="page-header">
    <h1>Accounts</h1>
    <Button variant="primary" onclick={() => (adding = true)}>Add account</Button>
  </div>

  <div class="accounts">
    {#each session.accounts as account (account.id)}
      {@const job = runningSync(account)}
      <article class="account card">
        <div class="head">
          <div>
            <div class="title">
              <strong>{account.label}</strong>
              {#if account.provider === 'gmail'}<Badge tone="accent">Gmail</Badge>{/if}
              {#if account.auth_kind === 'oauth2'}<Badge>OAuth</Badge>{/if}
              {#if account.last_error}<Badge tone="danger" title={account.last_error}>sync error</Badge>{/if}
            </div>
            <span class="muted small">{account.email} · {account.imap_host}:{account.imap_port}</span>
          </div>
          <div class="toolbar">
            <Button size="sm" busy={!!job} onclick={() => sync(account)}>Sync now</Button>
            <Button size="sm" onclick={() => openFolders(account)}>Folders</Button>
            <Button size="sm" onclick={() => openSettings(account)}>Settings</Button>
            <Button size="sm" variant="danger" onclick={() => ((removing = account), (confirmRemove = true))}>Remove</Button>
          </div>
        </div>
        <div class="stats">
          <span><strong>{formatNumber(account.message_count)}</strong> messages indexed</span>
          <span>{formatBytes(account.total_size)}</span>
          <span class="muted">Last sync {formatRelative(account.last_synced_at)}</span>
        </div>
        {#if job}
          <ProgressBar done={job.done} total={job.total} indeterminate />
          <span class="muted small">Syncing {formatNumber(job.done)}{job.total ? ` of ${formatNumber(job.total)}` : ''} headers</span>
        {/if}
        {#if account.last_error}<p class="error small">{account.last_error}</p>{/if}
      </article>
    {:else}
      <div class="card welcome">
        <h2>Connect your first mailbox</h2>
        <p class="muted">mail-sieve copies message headers into a local index so you can sort, filter and clean up thousands of messages in seconds. Message bodies stay on your mail server.</p>
        <Button variant="primary" onclick={() => (adding = true)}>Add account</Button>
      </div>
    {/each}
  </div>
</section>

<Modal bind:open={adding} title="Add account" width={600}>
  {#if session.status?.oauth.google || session.status?.oauth.microsoft}
    <div class="oauth">
      {#if session.status?.oauth.google}<Button onclick={() => signIn('google')}>Sign in with Google</Button>{/if}
      {#if session.status?.oauth.microsoft}<Button onclick={() => signIn('microsoft')}>Sign in with Microsoft</Button>{/if}
    </div>
    <p class="muted divider">or connect with a password</p>
  {/if}
  <button type="button" class="guide-banner" onclick={() => openGuide()}>
    <span class="guide-icon" aria-hidden="true">?</span>
    <span>
      <strong>New here? See how to connect {preset?.label ?? 'your account'}</strong>
      <span class="muted">Step-by-step, takes about two minutes.</span>
    </span>
  </button>
  <form id="add-account" class="grid" onsubmit={add}>
    <Field label="Provider">
      <select bind:value={form.preset} onchange={applyPreset}>
        {#each presets as option (option.id)}<option value={option.id}>{option.label}</option>{/each}
      </select>
    </Field>
    <Field label="Label" hint="Optional display name"><input type="text" bind:value={form.label} /></Field>
    <Field label="Email address"><input type="email" required bind:value={form.email} autocomplete="email" /></Field>
    <Field label="Username" hint="Defaults to the email address"><input type="text" bind:value={form.username} autocomplete="username" /></Field>
    <div class="wide">
      <Field label={preset?.appPassword ? 'App password' : 'Password'} hint={preset?.appPassword ? 'Not your normal password. Use the app password from the guide above.' : undefined}>
        <input type="password" required bind:value={form.password} autocomplete="current-password" />
      </Field>
    </div>
    <Field label="IMAP server"><input type="text" required bind:value={form.imap_host} /></Field>
    <div class="pair">
      <Field label="Port"><input type="number" bind:value={form.imap_port} /></Field>
      <Field label="Security">
        <select bind:value={form.security}>
          <option value="tls">TLS</option>
          <option value="plain">None (local only)</option>
        </select>
      </Field>
    </div>
    <Field label="SMTP server" hint="Used only for mailto unsubscribes"><input type="text" bind:value={form.smtp_host} /></Field>
    <Field label="SMTP port"><input type="number" bind:value={form.smtp_port} /></Field>
  </form>
  {#snippet footer()}
    <Button variant="ghost" onclick={() => (adding = false)}>Cancel</Button>
    <Button variant="primary" type="submit" form="add-account" busy={saving}>Connect</Button>
  {/snippet}
</Modal>

<GuideModal bind:open={guideOpen} {guide} notice={guideNotice} />

<Modal bind:open={foldersOpen} title="Folders for {folderAccount?.label ?? ''}" width={560}>
  <p class="muted">Only synced folders are indexed. Gmail accounts sync All Mail, which already contains every label, so other Gmail folders are off by default to avoid counting messages twice.</p>
  <ul class="folders">
    {#each folders as folder (folder.id)}
      <li>
        <label>
          <input type="checkbox" checked={folder.sync_enabled} onchange={() => toggleFolder(folder)} />
          <span class="truncate">{folder.display_name}</span>
          {#if folder.special_use}<Badge>{folder.special_use.replace('\\', '')}</Badge>{/if}
        </label>
        <span class="muted small">{folder.sync_enabled ? `${formatNumber(folder.message_count)} indexed` : 'not synced'}</span>
      </li>
    {/each}
  </ul>
</Modal>

<Modal bind:open={settingsOpen} title="Settings for {settingsAccount?.label ?? ''}">
  <Field label="Label"><input type="text" bind:value={settings.label} /></Field>
  <Field label="Parallel connections" hint="More connections sync faster. Gmail allows up to 15 per account.">
    <input type="number" min="1" max="15" bind:value={settings.max_connections} />
  </Field>
  <Field label="Sync every (minutes)" hint="IDLE push also triggers a sync when new mail arrives.">
    <input type="number" min="1" bind:value={settings.sync_interval_minutes} />
  </Field>
  <Field label="SMTP server"><input type="text" bind:value={settings.smtp_host} /></Field>
  <Field label="SMTP port"><input type="number" bind:value={settings.smtp_port} /></Field>
  {#if settingsAccount?.auth_kind === 'password'}
    <Field label="New password" hint="Leave empty to keep the current one"><input type="password" bind:value={settings.password} autocomplete="new-password" /></Field>
  {/if}
  {#snippet footer()}
    <Button variant="ghost" onclick={() => (settingsOpen = false)}>Cancel</Button>
    <Button variant="primary" onclick={saveSettings}>Save</Button>
  {/snippet}
</Modal>

<ConfirmDialog
  bind:open={confirmRemove}
  title="Remove account?"
  message="This removes {removing?.email ?? ''} and its local index from mail-sieve. Nothing is deleted on the mail server."
  confirmLabel="Remove account"
  danger
  onconfirm={remove} />

<style>
  .accounts {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    overflow: auto;
  }
  .account {
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }
  .head {
    display: flex;
    justify-content: space-between;
    gap: var(--space-3);
    flex-wrap: wrap;
  }
  .title {
    display: flex;
    gap: var(--space-2);
    align-items: center;
  }
  .stats {
    display: flex;
    gap: var(--space-5);
    flex-wrap: wrap;
  }
  .small {
    font-size: var(--text-xs);
  }
  .error {
    color: var(--color-danger);
    margin: 0;
  }
  .welcome {
    padding: var(--space-6);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    align-items: flex-start;
    max-width: 620px;
  }
  .welcome h2,
  .welcome p {
    margin: 0;
  }
  .guide-banner {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    padding: var(--space-3);
    border: 1px solid var(--color-accent);
    border-radius: var(--radius-sm);
    background: var(--color-accent-soft);
    text-align: left;
    cursor: pointer;
  }
  .guide-banner > span:last-child {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .guide-icon {
    flex-shrink: 0;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--color-accent);
    color: var(--color-accent-text);
    font-weight: 700;
  }
  .oauth {
    display: flex;
    gap: var(--space-2);
  }
  .divider {
    margin: 0;
    font-size: var(--text-sm);
  }
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-3);
  }
  .wide {
    grid-column: 1 / -1;
  }
  .pair {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-2);
  }
  .folders {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    max-height: 55vh;
    overflow: auto;
  }
  .folders li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-3);
  }
  .folders label {
    display: flex;
    gap: var(--space-2);
    align-items: center;
    min-width: 0;
  }
  p {
    margin: 0;
    line-height: 1.5;
  }
</style>
