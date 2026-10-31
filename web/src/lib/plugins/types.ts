import type { Component } from 'svelte';
import type { Rule } from '../types';
import type { PluginAvatarSpec, PluginColor, PluginIcon } from './avatar';
import type { PluginHost } from './host.svelte';

export interface SlotContexts {
  overlay: object;
  topbar: object;
  overview: { accountId: number };
  rules: { accountId: number; rules: Rule[]; reload: () => void };
  search: { accountId: number; query: string; setQuery: (query: string) => void };
  activity: { accountId: number | null };
}

export type SlotName = keyof SlotContexts;

export type Cleanup = void | (() => void);

export interface ContentContext {
  close: () => void;
}

export type SlotProps<K extends SlotName> = ContentContext & SlotContexts[K];

export type MountFn<P extends ContentContext = ContentContext> = (target: HTMLElement, host: PluginHost, context: P) => Cleanup;

export type Content<P extends ContentContext = ContentContext> = Component | Component<P> | { mount: MountFn<P> };

export interface PluginDefinition {
  slots?: { [K in SlotName]?: Content<SlotProps<K>> };
  configure?: Content;
  setup?: (host: PluginHost) => Cleanup;
}

export interface CorePlugin extends PluginDefinition {
  id: string;
  name: string;
  description: string;
  version: string;
  avatar: { icon: PluginIcon; color: PluginColor };
  defaultEnabled: boolean;
}

export type PluginSource = 'core' | 'community';

export interface PluginEntry {
  id: string;
  name: string;
  description: string;
  author: string;
  version: string;
  source: PluginSource;
  avatar: PluginAvatarSpec | null;
  enabled: boolean;
  definition: PluginDefinition | null;
  entryUrl: string | null;
  error: string | null;
}
