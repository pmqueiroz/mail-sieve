export type FieldKind = 'text' | 'choice' | 'duration' | 'date' | 'size';

export interface FieldSpec {
  field: string;
  label: string;
  kind: FieldKind;
  placeholder?: string;
  choices?: string[];
}

export const FIELDS: FieldSpec[] = [
  { field: 'from', label: 'From', kind: 'text', placeholder: '*@shop.com' },
  { field: 'domain', label: 'Domain', kind: 'text', placeholder: 'shop.com' },
  { field: 'subject', label: 'Subject', kind: 'text', placeholder: 'invoice' },
  { field: 'list', label: 'List', kind: 'text', placeholder: 'news' },
  { field: 'in', label: 'Folder', kind: 'text', placeholder: 'INBOX' },
  { field: 'category', label: 'Category', kind: 'choice', choices: ['newsletters', 'notifications', 'promotions', 'receipts', 'social', 'people'] },
  { field: 'is', label: 'Status', kind: 'choice', choices: ['unread', 'read', 'flagged', 'answered', 'pinned'] },
  { field: 'has', label: 'Has', kind: 'choice', choices: ['unsubscribe', 'list'] },
  { field: 'older_than', label: 'Older than', kind: 'duration' },
  { field: 'newer_than', label: 'Newer than', kind: 'duration' },
  { field: 'before', label: 'Before', kind: 'date' },
  { field: 'after', label: 'After', kind: 'date' },
  { field: 'larger', label: 'Larger than', kind: 'size' },
  { field: 'smaller', label: 'Smaller than', kind: 'size' },
  { field: '', label: 'Words', kind: 'text', placeholder: 'receipt' },
];

export const DURATION_UNITS: [string, string][] = [
  ['h', 'hours'],
  ['d', 'days'],
  ['w', 'weeks'],
  ['m', 'months'],
  ['y', 'years'],
];

export const SIZE_UNITS: [string, string][] = [
  ['kb', 'KB'],
  ['mb', 'MB'],
  ['gb', 'GB'],
];

const ALIASES: Record<string, string> = { older: 'older_than', newer: 'newer_than', size: 'larger', cat: 'category', mailbox: 'in' };

export function fieldLabel(field: string): string {
  const canonical = ALIASES[field] ?? field;
  return FIELDS.find((spec) => spec.field === canonical)?.label ?? field;
}
