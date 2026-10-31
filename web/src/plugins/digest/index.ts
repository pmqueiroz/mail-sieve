import type { CorePlugin } from '../../lib/plugins/types';
import Digest from './Digest.svelte';

export const digest: CorePlugin = {
  id: 'digest',
  name: 'Weekly digest',
  description: 'A This week section on Overview: what arrived, what you cleaned up, which category grew and who sent the most.',
  version: '1.0.0',
  avatar: { icon: 'calendar', color: 'teal' },
  defaultEnabled: false,
  slots: { overview: Digest },
};
