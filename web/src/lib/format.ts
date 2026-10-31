const numberFormat = new Intl.NumberFormat();
const dateFormat = new Intl.DateTimeFormat(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
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
  if (seconds < 60) return `${Math.round(seconds)}s`;
  return `${Math.floor(seconds / 60)}m ${Math.round(seconds % 60)}s`;
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

export function actionLabel(action: string | undefined): string {
  if (!action) return '';
  if (action.startsWith('move:')) return `Move to ${action.slice(5)}`;
  return actionLabels[action] ?? action;
}
