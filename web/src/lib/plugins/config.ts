import { api } from '../api';
import type { AiProviderKind, PluginConfig } from '../types';

export type SecretChanges = Record<string, string | null>;

export function loadPluginConfig<T>(id: string): Promise<PluginConfig<T>> {
  return api.get<PluginConfig<T>>(`/plugins/${id}/config`);
}

export function savePluginConfig<T>(id: string, settings: T, secrets: SecretChanges = {}): Promise<PluginConfig<T>> {
  return api.put<PluginConfig<T>>(`/plugins/${id}/config`, { settings, secrets });
}

export interface ModelQuery {
  kind: AiProviderKind;
  baseUrl: string;
  apiKey?: string;
  plugin?: string;
}

export async function listModels(query: ModelQuery): Promise<string[]> {
  const { models } = await api.post<{ models: string[] }>('/ai/models', {
    kind: query.kind,
    base_url: query.baseUrl,
    api_key: query.apiKey || null,
    plugin: query.plugin ?? null,
  });
  return models;
}
