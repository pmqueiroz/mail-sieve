import type { CorePlugin } from '../lib/plugins/types';
import { digest } from './digest';
import { queryBuilder } from './query-builder';
import { ruleRecipes } from './rule-recipes';
import { undoHistory } from './undo-history';
import { unsubscribeAudit } from './unsubscribe-audit';
import { wingman } from './wingman';

export const CORE_PLUGINS: CorePlugin[] = [wingman, undoHistory, digest, ruleRecipes, queryBuilder, unsubscribeAudit];
