import type { CorePlugin } from '../../lib/plugins/types';
import QueryBuilder from './QueryBuilder.svelte';

export const queryBuilder: CorePlugin = {
  id: 'query-builder',
  name: 'Query builder',
  description: 'Shows each search term as a chip under the search bar on Senders and Messages. Click a chip to exclude it, or add filters without typing the syntax.',
  version: '1.0.0',
  avatar: { icon: 'search', color: 'yellow' },
  defaultEnabled: false,
  slots: { search: QueryBuilder },
};
