<script lang="ts">
  import { api, errorMessage } from '../lib/api';
  import Button from '../lib/components/Button.svelte';
  import Chip from '../lib/components/Chip.svelte';
  import ConfirmDialog from '../lib/components/ConfirmDialog.svelte';
  import Field from '../lib/components/Field.svelte';
  import Modal from '../lib/components/Modal.svelte';
  import Pill from '../lib/components/Pill.svelte';
  import Segmented from '../lib/components/Segmented.svelte';
  import TextInput from '../lib/components/TextInput.svelte';
  import { liveTotals } from '../lib/count.svelte';
  import { describeRule, describeSchedule } from '../lib/describe';
  import { actionSentence, formatNumber, formatWhen } from '../lib/format';
  import { session } from '../lib/session.svelte';
  import { toasts } from '../lib/toasts.svelte';
  import type { Mailbox, Rule, RuleConflict, RuleImportReport } from '../lib/types';

  type Schedule = 'after_sync' | 'interval' | 'manual';
  type Draft = { id: number | null; name: string; query: string; action: string; enabled: boolean; schedule: Schedule; interval: number };

  const DEFAULT_INTERVAL = 60;
  const MOVE_PREFIX = 'move:';
  const ACTIONS: [string, string][] = [
    ['archive', 'Archive'],
    ['trash', 'Move to trash'],
    ['mark_read', 'Mark read'],
  ];
  const EXTRA_ACTIONS: Record<string, string> = { mark_unread: 'Mark unread', flag: 'Flag', unflag: 'Unflag' };

  let { accountId }: { accountId: number } = $props();

  const blank = (): Draft => ({
    id: null,
    name: '',
    query: 'has:list older_than:30d',
    action: 'archive',
    enabled: true,
    schedule: 'after_sync',
    interval: DEFAULT_INTERVAL,
  });

  let rules = $state<Rule[]>([]);
  let mailboxes = $state<Mailbox[]>([]);
  let editing = $state(false);
  let draft = $state<Draft>(blank());
  let saving = $state(false);
  let deleting = $state<Rule | null>(null);
  let confirmOpen = $state(false);
  let fileInput: HTMLInputElement | undefined = $state();
  let importing = $state(false);
  let importYaml = $state('');
  let importPreview = $state<RuleImportReport | null>(null);
  let onConflict = $state<RuleConflict>('skip');
  let applyingImport = $state(false);

  const accountRules = $derived(rules.filter((rule) => rule.account_id === accountId));
  const live = liveTotals(
    () => accountId,
    () => draft.query,
  );
  const movingToFolder = $derived(draft.action.startsWith(MOVE_PREFIX));
  const importCount = $derived(
    importPreview ? importPreview.added.length + (onConflict === 'replace' ? importPreview.existing.length : 0) : 0,
  );

  async function load() {
    try {
      [rules, mailboxes] = await Promise.all([api.get<Rule[]>('/rules'), api.get<Mailbox[]>(`/accounts/${accountId}/mailboxes`)]);
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
          interval: rule.interval_minutes ?? DEFAULT_INTERVAL,
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
      interval_minutes: draft.schedule === 'interval' ? Number(draft.interval) : null,
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
      const { job_id } = await api.post<{ job_id: number | null }>(`/rules/${rule.id}/run`);
      if (job_id === null) toasts.show('No messages matched.');
      else toasts.track({ jobId: job_id, action: rule.action, count: null, who: `rule ${rule.name}`, undoable: true });
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

  function exportRules() {
    location.assign(`/api/accounts/${accountId}/rules/export`);
  }

  async function previewImport(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = '';
    if (!file) return;
    try {
      importYaml = await file.text();
      importPreview = await api.post<RuleImportReport>(`/accounts/${accountId}/rules/import`, { yaml: importYaml });
      onConflict = 'skip';
      importing = true;
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }

  async function applyImport() {
    applyingImport = true;
    try {
      const report = await api.post<RuleImportReport>(`/accounts/${accountId}/rules/import`, {
        yaml: importYaml,
        on_conflict: onConflict,
        apply: true,
      });
      const count = report.added.length + (onConflict === 'replace' ? report.existing.length : 0);
      toasts.show(`Imported ${formatNumber(count)} ${count === 1 ? 'rule' : 'rules'}.`, 'success');
      importing = false;
      await load();
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    } finally {
      applyingImport = false;
    }
  }

  function lastRun(rule: Rule): string {
    const job = session.jobs.find((candidate) => candidate.payload.origin === `rule:${rule.id}` && candidate.state === 'done');
    const when = formatWhen(rule.last_run_at);
    return job ? `${when}, ${actionSentence(rule.action, job.done, 'past').toLowerCase()}` : when;
  }
</script>

<div class="mt-2 mb-10 flex flex-wrap items-center gap-6">
  <span class="min-w-120 flex-1 text-ink2">Rules run a saved search and an action on their own. They never delete permanently.</span>
  <div class="flex flex-wrap gap-4">
    <Button size="2xl" disabled={accountRules.length === 0} onclick={exportRules}>Export</Button>
    <Button size="2xl" onclick={() => fileInput?.click()}>Import</Button>
    <Button size="2xl" variant="primary" onclick={() => edit()}>New rule</Button>
  </div>
  <input bind:this={fileInput} type="file" accept=".yaml,.yml,application/yaml,text/yaml" class="hidden" onchange={previewImport} />
</div>

{#if accountRules.length === 0}
  <div class="py-24 text-center text-ink2">No rules yet. A rule archives or tidies matching mail after every sync, so the inbox stays clean on its own.</div>
{/if}

<div class="grid grid-cols-[repeat(auto-fill,minmax(--spacing(160),1fr))] gap-8">
  {#each accountRules as rule (rule.id)}
    <article class="flex flex-col gap-7 rounded-3xl border border-line p-10 {rule.enabled ? 'opacity-100' : 'opacity-65'}">
      <div class="flex items-center gap-5">
        <span class="flex-1 font-semibold">{rule.name}</span>
        <Pill tone={rule.enabled ? 'good' : 'muted'}>{rule.enabled ? 'Active' : 'Paused'}</Pill>
      </div>
      <div class="text-pretty text-xl leading-row font-medium tracking-snug">{describeRule(rule)}</div>
      <code class="self-start rounded-sm bg-chip px-5 py-2 font-mono text-xs text-ink2">{rule.query}</code>
      <div class="flex flex-wrap gap-9 text-xs text-ink3"><span>{describeSchedule(rule)}</span><span>Last run {lastRun(rule)}</span></div>
      <div class="flex flex-wrap gap-4">
        <Button variant="primary" onclick={() => runNow(rule)}>Run now</Button>
        <Button onclick={() => toggle(rule)}>{rule.enabled ? 'Pause' : 'Resume'}</Button>
        <Button onclick={() => edit(rule)}>Edit</Button>
        <Button variant="danger-text" onclick={() => ((deleting = rule), (confirmOpen = true))}>Delete</Button>
      </div>
    </article>
  {/each}
</div>

<Modal bind:open={editing} title="Rule">
  <Field label="Name"><TextInput bind:value={draft.name} placeholder="Old newsletters" /></Field>
  <Field label="Search">
    <TextInput mono spellcheck={false} bind:value={draft.query} placeholder="has:list older_than:30d" />
    {#snippet hint()}
      {#if live.error}<span class="text-bad">{live.error}</span>{:else}Matches <b class="text-ink">{formatNumber(live.totals?.count ?? 0)}</b> messages right now{/if}
    {/snippet}
  </Field>
  <div class="flex flex-col gap-3 text-sm font-semibold">
    Action
    <div class="flex flex-wrap gap-3">
      {#each ACTIONS as [value, label] (value)}
        <Chip active={draft.action === value} muted onclick={() => (draft.action = value)}>{label}</Chip>
      {/each}
      {#if EXTRA_ACTIONS[draft.action]}
        <Chip active muted onclick={() => {}}>{EXTRA_ACTIONS[draft.action]}</Chip>
      {/if}
      {#if mailboxes.length > 0}
        <Chip active={movingToFolder} muted onclick={() => (draft.action = `${MOVE_PREFIX}${mailboxes[0]!.name}`)}>Move to folder</Chip>
      {/if}
    </div>
    {#if movingToFolder}
      <div class="flex flex-wrap gap-3">
        {#each mailboxes as mailbox (mailbox.id)}
          <Chip active={draft.action === `${MOVE_PREFIX}${mailbox.name}`} muted onclick={() => (draft.action = `${MOVE_PREFIX}${mailbox.name}`)}>{mailbox.display_name}</Chip>
        {/each}
      </div>
    {/if}
  </div>
  <div class="flex flex-col gap-3 text-sm font-semibold">
    When
    <div class="flex flex-wrap gap-3">
      <Chip active={draft.schedule === 'after_sync'} muted onclick={() => (draft.schedule = 'after_sync')}>After every sync</Chip>
      <Chip active={draft.schedule === 'interval'} muted onclick={() => (draft.schedule = 'interval')}>Every {draft.interval} minutes</Chip>
      <Chip active={draft.schedule === 'manual'} muted onclick={() => (draft.schedule = 'manual')}>Manual only</Chip>
    </div>
  </div>
  {#if draft.schedule === 'interval'}
    <Field label="Minutes between runs"><TextInput type="number" min="1" bind:value={draft.interval} /></Field>
  {/if}
  {#snippet footer()}
    <Button size="dialog" onclick={() => (editing = false)}>Cancel</Button>
    <Button size="dialog" variant="primary" busy={saving} onclick={save}>Save rule</Button>
  {/snippet}
</Modal>

{#snippet ruleNames(names: string[])}
  <ul class="m-0 flex list-none flex-col gap-2 p-0 text-ink2">
    {#each names as name (name)}<li>{name}</li>{/each}
  </ul>
{/snippet}

<Modal bind:open={importing} title="Import rules">
  {#if importPreview}
    {#if importPreview.added.length > 0}
      <div class="flex flex-col gap-3 text-sm">
        <span class="font-semibold">New rules</span>
        {@render ruleNames(importPreview.added)}
      </div>
    {/if}
    {#if importPreview.existing.length > 0}
      <div class="flex flex-col gap-3 text-sm">
        <span class="font-semibold">Already exist</span>
        {@render ruleNames(importPreview.existing)}
        <div class="self-start">
          <Segmented
            label="Rules with the same name"
            bind:value={onConflict}
            options={[
              { value: 'skip', label: 'Keep mine' },
              { value: 'replace', label: 'Replace with file' },
            ]} />
        </div>
      </div>
    {/if}
    {#if importPreview.invalid.length > 0}
      <div class="flex flex-col gap-3 text-sm">
        <span class="font-semibold">Cannot be imported</span>
        <ul class="m-0 flex list-none flex-col gap-2 p-0">
          {#each importPreview.invalid as invalid, index (index)}
            <li><span class="text-ink">{invalid.name}</span> <span class="text-bad">{invalid.error}</span></li>
          {/each}
        </ul>
      </div>
    {/if}
    {#if importPreview.added.length + importPreview.existing.length + importPreview.invalid.length === 0}
      <span class="text-ink2">The file has no rules.</span>
    {/if}
  {/if}
  {#snippet footer()}
    <Button size="dialog" onclick={() => (importing = false)}>Cancel</Button>
    <Button size="dialog" variant="primary" busy={applyingImport} disabled={importCount === 0} onclick={applyImport}>
      Import {formatNumber(importCount)} {importCount === 1 ? 'rule' : 'rules'}
    </Button>
  {/snippet}
</Modal>

<ConfirmDialog
  bind:open={confirmOpen}
  title="Delete rule?"
  message="The rule {deleting?.name ?? ''} will stop running. Messages it already changed stay as they are."
  confirmLabel="Delete rule"
  danger
  onconfirm={remove} />
