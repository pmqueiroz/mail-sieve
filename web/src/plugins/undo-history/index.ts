import type { CorePlugin } from '../../lib/plugins/types';
import UndoHistory from './UndoHistory.svelte';

export const undoHistory: CorePlugin = {
  id: 'undo-history',
  name: 'Undo history',
  description: 'A button in the top bar that lists your recent actions and rule runs, so you can undo one long after its toast is gone.',
  version: '1.0.0',
  defaultEnabled: false,
  slots: { topbar: UndoHistory },
};
