import type { CorePlugin } from '../lib/plugins/types';
import { digest } from './digest';
import { ruleRecipes } from './rule-recipes';
import { undoHistory } from './undo-history';
import { wingman } from './wingman';

export const CORE_PLUGINS: CorePlugin[] = [wingman, undoHistory, digest, ruleRecipes];
