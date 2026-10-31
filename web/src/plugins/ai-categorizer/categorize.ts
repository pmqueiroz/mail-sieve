import { api, withParams } from '../../lib/api';
import { extractJson } from '../../lib/plugins/ai/json';
import { complete, type CompletionTarget, loadPluginConfig, savePluginConfig } from '../../lib/plugins/config';
import type { Group, MessagePage } from '../../lib/types';
import { type CategorizerSettings, OTHER, PLUGIN_ID, type SortedAccount, withCategorizerDefaults } from './settings';

export const PEOPLE_QUERY = 'category:people';
export const MAX_SENDERS = 150;
const SAMPLE_MESSAGES = 1000;
const SUBJECTS_PER_SENDER = 3;
const MAX_SUBJECT = 80;

export interface Bucket {
  category: string;
  senders: Group[];
  messages: number;
}

export function peopleSenders(accountId: number): Promise<Group[]> {
  return api.get<Group[]>(withParams(`/accounts/${accountId}/groups`, { q: PEOPLE_QUERY, group: 'sender', sort: 'count', limit: MAX_SENDERS }));
}

async function sampleSubjects(accountId: number): Promise<Map<string, string[]>> {
  const page = await api.get<MessagePage>(withParams(`/accounts/${accountId}/messages`, { q: PEOPLE_QUERY, limit: SAMPLE_MESSAGES }));
  const subjects = new Map<string, string[]>();
  for (const message of page.messages) {
    const list = subjects.get(message.from_addr) ?? [];
    const subject = message.subject?.replace(/\s+/g, ' ').trim().slice(0, MAX_SUBJECT);
    if (subject && list.length < SUBJECTS_PER_SENDER && !list.includes(subject)) list.push(subject);
    subjects.set(message.from_addr, list);
  }
  return subjects;
}

function system(categories: string[]): string {
  return `You sort the senders of a person's mailbox into categories. You only see sender names, addresses, message counts and a few subjects.
Use exactly one of these categories for each sender: ${categories.map((name) => JSON.stringify(name)).join(', ')}.
Use "${OTHER}" when no other category fits.
Answer with only a JSON object that maps each sender id to its category, and no other text, for example {"0": "${categories[0]}"}.`;
}

export function parseAssignments(answer: string, senders: Group[], categories: string[]): Record<string, string> {
  const parsed = extractJson(answer);
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) throw new Error('the model did not answer with an object');
  const byName = new Map(categories.map((name) => [name.toLowerCase(), name]));
  const assignments: Record<string, string> = {};
  for (const [id, raw] of Object.entries(parsed as Record<string, unknown>)) {
    const sender = senders[Number(id)];
    if (!sender || typeof raw !== 'string') continue;
    assignments[sender.key] = byName.get(raw.trim().toLowerCase()) ?? OTHER;
  }
  return assignments;
}

export async function sortPeople(accountId: number, target: CompletionTarget, categories: string[]): Promise<SortedAccount> {
  const [senders, subjects] = await Promise.all([peopleSenders(accountId), sampleSubjects(accountId)]);
  if (senders.length === 0) return { at: Date.now(), senders: {} };
  const prompt = [
    'id\tname\taddress\tmessages\tsample subjects',
    ...senders.map((sender, id) => [id, sender.name ?? '', sender.key, sender.message_count, (subjects.get(sender.key) ?? []).join(' | ')].join('\t')),
  ].join('\n');
  const answer = await complete({ ...target, system: system(categories), prompt, maxTokens: 4096 });
  const sorted = { at: Date.now(), senders: parseAssignments(answer, senders, categories) };
  const config = await loadPluginConfig<Partial<CategorizerSettings>>(PLUGIN_ID);
  const settings = withCategorizerDefaults(config.settings);
  await savePluginConfig(PLUGIN_ID, { ...settings, sorted: { ...settings.sorted, [accountId]: sorted } });
  return sorted;
}

export function buckets(senders: Group[], sorted: SortedAccount | undefined, categories: string[]): Bucket[] {
  if (!sorted) return [];
  const byCategory = new Map<string, Group[]>(categories.map((name) => [name, []]));
  for (const sender of senders) {
    const category = sorted.senders[sender.key];
    if (category) (byCategory.get(category) ?? byCategory.get(OTHER)!).push(sender);
  }
  return [...byCategory.entries()]
    .map(([category, members]) => ({ category, senders: members, messages: members.reduce((sum, sender) => sum + sender.message_count, 0) }))
    .filter((bucket) => bucket.senders.length > 0)
    .sort((a, b) => b.messages - a.messages);
}
