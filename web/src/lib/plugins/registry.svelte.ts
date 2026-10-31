import { CORE_PLUGINS } from '../../plugins';
import { api, errorMessage } from '../api';
import type { CommunityPluginManifest, PluginList } from '../types';
import { host } from './host.svelte';
import type { Cleanup, Content, PluginDefinition, PluginEntry, SlotName, SlotProps } from './types';

function isDefinition(value: unknown): value is PluginDefinition {
  if (!value || typeof value !== 'object') return false;
  const candidate = value as PluginDefinition;
  return (candidate.slots === undefined || typeof candidate.slots === 'object') && (candidate.setup === undefined || typeof candidate.setup === 'function');
}

async function importCommunity(url: string): Promise<PluginDefinition> {
  const module = await import(/* @vite-ignore */ url);
  if (!isDefinition(module.default)) throw new Error('the module does not export a plugin as its default export');
  return module.default;
}

function communityEntry(manifest: CommunityPluginManifest, enabled: boolean): PluginEntry {
  return {
    id: manifest.id,
    name: manifest.name,
    description: manifest.description,
    author: manifest.author,
    version: manifest.version,
    source: 'community',
    avatar: manifest.avatar,
    enabled,
    definition: null,
    entryUrl: manifest.entry_url,
    error: manifest.error,
  };
}

class Plugins {
  entries = $state<PluginEntry[]>([]);
  loaded = $state(false);
  active = $derived(this.entries.filter((entry) => entry.enabled && entry.definition && !entry.error));
  private cleanups = new Map<string, () => void>();

  async load() {
    const list = await api.get<PluginList>('/plugins');
    const core: PluginEntry[] = CORE_PLUGINS.map((plugin) => ({
      id: plugin.id,
      name: plugin.name,
      description: plugin.description,
      author: 'mailsieve',
      version: plugin.version,
      source: 'core',
      avatar: plugin.avatar,
      enabled: list.states[plugin.id] ?? plugin.defaultEnabled,
      definition: plugin,
      entryUrl: null,
      error: null,
    }));
    const coreIds = new Set(core.map((entry) => entry.id));
    const community = list.community.filter((manifest) => !coreIds.has(manifest.id)).map((manifest) => communityEntry(manifest, list.states[manifest.id] ?? false));
    this.entries = [...core, ...community];
    await Promise.all(this.entries.filter((entry) => entry.enabled).map((entry) => this.activate(entry.id)));
    this.loaded = true;
  }

  slot<K extends SlotName>(name: K): { id: string; content: Content<SlotProps<K>> }[] {
    return this.active.flatMap((entry) => {
      const content = entry.definition?.slots?.[name] as Content<SlotProps<K>> | undefined;
      return content ? [{ id: entry.id, content }] : [];
    });
  }

  async setEnabled(id: string, enabled: boolean) {
    await api.put(`/plugins/${id}`, { enabled });
    const entry = this.find(id);
    if (!entry) return;
    entry.enabled = enabled;
    if (enabled) await this.activate(id);
    else this.deactivate(id);
  }

  fail(id: string, error: unknown) {
    const entry = this.find(id);
    if (entry) entry.error = errorMessage(error);
    this.deactivate(id);
  }

  private find(id: string) {
    return this.entries.find((entry) => entry.id === id);
  }

  private async activate(id: string) {
    const entry = this.find(id);
    if (!entry || entry.error) return;
    try {
      if (!entry.definition && entry.entryUrl) entry.definition = await importCommunity(entry.entryUrl);
      const cleanup: Cleanup = entry.definition?.setup?.(host);
      if (cleanup) this.cleanups.set(id, cleanup);
    } catch (error) {
      this.fail(id, error);
    }
  }

  private deactivate(id: string) {
    this.cleanups.get(id)?.();
    this.cleanups.delete(id);
  }
}

export const plugins = new Plugins();
