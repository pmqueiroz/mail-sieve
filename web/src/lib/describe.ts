import type { Rule } from './types';

const UNITS: Record<string, [string, string]> = {
  h: ['hour', 'hours'],
  d: ['day', 'days'],
  w: ['week', 'weeks'],
  m: ['month', 'months'],
  y: ['year', 'years'],
};

function span(value: string): string {
  const match = /^(\d+)([hdwmy])$/i.exec(value);
  if (!match) return value;
  const amount = Number(match[1]);
  const [one, many] = UNITS[match[2]!.toLowerCase()]!;
  return amount === 1 ? `a ${one}` : `${amount} ${many}`;
}

function describeQuery(query: string): string {
  let noun = 'messages';
  const adjectives: string[] = [];
  const clauses: string[] = [];
  for (const raw of query.split(/\s+/).filter(Boolean)) {
    const negated = raw.startsWith('-');
    const term = negated ? raw.slice(1) : raw;
    const [field, value = ''] = term.includes(':') ? [term.slice(0, term.indexOf(':')), term.slice(term.indexOf(':') + 1)] : ['', term];
    const not = negated ? 'not ' : '';
    if ((field === 'has' && value === 'list') || (field === 'list' && value === '*')) noun = negated ? 'non-newsletter messages' : 'newsletters';
    else if (field === 'category' || field === 'cat') noun = negated ? `messages that are not ${value}` : value;
    else if (field === 'is') adjectives.push(`${not}${value}`);
    else if (field === 'from') clauses.push(`${not}from ${value.replace(/^\*@/, '')}`);
    else if (field === 'domain') clauses.push(`${not}from ${value}`);
    else if (field === 'subject') clauses.push(`${negated ? 'without' : 'with'} "${value}" in the subject`);
    else if (field === 'older_than' || field === 'older') clauses.push(`${not}older than ${span(value)}`);
    else if (field === 'newer_than' || field === 'newer') clauses.push(`${not}from the last ${span(value).replace(/^a /, '')}`);
    else if (field === 'larger' || field === 'size') clauses.push(`${not}larger than ${value.toUpperCase()}`);
    else if (field === 'smaller') clauses.push(`${not}smaller than ${value.toUpperCase()}`);
    else if (field === 'has' && value === 'unsubscribe') clauses.push(negated ? 'without an unsubscribe link' : 'that offer unsubscribe');
    else if (field === 'in' || field === 'mailbox') clauses.push(`${not}in ${value}`);
    else clauses.push(`${negated ? 'not matching' : 'matching'} ${term}`);
  }
  return [...adjectives, noun, ...clauses].join(' ');
}

export function describeSchedule(rule: Pick<Rule, 'run_after_sync' | 'interval_minutes'>): string {
  if (rule.run_after_sync) return 'After every sync';
  if (rule.interval_minutes) return `Every ${rule.interval_minutes} minutes`;
  return 'Manual only';
}

export function describeRule(rule: Pick<Rule, 'query' | 'action' | 'run_after_sync' | 'interval_minutes'>): string {
  const subject = describeQuery(rule.query);
  const sentence = (() => {
    if (rule.action === 'archive') return `Archive ${subject}`;
    if (rule.action === 'trash') return `Move ${subject} to trash`;
    if (rule.action === 'mark_read') return `Mark ${subject} read`;
    if (rule.action === 'mark_unread') return `Mark ${subject} unread`;
    if (rule.action === 'flag') return `Flag ${subject}`;
    if (rule.action === 'unflag') return `Unflag ${subject}`;
    if (rule.action.startsWith('move:')) return `Move ${subject} to ${rule.action.slice(5)}`;
    return `${rule.action} ${subject}`;
  })();
  const when = rule.run_after_sync ? 'after every sync' : rule.interval_minutes ? `every ${rule.interval_minutes} minutes` : 'when you run it';
  return `${sentence}, ${when}.`;
}
