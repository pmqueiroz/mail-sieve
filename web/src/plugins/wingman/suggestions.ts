import { api, withParams } from '../../lib/api';
import type { CompletionTarget } from '../../lib/plugins/config';
import { formatNumber } from '../../lib/format';
import { groupQuery } from '../../lib/query';
import type { Account, ActionResponse, Group, GroupBy, Rule } from '../../lib/types';
import { type ModelPick, modelPicks } from './model';

const NEVER_OPENED_MIN = 20;
const QUIET_LIST_MIN = 10;
const CANDIDATES = 25;

export type Outcome =
  | { kind: 'job'; jobId: number; working: string; done: string }
  | { kind: 'done'; done: string; undo: (() => Promise<void>) | null };

export interface Suggestion {
  key: string;
  alert: boolean;
  lead: string;
  strong: string;
  tail: string;
  primary: string;
  secondary: string;
  ack: string;
  snoozeDays: number;
  accept: () => Promise<Outcome>;
}

const plural = (count: number, word: string) => `${formatNumber(count)} ${word}${count === 1 ? '' : 's'}`;

const neverOpened = (group: Group, min: number) => group.pin === null && group.message_count >= min && group.unread_count === group.message_count;

const nameOf = (group: Group) => group.name || group.key;

type Wording = Pick<Suggestion, 'lead' | 'strong' | 'tail'>;

function modelWording(group: Group, reason: string): Wording | undefined {
  return reason ? { lead: '', strong: nameOf(group), tail: `: ${reason}` } : undefined;
}

function syncFailure(account: Account): Suggestion {
  return {
    key: `sync-error:${account.id}`,
    alert: true,
    lead: 'Your ',
    strong: account.label,
    tail: ' mailbox failed to sync, so counts may be out of date. Try again now?',
    primary: 'Sync now',
    secondary: 'Later',
    ack: "Okay. I'll bring it up again tomorrow.",
    snoozeDays: 1,
    accept: async () => {
      const { job_id } = await api.post<{ job_id: number | null }>(`/accounts/${account.id}/sync`);
      return { kind: 'done', done: job_id === null ? 'A sync is already running.' : 'Sync started. Check Accounts if it fails again.', undo: null };
    },
  };
}

function clearSender(accountId: number, group: Group, wording?: Wording, key = `never-opened:${accountId}:${group.key}`): Suggestion {
  const who = nameOf(group);
  const count = group.message_count;
  return {
    key,
    alert: false,
    lead: 'You never open mail from ',
    strong: who,
    tail: `. ${plural(count, 'message')} are sitting there. Clear them out?`,
    ...wording,
    primary: `Trash ${formatNumber(count)}`,
    secondary: 'Not now',
    ack: `Okay. I won't bring up ${who} again this week.`,
    snoozeDays: 7,
    accept: async () => {
      const { job_id } = await api.post<ActionResponse>(`/accounts/${accountId}/actions`, {
        query: groupQuery('sender', group.key),
        action: 'trash',
        confirm: false,
      });
      if (job_id === null) return { kind: 'done', done: `Nothing left to clear from ${who}.`, undo: null };
      return { kind: 'job', jobId: job_id, working: `Moving ${plural(count, 'message')} to trash`, done: `${plural(count, 'message')} from ${who} moved to trash.` };
    },
  };
}

function archiveList(accountId: number, group: Group, wording?: Wording, key = `quiet-list:${accountId}:${group.key}`): Suggestion {
  const who = nameOf(group);
  const query = groupQuery('list', group.key);
  return {
    key,
    alert: false,
    lead: '',
    strong: who,
    tail: ` sent ${plural(group.message_count, 'message')} and you opened none. Want a rule that archives them after every sync?`,
    ...wording,
    primary: 'Create rule',
    secondary: 'Skip',
    ack: "Skipped. I'll keep an eye on them.",
    snoozeDays: 7,
    accept: async () => {
      const rule = await api.post<Rule>('/rules', {
        account_id: accountId,
        name: `Archive ${who}`,
        query,
        action: 'archive',
        run_after_sync: true,
      });
      return { kind: 'done', done: `Rule created. Mail from ${who} gets archived after each sync.`, undo: () => api.delete(`/rules/${rule.id}`) };
    },
  };
}

