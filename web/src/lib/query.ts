import type { GroupBy } from './types';

export function quote(value: string): string {
  return /[\s"]/.test(value) ? `"${value.replaceAll('"', '')}"` : value;
}

export function groupQuery(groupBy: GroupBy, key: string): string {
  const field = groupBy === 'sender' ? 'from' : groupBy === 'domain' ? 'domain' : 'list';
  return `${field}:${quote(key)}`;
}

export function combine(...parts: (string | null | undefined)[]): string {
  return parts
    .map((part) => part?.trim())
    .filter(Boolean)
    .join(' ');
}

export const QUERY_HELP = [
  ['from:deals@shop.com', 'exact sender'],
  ['from:*@shop.com', 'wildcard sender'],
  ['domain:shop.com', 'sender domain and subdomains'],
  ['subject:invoice', 'subject contains'],
  ['list:news', 'mailing list id contains'],
  ['in:inbox', 'folder or Gmail label'],
  ['is:unread  is:read  is:flagged', 'state'],
  ['has:unsubscribe  has:list', 'newsletter signals'],
  ['older_than:1y  newer_than:7d', 'age (h, d, w, m, y)'],
  ['before:2024-01-01  after:2023-06-01', 'dates'],
  ['larger:5mb  smaller:100kb', 'size'],
  ['-from:boss@work.com', 'negate any term'],
  ['receipt', 'free text over subject and sender'],
] as const;
