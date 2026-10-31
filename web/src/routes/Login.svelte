<script lang="ts">
  import { api, errorMessage } from '../lib/api';
  import Button from '../lib/components/Button.svelte';
  import Field from '../lib/components/Field.svelte';
  import { session } from '../lib/session.svelte';

  let { configured }: { configured: boolean } = $props();

  let password = $state('');
  let confirmation = $state('');
  let error = $state<string | null>(null);
  let busy = $state(false);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    if (!configured && password !== confirmation) {
      error = 'The passwords do not match.';
      return;
    }
    busy = true;
    try {
      await api.post(configured ? '/auth/login' : '/auth/setup', { password });
      await session.boot();
    } catch (reason) {
      error = configured && reason instanceof Error && 'status' in reason && reason.status === 401 ? 'Wrong password.' : errorMessage(reason);
    } finally {
      busy = false;
    }
  }
</script>

<main class="login">
  <form class="card" onsubmit={submit}>
    <div class="brand">
      <img src="/favicon.svg" alt="" width="36" height="36" />
      <div>
        <h1>mail-sieve</h1>
        <p class="muted">{configured ? 'Sign in to your mailbox control panel.' : 'Choose an admin password to finish setup.'}</p>
      </div>
    </div>
    <Field label="Password" hint={configured ? undefined : 'At least 10 characters.'}>
      <!-- svelte-ignore a11y_autofocus -->
      <input type="password" autocomplete={configured ? 'current-password' : 'new-password'} bind:value={password} autofocus />
    </Field>
    {#if !configured}
      <Field label="Confirm password">
        <input type="password" autocomplete="new-password" bind:value={confirmation} />
      </Field>
    {/if}
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <Button variant="primary" type="submit" {busy}>{configured ? 'Sign in' : 'Create admin password'}</Button>
  </form>
</main>

<style>
  .login {
    min-height: 100%;
    display: grid;
    place-items: center;
    padding: var(--space-4);
  }
  form {
    width: min(400px, 100%);
    padding: var(--space-6);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }
  .brand {
    display: flex;
    gap: var(--space-3);
    align-items: center;
  }
  h1 {
    margin: 0;
    font-size: var(--text-xl);
  }
  p {
    margin: 0;
  }
  .error {
    color: var(--color-danger);
  }
</style>
