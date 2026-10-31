import { api } from './api';
import { toasts } from './toasts.svelte';

export type ActionTarget = { ids: number[] } | { queries: string[] } | { query: string };

export async function runAction(
  accountId: number,
  target: ActionTarget,
  action: string,
  confirm: boolean,
  summary: { count: number; who: string },
): Promise<number | null> {
  const { job_id } = await api.post<{ job_id: number | null }>(`/accounts/${accountId}/actions`, { ...target, action, confirm });
  if (job_id === null) toasts.show('Nothing to do, no messages matched.');
  else toasts.track({ jobId: job_id, action, count: summary.count, who: summary.who, undoable: action !== 'delete' });
  return job_id;
}
