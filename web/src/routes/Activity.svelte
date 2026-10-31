<script lang="ts">
  import { api, errorMessage } from '../lib/api';
  import Button from '../lib/components/Button.svelte';
  import Pill, { type PillTone } from '../lib/components/Pill.svelte';
  import ProgressBar, { type ProgressTone } from '../lib/components/ProgressBar.svelte';
  import { formatDuration, formatNumber, percent } from '../lib/format';
  import PluginSlot from '../lib/plugins/PluginSlot.svelte';
  import { isUndoable, jobOrigin, jobTitle, undoJob } from '../lib/jobs';
  import { session } from '../lib/session.svelte';
  import { toasts } from '../lib/toasts.svelte';
  import type { Job, JobState, PendingLink } from '../lib/types';

  const STATES: Record<JobState, [PillTone, string, ProgressTone]> = {
    running: ['strong', 'Running', 'btn'],
    queued: ['neutral', 'Queued', 'btn'],
    done: ['good', 'Done', 'good'],
    failed: ['bad', 'Failed', 'bad'],
    cancelled: ['muted', 'Cancelled', 'muted'],
  };

  let pending = $state<PendingLink[]>([]);

  $effect(() => {
    session.refreshJobs().catch((error) => toasts.show(errorMessage(error), 'error'));
  });

  $effect(() => {
    const id = session.accountId;
    void session.dataVersion;
    if (id === null) return;
    api
      .get<PendingLink[]>(`/accounts/${id}/unsubscribes/pending`)
      .then((links) => (pending = links))
      .catch(() => (pending = []));
  });

  function account(job: Job) {
    return session.accounts.find((candidate) => candidate.id === job.account_id);
  }

  function subtitle(job: Job): string {
    if (job.error) return job.error;
    const label = account(job)?.label ?? '';
    if (job.kind === 'sync') {
      const email = account(job)?.email ?? '';
      if (job.state === 'done') return `${email} · ${formatNumber(job.done)} new`;
      return `${email} · ${account(job)?.last_synced_at ? 'checking for changes' : 'first sync'}`;
    }
    const origin = jobOrigin(job);
    return origin ? `${origin} · ${label}` : label;
  }

  async function cancel(job: Job) {
    await api.post(`/jobs/${job.id}/cancel`).catch((error) => toasts.show(errorMessage(error), 'error'));
  }

  async function undo(job: Job) {
    try {
      await undoJob(job, account(job)?.label ?? 'this account');
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }

  async function markDone(link: PendingLink) {
    if (session.accountId === null) return;
    try {
      await api.post(`/accounts/${session.accountId}/unsubscribes/done`, { from_addr: link.from_addr });
      pending = pending.filter((candidate) => candidate.from_addr !== link.from_addr);
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }
</script>

<PluginSlot name="activity" context={{ accountId: session.accountId }} />

{#if pending.length > 0}
  <div class="mt-2 mb-10 flex flex-col gap-6 rounded-3xl border border-line px-10 py-9">
    <div class="flex items-center gap-5">
      <span class="grid size-10 place-items-center rounded-check bg-warn text-xs font-bold text-white">!</span>
      <span class="font-semibold">{pending.length} unsubscribe {pending.length === 1 ? 'link still needs' : 'links still need'} you</span>
    </div>
    {#each pending as link (link.from_addr)}
      <div class="flex flex-wrap items-center gap-6">
        <span class="min-w-100 flex-1 text-ink2"><b class="font-semibold text-ink">{link.name ?? link.from_addr}</b> asks you to finish on its site.</span>
        {#if link.url}<a href={link.url} target="_blank" rel="noopener noreferrer" class="text-sm font-semibold">Open link</a>{/if}
        <Button size="sm" onclick={() => markDone(link)}>Mark done</Button>
      </div>
    {/each}
  </div>
{/if}

{#if session.jobs.length === 0}
  <div class="py-24 text-center text-ink2">Nothing has run yet. Syncs, actions and rule runs show up here.</div>
{/if}

<div class="border-t border-line {pending.length > 0 ? '' : 'mt-2'}">
  {#each session.jobs as job (job.id)}
    {@const [tone, label, bar] = STATES[job.state]}
    {@const pct = job.state === 'done' ? 100 : percent(job.done, job.total)}
    <div class="box-content grid min-h-32 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-8 border-b border-line py-4 wide:grid-cols-[--spacing(48)_minmax(0,1.4fr)_minmax(0,1fr)_--spacing(32)_--spacing(40)]">
      <span class="justify-self-start"><Pill {tone}>{label}</Pill></span>
      <span class="flex min-w-0 flex-col leading-row">
        <span class="font-semibold">{jobTitle(job, account(job))}</span>
        <span class="text-xs {job.error ? 'text-bad' : 'text-ink3'}">{subtitle(job)}</span>
      </span>
      <span class="hidden items-center gap-5 wide:flex">
        <ProgressBar value={pct} tone={bar} class="flex-1" />
        <span class="w-18 text-right text-xs text-ink2">{pct}%</span>
      </span>
      <span class="hidden text-xs text-ink3 wide:inline">{formatDuration(job.started_at, job.finished_at)}</span>
      <span class="justify-self-end">
        {#if job.state === 'running' || job.state === 'queued'}
          <Button size="sm" onclick={() => cancel(job)}>Cancel</Button>
        {:else if isUndoable(job)}
          <Button size="sm" onclick={() => undo(job)}>Undo</Button>
        {/if}
      </span>
    </div>
  {/each}
</div>
