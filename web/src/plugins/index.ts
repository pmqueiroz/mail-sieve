import type { CorePlugin } from '../lib/plugins/types';
import { digest } from './digest';
import { undoHistory } from './undo-history';
import { wingman } from './wingman';

export const CORE_PLUGINS: CorePlugin[] = [wingman, undoHistory, digest];
