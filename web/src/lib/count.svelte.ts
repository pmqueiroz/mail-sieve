import { api, errorMessage, withParams } from './api';
import { session } from './session.svelte';
import type { Totals } from './types';

export function liveTotals(accountId: () => number, query: () => string) {
  const state = $state<{ totals: Totals | null; error: string | null }>({ totals: null, error: null });
  $effect(() => {
    const id = accountId();
    const q = query();
    void session.dataVersion;
    let cancelled = false;
    api
      .get<Totals>(withParams(`/accounts/${id}/count`, { q }))
      .then((result) => {
        if (cancelled) return;
        state.totals = result;
        state.error = null;
      })
      .catch((reason) => {
        if (cancelled) return;
        state.totals = null;
        state.error = errorMessage(reason);
      });
    return () => (cancelled = true);
  });
  return state;
}
