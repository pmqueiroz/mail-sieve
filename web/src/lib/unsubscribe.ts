import type { PillTone } from './components/Pill.svelte';
import type { Group } from './types';

export function unsubscribeBadge(group: Group): { tone: PillTone; label: string } | null {
  if (group.unsubscribe_status === 'completed') return { tone: 'muted', label: 'Unsubscribed' };
  if (group.unsubscribe_status === 'link_opened') return { tone: 'warn', label: 'Link opened' };
  if (group.unsubscribe_status === 'failed') return { tone: 'bad', label: 'Unsubscribe failed' };
  if (group.unsub_one_click) return { tone: 'good', label: 'One-click' };
  if (group.unsub_http || group.unsub_mailto) return { tone: 'neutral', label: 'Unsubscribe' };
  return null;
}
