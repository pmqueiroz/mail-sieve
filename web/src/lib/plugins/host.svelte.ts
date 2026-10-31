import { runAction } from '../actions';
import { api } from '../api';
import { type Route, router } from '../router.svelte';
import { session } from '../session.svelte';
import { toasts } from '../toasts.svelte';
import { complete, listModels, loadPluginConfig, savePluginConfig } from './config';
import type { Account, Job } from '../types';

export const HOST_API_VERSION = 1;

export interface HostSnapshot {
  route: Route;
  accountId: number | null;
  account: Account | null;
  dataVersion: number;
}

function snapshot(): HostSnapshot {
  return { route: router.route, accountId: session.accountId, account: session.current, dataVersion: session.dataVersion };
}

export const host = {
  apiVersion: HOST_API_VERSION,
  api,
  runAction,
  config: { load: loadPluginConfig, save: savePluginConfig },
  ai: { listModels, complete },
  toast: (message: string, kind: 'info' | 'success' | 'error' = 'info') => toasts.show(message, kind),
  go: (route: Route, params: Record<string, string> = {}) => router.go(route, params),
  get route(): Route {
    return router.route;
  },
  get accountId(): number | null {
    return session.accountId;
  },
  get account(): Account | null {
    return session.current;
  },
  get accounts(): Account[] {
    return session.accounts;
  },
  get jobs(): Job[] {
    return session.jobs;
  },
  subscribe(listener: (state: HostSnapshot) => void): () => void {
    return $effect.root(() => {
      $effect(() => listener(snapshot()));
    });
  },
};

export type PluginHost = typeof host;
