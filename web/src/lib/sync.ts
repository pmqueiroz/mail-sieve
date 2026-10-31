import { formatRelative, percent } from './format';
import type { Account, Job } from './types';

export type SyncTone = 'good' | 'warn' | 'bad';

export interface SyncStatus {
  tone: SyncTone;
  state: 'synced' | 'syncing' | 'error';
  label: string;
  short: string;
  pct: number;
  first: boolean;
}

export const syncDot: Record<SyncTone, string> = {
  good: 'bg-good',
  warn: 'bg-warn',
  bad: 'bg-bad',
};

export function runningSync(jobs: Job[], accountId: number): Job | null {
  return jobs.find((job) => job.kind === 'sync' && job.account_id === accountId && (job.state === 'running' || job.state === 'queued')) ?? null;
}

export function syncStatus(account: Account, job: Job | null): SyncStatus {
  const first = account.last_synced_at === null;
  if (job) {
    const pct = percent(job.done, job.total);
    return { tone: 'warn', state: 'syncing', label: `Syncing ${pct}%`, short: `${pct}%`, pct, first };
  }
  if (account.last_error) {
    return { tone: 'bad', state: 'error', label: 'Sync failed, see Accounts', short: 'Error', pct: 0, first };
  }
  return { tone: 'good', state: 'synced', label: `Synced ${formatRelative(account.last_synced_at)}`, short: 'Synced', pct: 100, first };
}
