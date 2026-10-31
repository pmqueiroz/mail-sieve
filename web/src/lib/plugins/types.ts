import type { Component } from 'svelte';
import type { PluginHost } from './host.svelte';

export type SlotName = 'overlay';

export type Cleanup = void | (() => void);

export interface ContentContext {
  close: () => void;
}

export type MountFn = (target: HTMLElement, host: PluginHost, context: ContentContext) => Cleanup;

export type SlotContent = Component | Component<ContentContext> | { mount: MountFn };

export interface PluginDefinition {
  slots?: Partial<Record<SlotName, SlotContent>>;
  configure?: SlotContent;
  setup?: (host: PluginHost) => Cleanup;
}

export interface CorePlugin extends PluginDefinition {
  id: string;
  name: string;
  description: string;
  version: string;
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
  enabled: boolean;
  definition: PluginDefinition | null;
  entryUrl: string | null;
  error: string | null;
}
