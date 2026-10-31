<script lang="ts">
  import { api, errorMessage } from '../lib/api';
  import Badge from '../lib/components/Badge.svelte';
  import Button from '../lib/components/Button.svelte';
  import ConfirmDialog from '../lib/components/ConfirmDialog.svelte';
  import EmptyState from '../lib/components/EmptyState.svelte';
  import Field from '../lib/components/Field.svelte';
  import Modal from '../lib/components/Modal.svelte';
  import QueryBar from '../lib/components/QueryBar.svelte';
  import { actionLabel, formatRelative } from '../lib/format';
  import { toasts } from '../lib/toasts.svelte';
  import type { Mailbox, Rule } from '../lib/types';

  let { accountId }: { accountId: number } = $props();

  type Draft = {
    id: number | null;
    name: string;
    query: string;
    action: string;
    enabled: boolean;
    schedule: 'interval' | 'after_sync' | 'manual';
    interval: number;
  };

  const blank = (): Draft => ({
    id: null,
    name: '',
    query: 'has:unsubscribe older_than:30d',
    action: 'archive',
    enabled: true,
    schedule: 'after_sync',
    interval: 60,
  });

  let rules = $state<Rule[]>([]);
  let mailboxes = $state<Mailbox[]>([]);
  let editing = $state(false);
  let draft = $state<Draft>(blank());
  let saving = $state(false);
  let deleting = $state<Rule | null>(null);
  let confirmOpen = $state(false);

  const accountRules = $derived(rules.filter((rule) => rule.account_id === accountId));

  async function load() {
    try {
      [rules, mailboxes] = await Promise.all([
        api.get<Rule[]>('/rules'),
        api.get<Mailbox[]>(`/accounts/${accountId}/mailboxes`),
      ]);
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }

  $effect(() => {
    void accountId;
    load();
  });

  function edit(rule?: Rule) {
    draft = rule
      ? {
          id: rule.id,
          name: rule.name,
          query: rule.query,
          action: rule.action,
          enabled: rule.enabled,
          schedule: rule.run_after_sync ? 'after_sync' : rule.interval_minutes ? 'interval' : 'manual',
          interval: rule.interval_minutes ?? 60,
        }
      : blank();
    editing = true;
  }

  async function save() {
    saving = true;
    const body = {
      account_id: accountId,
      name: draft.name,
      query: draft.query,
      action: draft.action,
      enabled: draft.enabled,
      interval_minutes: draft.schedule === 'interval' ? draft.interval : null,
      run_after_sync: draft.schedule === 'after_sync',
    };
    try {
      if (draft.id === null) await api.post('/rules', body);
      else await api.put(`/rules/${draft.id}`, body);
      editing = false;
      await load();
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    } finally {
      saving = false;
    }
  }

  async function toggle(rule: Rule) {
    try {
      await api.put(`/rules/${rule.id}`, { ...rule, enabled: !rule.enabled });
      await load();
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }

  async function runNow(rule: Rule) {
    try {
      const response = await api.post<{ job_id: number | null }>(`/rules/${rule.id}/run`);
      toasts.show(response.job_id === null ? 'No messages matched.' : `Rule "${rule.name}" queued.`, 'success');
      await load();
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }

  async function remove() {
    if (!deleting) return;
    await api.delete(`/rules/${deleting.id}`);
    deleting = null;
    await load();
  }

  function schedule(rule: Rule) {
    if (rule.run_after_sync) return 'after every sync';
    if (rule.interval_minutes) return `every ${rule.interval_minutes} min`;
    return 'manual only';
  }
</script>

<section class="page">
  <div class="page-header">
    <h1>Rules</h1>
    <Button variant="primary" onclick={() => edit()}>New rule</Button>
  </div>

  {#if accountRules.length === 0}
    <EmptyState title="No rules yet">
      <p>Rules apply an action to every message matching a query, after each sync or on a schedule. Permanent deletion is never automated.</p>
      <Button variant="primary" onclick={() => edit()}>Create a rule</Button>
    </EmptyState>
  {:else}
    <div class="rules">
      {#each accountRules as rule (rule.id)}
        <article class="rule card" class:disabled={!rule.enabled}>
          <div class="main">
            <div class="title">
              <strong>{rule.name}</strong>
              <Badge tone="accent">{actionLabel(rule.action)}</Badge>
              <Badge>{schedule(rule)}</Badge>
              {#if !rule.enabled}<Badge tone="warning">paused</Badge>{/if}
            </div>
            <code class="mono">{rule.query}</code>
            <span class="muted small">Last run {formatRelative(rule.last_run_at)}</span>
          </div>
          <div class="toolbar">
            <Button size="sm" onclick={() => runNow(rule)}>Run now</Button>
            <Button size="sm" onclick={() => toggle(rule)}>{rule.enabled ? 'Pause' : 'Resume'}</Button>
            <Button size="sm" onclick={() => edit(rule)}>Edit</Button>
            <Button size="sm" variant="danger" onclick={() => ((deleting = rule), (confirmOpen = true))}>Delete</Button>
          </div>
        </article>
      {/each}
    </div>
  {/if}
</section>

<Modal bind:open={editing} title={draft.id === null ? 'New rule' : 'Edit rule'} width={620}>
  <Field label="Name"><input type="text" bind:value={draft.name} placeholder="Archive old newsletters" /></Field>
  <Field label="Messages matching">
    <QueryBar {accountId} bind:value={draft.query} />
  </Field>
  <Field label="Action">
    <select bind:value={draft.action}>
      <option value="archive">Archive</option>
      <option value="trash">Move to trash</option>
      <option value="mark_read">Mark read</option>
      <option value="mark_unread">Mark unread</option>
      <option value="flag">Flag</option>
      <option value="unflag">Unflag</option>
      {#each mailboxes as mailbox (mailbox.id)}
        <option value="move:{mailbox.name}">Move to {mailbox.display_name}</option>
      {/each}
    </select>
  </Field>
  <Field label="When">
    <select bind:value={draft.schedule}>
      <option value="after_sync">After every sync</option>
      <option value="interval">On a schedule</option>
      <option value="manual">Only when I run it</option>
    </select>
  </Field>
  {#if draft.schedule === 'interval'}
    <Field label="Every (minutes)"><input type="number" min="1" bind:value={draft.interval} /></Field>
  {/if}
  <label class="inline"><input type="checkbox" bind:checked={draft.enabled} /> Enabled</label>
  {#snippet footer()}
    <Button variant="ghost" onclick={() => (editing = false)}>Cancel</Button>
    <Button variant="primary" busy={saving} onclick={save}>Save rule</Button>
  {/snippet}
</Modal>

<ConfirmDialog
  bind:open={confirmOpen}
  title="Delete rule?"
  message="The rule {deleting?.name ?? ''} will stop running. Messages it already changed stay as they are."
  confirmLabel="Delete rule"
  danger
  onconfirm={remove} />

<style>
  .rules {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    overflow: auto;
  }
  .rule {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4);
    flex-wrap: wrap;
  }
  .rule.disabled {
    opacity: 0.7;
  }
  .main {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 0;
  }
  .title {
    display: flex;
    gap: var(--space-2);
    align-items: center;
    flex-wrap: wrap;
  }
  .small {
    font-size: var(--text-xs);
  }
  .inline {
    display: flex;
    gap: var(--space-2);
    align-items: center;
  }
</style>
