import type { CorePlugin } from '../../lib/plugins/types';
import UnsubscribeAudit from './UnsubscribeAudit.svelte';

export const unsubscribeAudit: CorePlugin = {
  id: 'unsubscribe-audit',
  name: 'Unsubscribe audit',
  description: 'Watches senders you unsubscribed from. If one keeps mailing you, Activity lists it and offers a rule that trashes its mail after every sync.',
  version: '1.0.0',
  defaultEnabled: false,
  slots: { activity: UnsubscribeAudit },
};
