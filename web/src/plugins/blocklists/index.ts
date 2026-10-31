import type { CorePlugin } from '../../lib/plugins/types';
import Blocklists from './Blocklists.svelte';
import { PLUGIN_ID } from './settings';

export const blocklists: CorePlugin = {
  id: PLUGIN_ID,
  name: 'Shared blocklists',
  description: 'Checks public lists of spam and tracker domains against your mail on the Rules page, then trashes the matches, adds rules for them or pins them.',
  version: '1.0.0',
  avatar: { icon: 'shield', color: 'green' },
  defaultEnabled: false,
  slots: { rules: Blocklists },
};
