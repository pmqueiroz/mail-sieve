<script lang="ts">
  import { api, errorMessage } from '../lib/api';
  import Badge from '../lib/components/Badge.svelte';
  import Button from '../lib/components/Button.svelte';
  import EmptyState from '../lib/components/EmptyState.svelte';
  import ProgressBar from '../lib/components/ProgressBar.svelte';
  import { actionLabel, formatDateTime, formatDuration, formatNumber } from '../lib/format';
  import { session } from '../lib/session.svelte';
  import { toasts } from '../lib/toasts.svelte';
  import type { Job, JobState } from '../lib/types';

  const tones: Record<JobState, 'neutral' | 'accent' | 'success' | 'warning' | 'danger'> = {
    queued: 'neutral',
    running: 'accent',
    done: 'success',
    failed: 'danger',
    cancelled: 'warning',
  };

  $effect(() => {
    session.refreshJobs().catch((error) => toasts.show(errorMessage(error), 'error'));
  });

  function title(job: Job) {
    if (job.kind === 'sync') return 'Sync';
    const origin = job.payload.origin?.startsWith('rule:') ? ' (rule)' : job.payload.origin === 'unsubscribe' ? ' (after unsubscribe)' : '';
    return `${actionLabel(job.payload.action)}${origin}`;
  }

  function accountLabel(job: Job) {
    return session.accounts.find((account) => account.id === job.account_id)?.label ?? '';
  }

  async function cancel(job: Job) {
    try {
      await api.post(`/jobs/${job.id}/cancel`);
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }
</script>

<section class="page">
  <div class="page-header">
    <h1>Activity</h1>
  </div>
  {#if session.jobs.length === 0}
    <EmptyState title="Nothing has run yet" />
  {:else}
    <div class="jobs">
      {#each session.jobs as job (job.id)}
        <article class="job card">
          <div class="info">
            <div class="title">
              <strong>{title(job)}</strong>
              <Badge tone={tones[job.state]}>{job.state}</Badge>
              <span class="muted small">{accountLabel(job)} · #{job.id} · {formatDateTime(job.created_at)}</span>
            </div>
            {#if job.state === 'running' || job.state === 'queued'}
              <ProgressBar done={job.done} total={job.total} indeterminate />
            {/if}
            <span class="muted small">
              {formatNumber(job.done)}{job.total ? ` of ${formatNumber(job.total)}` : ''} messages
              {#if job.started_at}· {formatDuration(job.started_at, job.finished_at)}{/if}
            </span>
            {#if job.error}<span class="error small">{job.error}</span>{/if}
          </div>
          {#if job.state === 'running' || job.state === 'queued'}
            <Button size="sm" onclick={() => cancel(job)}>Cancel</Button>
          {/if}
        </article>
      {/each}
    </div>
  {/if}
</section>

<style>
  .jobs {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    overflow: auto;
  }
  .job {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-3) var(--space-4);
  }
  .info {
    flex: 1;
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
  .error {
    color: var(--color-danger);
  }
</style>
