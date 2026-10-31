<script lang="ts">
  import { api, errorMessage } from '../lib/api';
  import Button from '../lib/components/Button.svelte';
  import Icon from '../lib/components/Icon.svelte';
  import PasswordField from '../lib/components/PasswordField.svelte';
  import { session } from '../lib/session.svelte';

  let { configured }: { configured: boolean } = $props();

  let password = $state('');
  let confirmation = $state('');
  let passwordError = $state<string | null>(null);
  let confirmationError = $state<string | null>(null);
  let busy = $state(false);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!password) {
      passwordError = configured ? 'Enter your password.' : 'Choose a password.';
      return;
    }
    if (!configured && password !== confirmation) {
      confirmationError = 'The passwords do not match.';
      return;
    }
    busy = true;
    try {
      await api.post(configured ? '/auth/login' : '/auth/setup', { password });
      await session.boot();
    } catch (reason) {
      passwordError = configured && reason instanceof Error && 'status' in reason && reason.status === 401 ? 'Incorrect password.' : errorMessage(reason);
    } finally {
      busy = false;
    }
  }
</script>

<main class="flex min-h-screen flex-col items-center justify-center gap-18 bg-page-glow px-10 py-24">
  <div class="flex items-center gap-7">
    <div class="grid size-24 place-items-center rounded-logo-lg bg-btn text-btn-ink"><Icon name="logo" size={28} /></div>
    <span class="text-wordmark tracking-wordmark"><span class="text-ink2">mail</span><span class="font-bold">sieve</span></span>
  </div>
  <form class="flex w-full max-w-190 flex-col gap-10 rounded-4xl border border-line bg-surface p-14 shadow-card" novalidate onsubmit={submit}>
    <div class="flex flex-col gap-2">
      <h1 class="m-0 text-xl font-semibold tracking-tight">{configured ? 'Welcome back' : 'Welcome'}</h1>
      <p class="m-0 text-base text-ink2">{configured ? 'Sign in to your mailbox control panel.' : 'Choose an admin password to finish setup.'}</p>
    </div>
    <PasswordField
      label="Password"
      placeholder={configured ? 'Enter password' : 'Choose a password'}
      autocomplete={configured ? 'current-password' : 'new-password'}
      hint={configured ? undefined : 'At least 10 characters.'}
      error={passwordError}
      bind:value={password}
      oninput={() => (passwordError = null)}
      autofocus />
    {#if !configured}
      <PasswordField label="Confirm password" placeholder="Repeat password" autocomplete="new-password" error={confirmationError} bind:value={confirmation} oninput={() => (confirmationError = null)} />
    {/if}
    <Button size="block" variant="primary" class="hover:bg-btn-hover" type="submit" {busy}>
      {#if configured}{busy ? 'Signing in…' : 'Sign in'}{:else}{busy ? 'Creating…' : 'Create admin password'}{/if}
    </Button>
  </form>
  <span class="text-xs text-ink3">Self-hosted · v{__APP_VERSION__}</span>
</main>
