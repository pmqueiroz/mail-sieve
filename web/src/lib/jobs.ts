import { api } from './api';
import { actionSentence } from './format';
import { toasts } from './toasts.svelte';
import type { Account, Job } from './types';

export function isActive(job: Job): boolean {
  return job.state === 'running' || job.state === 'queued';
}

export function isUndoable(job: Job): boolean {
  return job.kind === 'action' && job.state === 'done' && job.payload.action !== 'delete' && job.payload.origin !== 'undo' && !job.payload.undone_by;
}

export function jobTitle(job: Job, account: Account | undefined): string {
  const active = isActive(job);
  if (job.kind === 'sync') return `${active ? 'Syncing' : 'Sync'} ${account?.label ?? ''}`.trim();
  const count = job.total || job.done;
  if (job.kind === 'undo' || job.payload.origin === 'undo') return actionSentence('restore', count, active ? 'running' : 'past');
  return actionSentence(job.payload.action, count, active ? 'running' : 'past');
}

export function jobOrigin(job: Job): string {
  const origin = job.payload.origin ?? '';
  if (origin.startsWith('rule:')) return 'Rule';
  if (origin === 'unsubscribe') return 'After unsubscribe';
  if (origin === 'undo') return 'Undo';
  return '';
}

export async function undoJob(job: Job, who: string): Promise<number> {
  const { job_id } = await api.post<{ job_id: number }>(`/jobs/${job.id}/undo`);
  toasts.track({ jobId: job_id, action: 'restore', count: job.done, who, undoable: false });
  return job_id;
}
