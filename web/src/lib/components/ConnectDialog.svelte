<script lang="ts" module>
  export type Preset = {
    id: string;
    label: string;
    imap: string;
    smtp: string;
    smtpPort: number;
    appPassword: boolean;
    tip: string;
    passwordLabel: string;
  };

  export const presets: Preset[] = [
    { id: 'gmail', label: 'Gmail', imap: 'imap.gmail.com', smtp: 'smtp.gmail.com', smtpPort: 587, appPassword: true, tip: 'Gmail needs an app password, not your normal one. Turn on 2-step verification first, then create one.', passwordLabel: 'App password' },
    { id: 'outlook', label: 'Outlook', imap: 'outlook.office365.com', smtp: 'smtp.office365.com', smtpPort: 587, appPassword: false, tip: 'Outlook works with your normal password if IMAP is turned on in Outlook settings.', passwordLabel: 'Password' },
    { id: 'icloud', label: 'iCloud', imap: 'imap.mail.me.com', smtp: 'smtp.mail.me.com', smtpPort: 587, appPassword: true, tip: 'iCloud needs an app-specific password from appleid.apple.com.', passwordLabel: 'App-specific password' },
    { id: 'yahoo', label: 'Yahoo', imap: 'imap.mail.yahoo.com', smtp: 'smtp.mail.yahoo.com', smtpPort: 465, appPassword: true, tip: 'Yahoo needs an app password from Account security.', passwordLabel: 'App password' },
    { id: 'fastmail', label: 'Fastmail', imap: 'imap.fastmail.com', smtp: 'smtp.fastmail.com', smtpPort: 465, appPassword: true, tip: 'Fastmail needs an app password with IMAP access.', passwordLabel: 'App password' },
    { id: 'custom', label: 'Other', imap: '', smtp: '', smtpPort: 587, appPassword: false, tip: 'Enter your server details. Ask your provider for the IMAP host and port.', passwordLabel: 'Password' },
  ];

  export function providerName(host: string): string {
    return presets.find((preset) => preset.imap === host)?.label ?? 'IMAP';
  }
</script>

<script lang="ts">
  import { api, errorMessage } from '../api';
  import { guides, isAuthenticationFailure } from '../guides';
  import { session } from '../session.svelte';
  import { toasts } from '../toasts.svelte';
  import type { Account } from '../types';
  import { ui } from '../ui.svelte';
  import Button from './Button.svelte';
  import Chip from './Chip.svelte';
  import Field from './Field.svelte';
  import GuideModal from './GuideModal.svelte';
  import Modal from './Modal.svelte';
  import TextInput from './TextInput.svelte';

  const IMAPS_PORT = 993;

  const blank = (preset: Preset) => ({
    preset: preset.id,
    email: '',
    username: '',
    password: '',
    imap_host: preset.imap,
    imap_port: IMAPS_PORT,
    security: 'tls',
    smtp_host: preset.smtp,
    smtp_port: preset.smtpPort,
  });

  let form = $state(blank(presets[0]!));
  let saving = $state(false);
  let guideOpen = $state(false);
  let guideNotice = $state<string | null>(null);

  const preset = $derived(presets.find((candidate) => candidate.id === form.preset) ?? presets[0]!);
  const guide = $derived(guides[form.preset] ?? guides.custom!);
  const oauth = $derived(session.status?.oauth);

  function choose(next: Preset) {
    form = { ...blank(next), email: form.email, password: form.password };
  }

  function openGuide(notice: string | null = null) {
    guideNotice = notice;
    guideOpen = true;
  }

  async function signIn(provider: 'google' | 'microsoft') {
    try {
      const { url } = await api.get<{ url: string }>(`/oauth/${provider}/start`);
      window.location.href = url;
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }

  async function connect(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    try {
      const account = await api.post<Account>('/accounts', {
        email: form.email,
        username: form.username,
        password: preset.appPassword ? form.password.replace(/\s+/g, '') : form.password,
        imap_host: form.imap_host,
        imap_port: Number(form.imap_port),
        security: form.security,
        smtp_host: form.smtp_host,
        smtp_port: Number(form.smtp_port),
      });
      ui.connectOpen = false;
      form = blank(preset);
      await session.refreshAccounts();
      session.selectAccount(account.id);
      toasts.show(`Connected ${account.email}. The first sync has started.`, 'success');
    } catch (error) {
      const message = errorMessage(error);
      if (isAuthenticationFailure(message)) {
        openGuide(
          preset.appPassword
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
</script>

<Modal bind:open={ui.connectOpen} title="Connect a mailbox">
  {#if oauth?.google || oauth?.microsoft}
    <div class="flex flex-wrap gap-4">
      {#if oauth.google}<Button size="dialog" onclick={() => signIn('google')}>Sign in with Google</Button>{/if}
      {#if oauth.microsoft}<Button size="dialog" onclick={() => signIn('microsoft')}>Sign in with Microsoft</Button>{/if}
    </div>
    <div class="text-sm text-ink2">or connect with a password</div>
  {/if}
  <div class="grid grid-cols-3 gap-4" role="radiogroup" aria-label="Provider">
    {#each presets as option (option.id)}
      <button
        type="button"
        role="radio"
        aria-checked={option.id === form.preset}
        class="h-24 rounded-xl border-check bg-transparent text-sm font-semibold {option.id === form.preset ? 'border-ink' : 'border-line'}"
        onclick={() => choose(option)}>{option.label}</button>
    {/each}
  </div>
  <div class="rounded-xl bg-warn-bg px-7 py-6 text-sm leading-normal text-ink">
    {preset.tip}
    <button type="button" class="border-0 bg-transparent p-0 font-semibold underline underline-offset-2" onclick={() => openGuide()}>Step-by-step guide</button>
  </div>
  <form id="connect-mailbox" class="flex flex-col gap-8" onsubmit={connect}>
    <Field label="Email address"><TextInput type="email" required placeholder="you@example.com" autocomplete="email" bind:value={form.email} /></Field>
    <Field label={preset.passwordLabel}><TextInput type="password" required placeholder="••••••••••••" autocomplete="current-password" bind:value={form.password} /></Field>
    {#if preset.id === 'custom'}
      <Field label="Username" hint="Leave empty to use the email address"><TextInput bind:value={form.username} autocomplete="username" /></Field>
      <div class="grid grid-cols-[1fr_--spacing(48)] gap-6">
        <Field label="IMAP server"><TextInput required placeholder="imap.example.com" bind:value={form.imap_host} /></Field>
        <Field label="Port"><TextInput type="number" bind:value={form.imap_port} /></Field>
      </div>
      <Field label="Security">
        <div class="flex gap-3">
          {#each [['tls', 'TLS'], ['plain', 'None, local only']] as [value, label] (value)}
            <Chip active={form.security === value} muted onclick={() => (form.security = value!)}>{label}</Chip>
          {/each}
        </div>
      </Field>
      <div class="grid grid-cols-[1fr_--spacing(48)] gap-6">
        <Field label="SMTP server" hint="Used only for unsubscribe emails"><TextInput bind:value={form.smtp_host} /></Field>
        <Field label="Port"><TextInput type="number" bind:value={form.smtp_port} /></Field>
      </div>
    {/if}
  </form>
  {#snippet footer()}
    <Button size="dialog" onclick={() => (ui.connectOpen = false)}>Cancel</Button>
    <Button size="dialog" variant="primary" type="submit" form="connect-mailbox" busy={saving}>Connect and sync</Button>
  {/snippet}
</Modal>

<GuideModal bind:open={guideOpen} {guide} notice={guideNotice} />
