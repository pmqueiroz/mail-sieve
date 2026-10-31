import { api, withParams } from './api';
import { formatNumber } from './format';
import type { Pin, PinKind } from './types';

export async function setPinned(accountId: number, kind: PinKind, key: string, pinned: boolean): Promise<Pin | null> {
  if (pinned) return api.post<Pin>(`/accounts/${accountId}/pins`, { kind, key });
  await api.delete(withParams(`/accounts/${accountId}/pins`, { kind, key }));
  return null;
}

export function skippedNote(skipped: number): string {
  return `${formatNumber(skipped)} from pinned senders ${skipped === 1 ? 'is' : 'are'} skipped`;
}

export function pinDescription(pin: Pin): string {
  return pin.kind === 'domain' ? `Pinned domain ${pin.key}` : 'Pinned sender';
}
