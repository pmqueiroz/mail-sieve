import { api } from './api';
import { formatNumber } from './format';
import { toasts } from './toasts.svelte';
import type { ActionResponse } from './types';

export type ActionTarget = { ids: number[] } | { queries: string[] } | { query: string };

export async function runAction(
  accountId: number,
  target: ActionTarget,
  action: string,
  confirm: boolean,
  summary: { count: number; who: string },
  includePinned = false,
): Promise<number | null> {
  const { job_id, skipped_pinned } = await api.post<ActionResponse>(`/accounts/${accountId}/actions`, {
    ...target,
    action,
    confirm: confirm || includePinned,
    include_pinned: includePinned,
  });
  if (job_id === null) {
    toasts.show(skipped_pinned > 0 ? 'Nothing to do. Every matching message is from a pinned sender.' : 'Nothing to do, no messages matched.');
    return null;
  }
  toasts.track({ jobId: job_id, action, count: summary.count, who: summary.who, undoable: action !== 'delete' });
  if (skipped_pinned > 0) toasts.show(`Skipped ${formatNumber(skipped_pinned)} ${skipped_pinned === 1 ? 'message' : 'messages'} from pinned senders.`);
  return job_id;
}
