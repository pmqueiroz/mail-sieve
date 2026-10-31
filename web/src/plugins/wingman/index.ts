import type { CorePlugin } from '../../lib/plugins/types';
import Wingman from './Wingman.svelte';
import WingmanSettings from './WingmanSettings.svelte';

export const wingman: CorePlugin = {
  id: 'wingman',
  name: 'Ghoul wingman',
  description: 'A small ghoul that floats over the app, spots mail you never open and offers to clean it up. Click the ghoul to poke it, or send it to nap.',
  version: '1.0.0',
  defaultEnabled: true,
  slots: { overlay: Wingman },
  configure: WingmanSettings,
};
