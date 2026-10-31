import type { CorePlugin } from '../../lib/plugins/types';
import Categorizer from './Categorizer.svelte';
import CategorizerSettings from './CategorizerSettings.svelte';
import { PLUGIN_ID } from './settings';

export const aiCategorizer: CorePlugin = {
  id: PLUGIN_ID,
  name: 'AI categorizer',
  description: 'Splits the People category on Overview into your own groups, such as Work or Finance, using an AI model and only headers. Archive or mark a whole group read in one go.',
  version: '1.0.0',
  avatar: { icon: 'sparkles', color: 'pink' },
  defaultEnabled: false,
  slots: { overview: Categorizer },
  configure: CategorizerSettings,
};
