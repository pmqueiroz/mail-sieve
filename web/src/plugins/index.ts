import type { CorePlugin } from '../lib/plugins/types';
import { undoHistory } from './undo-history';
import { wingman } from './wingman';

export const CORE_PLUGINS: CorePlugin[] = [wingman, undoHistory];
