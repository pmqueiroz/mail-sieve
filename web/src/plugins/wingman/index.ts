import type { CorePlugin } from '../../lib/plugins/types';
import Wingman from './Wingman.svelte';
import WingmanSettings from './WingmanSettings.svelte';

export const wingman: CorePlugin = {
  id: 'wingman',
  name: 'Ghoul wingman',
  description: 'Your AI mail assistant. A friendly ghoul floats over the app, reads your mailbox and suggests what to clean up next, one tap at a time. Connect Claude, any OpenAI-compatible model or a local one, or let it run on built-in rules.',
  version: '1.0.0',
  defaultEnabled: false,
  slots: { overlay: Wingman },
  configure: WingmanSettings,
};
