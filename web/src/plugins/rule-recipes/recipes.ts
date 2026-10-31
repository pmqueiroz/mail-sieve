import type { Rule } from '../../lib/types';

export interface Recipe {
  name: string;
  query: string;
  action: string;
}

export const RECIPES: Recipe[] = [
  { name: 'Archive old receipts', query: 'category:receipts older_than:90d', action: 'archive' },
  { name: 'Clear stale promotions', query: 'category:promotions older_than:30d', action: 'trash' },
  { name: 'Read social updates', query: 'category:social is:unread', action: 'mark_read' },
  { name: 'Archive read newsletters', query: 'has:list is:read older_than:14d', action: 'archive' },
  { name: 'Quiet old notifications', query: 'category:notifications is:unread older_than:3d', action: 'mark_read' },
  { name: 'Clear old notifications', query: 'category:notifications older_than:6m', action: 'trash' },
];

export function available(rules: Rule[]): Recipe[] {
  return RECIPES.filter((recipe) => !rules.some((rule) => rule.query.trim() === recipe.query && rule.action === recipe.action));
}
