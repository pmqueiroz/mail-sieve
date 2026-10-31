const numberFormat = new Intl.NumberFormat();
const dateFormat = new Intl.DateTimeFormat(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
const shortDateFormat = new Intl.DateTimeFormat(undefined, { month: 'short', day: 'numeric' });
const timeFormat = new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit' });
const dateTimeFormat = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' });
const relativeFormat = new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' });

export function formatNumber(value: number): string {
  return numberFormat.format(value);
}

export function formatBytes(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value < 10 && unit > 0 ? value.toFixed(1) : Math.round(value)} ${units[unit]}`;
}

export function formatDate(timestamp: number | null): string {
  if (!timestamp) return '';
  return dateFormat.format(new Date(timestamp * 1000));
}

export function formatShortDate(timestamp: number | null): string {
  if (!timestamp) return '';
  const date = new Date(timestamp * 1000);
  return date.getFullYear() === new Date().getFullYear() ? shortDateFormat.format(date) : dateFormat.format(date);
}

export function formatWhen(timestamp: number | null): string {
  if (!timestamp) return 'never';
  const date = new Date(timestamp * 1000);
  const today = new Date().toDateString() === date.toDateString();
  return today ? `today ${timeFormat.format(date)}` : formatShortDate(timestamp);
}

export function percent(part: number, whole: number): number {
  return whole > 0 ? Math.round((part / whole) * 100) : 0;
}

export function formatDateTime(timestamp: number | null): string {
  if (!timestamp) return '';
  return dateTimeFormat.format(new Date(timestamp * 1000));
}

export function formatRelative(timestamp: number | null): string {
  if (!timestamp) return 'never';
  const seconds = timestamp - Date.now() / 1000;
  const steps: [number, Intl.RelativeTimeFormatUnit][] = [
    [60, 'second'],
    [60, 'minute'],
    [24, 'hour'],
    [30, 'day'],
    [12, 'month'],
    [Infinity, 'year'],
  ];
  let value = seconds;
  for (const [size, unit] of steps) {
    if (Math.abs(value) < size) return relativeFormat.format(Math.round(value), unit);
    value /= size;
  }
  return '';
}

export function formatDuration(start: number | null, end: number | null): string {
  if (!start) return '';
  const seconds = Math.max(0, (end ?? Date.now() / 1000) - start);
  if (seconds < 1) return `${Math.round(seconds * 1000)} ms`;
  if (seconds < 60) return `${Math.round(seconds)} s`;
  return `${Math.round(seconds / 60)} min`;
}

const actionLabels: Record<string, string> = {
  archive: 'Archive',
  trash: 'Move to trash',
  delete: 'Delete permanently',
  mark_read: 'Mark read',
  mark_unread: 'Mark unread',
  flag: 'Flag',
  unflag: 'Unflag',
};

const pastActions: Record<string, [string, string]> = {
  archive: ['Archived', ''],
  trash: ['Moved', ' to trash'],
  delete: ['Permanently deleted', ''],
  mark_read: ['Marked', ' read'],
  mark_unread: ['Marked', ' unread'],
  flag: ['Flagged', ''],
  unflag: ['Unflagged', ''],
  restore: ['Restored', ''],
};

const runningActions: Record<string, [string, string]> = {
  archive: ['Archiving', ''],
  trash: ['Moving', ' to trash'],
  delete: ['Deleting', ''],
  mark_read: ['Marking', ' read'],
  mark_unread: ['Marking', ' unread'],
  flag: ['Flagging', ''],
  unflag: ['Unflagging', ''],
  restore: ['Restoring', ''],
};

export function actionSentence(action: string | undefined, count: number, tense: 'past' | 'running'): string {
  const messages = `${formatNumber(count)} message${count === 1 ? '' : 's'}`;
  if (!action) return messages;
  if (action.startsWith('move:')) return `${tense === 'past' ? 'Moved' : 'Moving'} ${messages} to ${action.slice(5)}`;
  const [verb, suffix] = (tense === 'past' ? pastActions : runningActions)[action] ?? [actionLabel(action), ''];
  return `${verb} ${messages}${suffix}`;
}

export function actionLabel(action: string | undefined): string {
  if (!action) return '';
  if (action.startsWith('move:')) return `Move to ${action.slice(5)}`;
  return actionLabels[action] ?? action;
}
