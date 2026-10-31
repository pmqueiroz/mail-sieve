<script lang="ts">
  import { api, errorMessage } from '../api';
  import { actionSentence, percent } from '../format';
  import { session } from '../session.svelte';
  import { type JobToast, type MessageToast, toasts } from '../toasts.svelte';
  import Button from './Button.svelte';
  import ProgressBar from './ProgressBar.svelte';

  const messageIcons: Record<MessageToast['kind'], [string, string]> = {
    success: ['✓', 'bg-good'],
    error: ['!', 'bg-bad'],
    info: ['i', 'bg-ink3'],
  };

  function jobOf(toast: JobToast) {
    return session.jobs.find((job) => job.id === toast.jobId);
  }

  $effect(() => {
    for (const toast of toasts.items) {
      if (toast.type !== 'job') continue;
      const job = jobOf(toast);
      if (job && ['done', 'failed', 'cancelled'].includes(job.state)) toasts.expire(toast.id);
    }
  });

  async function cancel(toast: JobToast) {
    toasts.dismiss(toast.id);
    await api.post(`/jobs/${toast.jobId}/cancel`).catch((error) => toasts.show(errorMessage(error), 'error'));
  }

  async function undo(toast: JobToast) {
    toasts.dismiss(toast.id);
    try {
      const { job_id } = await api.post<{ job_id: number }>(`/jobs/${toast.jobId}/undo`);
      toasts.track({ jobId: job_id, action: 'restore', count: toast.count, who: toast.who, undoable: false });
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
  }
</script>

{#snippet card(icon: string, iconTone: string, title: string, sub: string, actionLabel: string, action: () => void, progress: number | null)}
  <div class="box-content flex flex-col gap-5 rounded-2xl border border-line bg-surface px-8 py-7 shadow-pop" role="status">
    <div class="flex items-center gap-5">
      <span class="grid size-10 shrink-0 place-items-center rounded-check text-tiny font-bold text-white {iconTone}" aria-hidden="true">{icon}</span>
      <span class="flex-1 leading-row"><b class="font-semibold">{title}</b>{#if sub}<br /><span class="text-xs text-ink2">{sub}</span>{/if}</span>
      <Button size="sm" class="font-bold" onclick={action}>{actionLabel}</Button>
    </div>
    {#if progress !== null}<ProgressBar value={progress} tone="good" thin />{/if}
  </div>
{/snippet}

<div class="fixed right-12 z-36 flex w-170 max-w-[calc(100vw-var(--spacing)*16)] flex-col gap-4 {toasts.raised ? 'bottom-48' : 'bottom-12'}" aria-live="polite">
  {#each toasts.items as toast (toast.id)}
    {#if toast.type === 'message'}
      {@const [icon, tone] = messageIcons[toast.kind]}
      {@render card(icon, tone, toast.message, '', 'Dismiss', () => toasts.dismiss(toast.id), null)}
    {:else}
      {@const job = jobOf(toast)}
      {@const state = job?.state ?? 'queued'}
      {@const progress = job ? percent(job.done, job.total) : 0}
      {@const count = toast.count ?? job?.total ?? 0}
      {#if state === 'done'}
        {@render card('✓', 'bg-good', actionSentence(toast.action, count, 'past'), `from ${toast.who}`, toast.undoable ? 'Undo' : 'Dismiss', () => (toast.undoable ? undo(toast) : toasts.dismiss(toast.id)), 100)}
      {:else if state === 'failed' || state === 'cancelled'}
        {@render card('!', 'bg-bad', `${actionSentence(toast.action, count, 'running')} ${state === 'failed' ? 'failed' : 'was cancelled'}`, job?.error ?? '', 'Dismiss', () => toasts.dismiss(toast.id), progress)}
      {:else}
        {@render card('…', 'bg-ink3', actionSentence(toast.action, count, 'running'), 'Running in the background. You can keep working.', 'Cancel', () => cancel(toast), progress)}
      {/if}
    {/if}
  {/each}
</div>
