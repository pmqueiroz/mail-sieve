import { api, withParams } from '../../lib/api';
import type { Category, Group, Job, Overview, Totals } from '../../lib/types';

const WEEK = 'newer_than:7d';
const WEEK_SECONDS = 7 * 86_400;
const TOP_SENDERS = 3;

export const CATEGORY_LABELS: Record<Category, string> = {
  newsletters: 'Newsletters',
  notifications: 'Notifications',
  promotions: 'Promotions',
  receipts: 'Receipts',
  social: 'Social',
  people: 'People',
};

export interface MixShift {
  category: Category;
  week: number;
  overall: number;
}

export interface Digest {
  arrived: Totals;
  unread: number;
  senders: Group[];
  shift: MixShift | null;
  freedBytes: number;
}

export interface Cleanup {
  messages: number;
  actions: number;
}

const share = (part: number, whole: number) => (whole > 0 ? part / whole : 0);

function biggestShift(week: { category: Category; count: number }[], overall: Overview['categories']): MixShift | null {
  const weekTotal = week.reduce((sum, entry) => sum + entry.count, 0);
  const overallTotal = overall.reduce((sum, entry) => sum + entry.count, 0);
  if (weekTotal === 0 || overallTotal === 0) return null;
  const shifts = week.map((entry) => ({
    category: entry.category,
    week: share(entry.count, weekTotal),
    overall: share(overall.find((candidate) => candidate.category === entry.category)?.count ?? 0, overallTotal),
  }));
  return shifts.reduce((best, entry) => (entry.week - entry.overall > best.week - best.overall ? entry : best));
}

export async function loadDigest(accountId: number): Promise<Digest> {
  const count = (q: string) => api.get<Totals>(withParams(`/accounts/${accountId}/count`, { q }));
  const categories = Object.keys(CATEGORY_LABELS) as Category[];
  const [arrived, unread, senders, overview, week] = await Promise.all([
    count(WEEK),
    count(`${WEEK} is:unread`),
    api.get<Group[]>(withParams(`/accounts/${accountId}/groups`, { q: WEEK, group: 'sender', sort: 'count', limit: TOP_SENDERS })),
    api.get<Overview>(`/accounts/${accountId}/overview`),
    Promise.all(categories.map(async (category) => ({ category, count: (await count(`${WEEK} category:${category}`)).count }))),
  ]);
  return { arrived, unread: unread.count, senders, shift: biggestShift(week, overview.categories), freedBytes: overview.freed_week_bytes };
}

export function weeklyCleanup(jobs: Job[], accountId: number, now = Date.now() / 1000): Cleanup {
  const cleaned = jobs.filter(
    (job) => job.kind === 'action' && job.account_id === accountId && job.state === 'done' && job.payload.origin !== 'undo' && (job.finished_at ?? 0) > now - WEEK_SECONDS,
  );
  return { messages: cleaned.reduce((sum, job) => sum + job.done, 0), actions: cleaned.length };
}
