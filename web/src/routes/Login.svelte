<script lang="ts">
  import { api, errorMessage } from '../lib/api';
  import Button from '../lib/components/Button.svelte';
  import Field from '../lib/components/Field.svelte';
  import Icon from '../lib/components/Icon.svelte';
  import TextInput from '../lib/components/TextInput.svelte';
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

<main class="grid min-h-screen place-items-center bg-page p-8">
  <form class="flex w-[min(--spacing(200),100%)] flex-col gap-8 rounded-4xl bg-surface p-12 shadow-pop" onsubmit={submit}>
    <div class="flex items-center gap-6">
      <div class="grid size-18 place-items-center rounded-md bg-btn text-btn-ink"><Icon name="logo" size={20} /></div>
      <div>
        <h1 class="m-0 text-xl font-semibold tracking-tight">mail-sieve</h1>
        <p class="m-0 text-sm text-ink2">{configured ? 'Sign in to your mailbox control panel.' : 'Choose an admin password to finish setup.'}</p>
      </div>
    </div>
    <Field label="Password" hint={configured ? undefined : 'At least 10 characters.'}>
      <!-- svelte-ignore a11y_autofocus -->
      <TextInput type="password" autocomplete={configured ? 'current-password' : 'new-password'} bind:value={password} autofocus />
    </Field>
    {#if !configured}
      <Field label="Confirm password">
        <TextInput type="password" autocomplete="new-password" bind:value={confirmation} />
      </Field>
    {/if}
    {#if error}<p class="m-0 text-bad" role="alert">{error}</p>{/if}
    <Button size="dialog" variant="primary" type="submit" {busy}>{configured ? 'Sign in' : 'Create admin password'}</Button>
  </form>
</main>
