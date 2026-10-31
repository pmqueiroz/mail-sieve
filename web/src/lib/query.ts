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
  ['from:*@linkedin.com', 'Any address at a domain'],
  ['domain:shop.com', 'Same, shorter'],
  ['subject:invoice', 'Words in the subject'],
  ['list:news', 'From a mailing list'],
  ['is:unread', 'Unread only'],
  ['is:pinned', 'Pinned senders only'],
  ['has:unsubscribe', 'Sender offers unsubscribe'],
  ['older_than:1y', 'Older than a year (d, m, y)'],
  ['newer_than:7d', 'From the last week'],
  ['before:2024-01-01', 'Before a date'],
  ['larger:5mb', 'Bigger than 5 MB'],
  ['-from:boss@work.com', 'Minus excludes a match'],
] as const;

export function hasTerm(query: string, term: string): boolean {
  return query.split(/\s+/).includes(term);
}

export function toggleTerm(query: string, term: string): string {
  const terms = query.split(/\s+/).filter(Boolean);
  return (terms.includes(term) ? terms.filter((existing) => existing !== term) : [...terms, term]).join(' ');
}

export interface Term {
  negated: boolean;
  field: string;
  value: string;
}

export function splitTerms(query: string): string[] {
  const terms: string[] = [];
  let current = '';
  let quoted = false;
  for (const char of query) {
    if (char === '"') quoted = !quoted;
    if (!quoted && /\s/.test(char)) {
      if (current) terms.push(current);
      current = '';
    } else {
      current += char;
    }
  }
  if (current) terms.push(current);
  return terms;
}

export function parseTerm(raw: string): Term {
  const negated = raw.length > 1 && raw.startsWith('-');
  const body = negated ? raw.slice(1) : raw;
  const colon = body.indexOf(':');
  const quote = body.indexOf('"');
  const fielded = colon > 0 && (quote === -1 || colon < quote);
  return {
    negated,
    field: fielded ? body.slice(0, colon).toLowerCase() : '',
    value: (fielded ? body.slice(colon + 1) : body).replaceAll('"', ''),
  };
}

export function formatTerm(term: Term): string {
  return `${term.negated ? '-' : ''}${term.field ? `${term.field}:` : ''}${quote(term.value)}`;
}
