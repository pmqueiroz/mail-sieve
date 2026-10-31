import type { CorePlugin } from '../../lib/plugins/types';
import RuleRecipes from './RuleRecipes.svelte';

export const ruleRecipes: CorePlugin = {
  id: 'rule-recipes',
  name: 'Rule recipes',
  description: 'Starter rules on the Rules page, such as archiving old receipts or clearing stale promotions, each with a live count of what it would match.',
  version: '1.0.0',
  avatar: { icon: 'list-checks', color: 'orange' },
  defaultEnabled: false,
  slots: { rules: RuleRecipes },
};
