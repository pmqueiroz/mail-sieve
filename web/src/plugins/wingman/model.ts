import { errorMessage } from '../../lib/api';
import { extractJson } from '../../lib/plugins/ai/json';
import { complete, type CompletionTarget } from '../../lib/plugins/config';
import { toasts } from '../../lib/toasts.svelte';
import type { Group } from '../../lib/types';

const CACHE_MS = 30 * 60_000;
const MAX_PICKS = 5;
const MAX_REASON = 180;

export type PickKind = 'sender' | 'list';
export type PickAction = 'trash' | 'mark_read' | 'archive_rule';

export interface ModelPick {
  kind: PickKind;
  key: string;
  action: PickAction;
  reason: string;
}

const ALLOWED: Record<PickKind, PickAction[]> = {
  sender: ['trash', 'mark_read'],
  list: ['archive_rule', 'mark_read'],
};

const SYSTEM = `You help a person clean up their mailbox. You only see header statistics, never message bodies.
Pick at most ${MAX_PICKS} groups the person would most likely want gone, preferring groups with many unread messages.
Never pick anything that looks personal, financial, security related, a receipt or a reply thread.
Senders allow "trash" or "mark_read". Lists allow "archive_rule" (archive after every sync) or "mark_read".
Answer with only a JSON array and no other text:
[{"id": <number>, "action": "<action>", "reason": "<one short sentence to the person about why>"}]
Return [] when nothing is clearly worth cleaning.`;

const cache = new Map<string, { at: number; picks: ModelPick[] | null }>();

function describeGroup(id: number, kind: PickKind, group: Group): string {
  const last = group.last_date ? new Date(group.last_date * 1000).toISOString().slice(0, 10) : 'unknown';
  return [id, kind, group.name ?? '', group.key, group.message_count, group.unread_count, last].join('\t');
}

function cleanReason(raw: unknown): string {
  const text = typeof raw === 'string' ? raw.replace(/\s*—\s*/g, ', ').replace(/\s+/g, ' ').trim() : '';
  return text.length > MAX_REASON ? `${text.slice(0, MAX_REASON - 1)}…` : text;
}

export function parsePicks(answer: string, pool: { kind: PickKind; group: Group }[]): ModelPick[] {
  const parsed = extractJson(answer);
  if (!Array.isArray(parsed)) throw new Error('the model did not answer with a list');
  const picks: ModelPick[] = [];
  for (const entry of parsed) {
    const candidate = pool[Number(entry?.id)];
    const action = entry?.action as PickAction;
    if (!candidate || !ALLOWED[candidate.kind].includes(action)) continue;
    if (picks.some((pick) => pick.kind === candidate.kind && pick.key === candidate.group.key)) continue;
    picks.push({ kind: candidate.kind, key: candidate.group.key, action, reason: cleanReason(entry?.reason) });
  }
  return picks.slice(0, MAX_PICKS);
}

export async function modelPicks(
  target: CompletionTarget,
  accountId: number,
  senders: Group[],
  lists: Group[],
  analyzing: (busy: boolean) => void,
): Promise<ModelPick[] | null> {
  const cacheKey = `${accountId}|${target.kind}|${target.baseUrl}|${target.model}`;
  const cached = cache.get(cacheKey);
  if (cached && Date.now() - cached.at < CACHE_MS) return cached.picks;
  const pool = [...senders.map((group) => ({ kind: 'sender' as const, group })), ...lists.map((group) => ({ kind: 'list' as const, group }))];
  if (pool.length === 0) return [];
  const prompt = ['id\tkind\tname\taddress or list id\tmessages\tunread\tlatest', ...pool.map((entry, id) => describeGroup(id, entry.kind, entry.group))].join('\n');
  analyzing(true);
  try {
    const picks = parsePicks(await complete({ ...target, system: SYSTEM, prompt, maxTokens: 1024 }), pool);
    cache.set(cacheKey, { at: Date.now(), picks });
    return picks;
  } catch (error) {
    cache.set(cacheKey, { at: Date.now(), picks: null });
    toasts.show(`The wingman could not reach its model, so it uses the built-in rules for now. ${errorMessage(error)}`, 'error', 10000);
    return null;
  } finally {
    analyzing(false);
  }
}