function markRead(accountId: number, groupBy: GroupBy, group: Group, wording?: Wording): Suggestion {
  const who = nameOf(group);
  const count = group.unread_count;
  return {
    key: `ai-read:${accountId}:${groupBy}:${group.key}`,
    alert: false,
    lead: '',
    strong: who,
    tail: ` has ${plural(count, 'unread message')}. Mark them read?`,
    ...wording,
    primary: `Mark ${formatNumber(count)} read`,
    secondary: 'Not now',
    ack: `Okay. I won't bring up ${who} again this week.`,
    snoozeDays: 7,
    accept: async () => {
      const { job_id } = await api.post<ActionResponse>(`/accounts/${accountId}/actions`, {
        query: `${groupQuery(groupBy, group.key)} is:unread`,
        action: 'mark_read',
        confirm: false,
      });
      if (job_id === null) return { kind: 'done', done: `Nothing unread from ${who}.`, undo: null };
      return { kind: 'job', jobId: job_id, working: `Marking ${plural(count, 'message')} read`, done: `${plural(count, 'message')} from ${who} marked read.` };
    },
  };
}

function fromPick(accountId: number, pick: ModelPick, senders: Group[], lists: Group[]): Suggestion | null {
  const group = (pick.kind === 'sender' ? senders : lists).find((candidate) => candidate.key === pick.key);
  if (!group) return null;
  const wording = modelWording(group, pick.reason);
  if (pick.action === 'mark_read') return group.unread_count > 0 ? markRead(accountId, pick.kind, group, wording) : null;
  if (pick.action === 'trash') return clearSender(accountId, group, wording, `ai-trash:${accountId}:${group.key}`);
  return archiveList(accountId, group, wording, `ai-archive:${accountId}:${group.key}`);
}

async function candidates(accountId: number, group: 'sender' | 'list'): Promise<Group[]> {
  return api.get<Group[]>(withParams(`/accounts/${accountId}/groups`, { group, sort: 'unread', limit: CANDIDATES }));
}

export interface ModelOptions {
  target: CompletionTarget;
  analyzing: (busy: boolean) => void;
}

export async function findSuggestions(account: Account, syncing: boolean, model: ModelOptions | null): Promise<Suggestion[]> {
  const [senders, lists, rules] = await Promise.all([candidates(account.id, 'sender'), candidates(account.id, 'list'), api.get<Rule[]>('/rules')]);
  const ruled = rules.filter((rule) => rule.account_id === account.id).map((rule) => rule.query);
  const unruled = (group: Group) => !ruled.some((query) => query.includes(groupQuery('list', group.key)));
  const senderPool = senders.filter((group) => group.list_id === null && group.pin === null && group.unread_count > 0);
  const listPool = lists.filter((group) => group.pin === null && group.unread_count > 0 && unruled(group));
  const suggestions: Suggestion[] = [];
  if (account.last_error && !syncing) suggestions.push(syncFailure(account));
  const picks = model ? await modelPicks(model.target, account.id, senderPool, listPool, model.analyzing) : null;
  if (picks) {
    suggestions.push(...picks.flatMap((pick) => fromPick(account.id, pick, senderPool, listPool) ?? []));
    return suggestions;
  }
  suggestions.push(...senderPool.filter((group) => neverOpened(group, NEVER_OPENED_MIN)).map((group) => clearSender(account.id, group)));
  suggestions.push(...listPool.filter((group) => neverOpened(group, QUIET_LIST_MIN)).map((group) => archiveList(account.id, group)));
  return suggestions;
}
