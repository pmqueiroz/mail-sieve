import type { AiProviderKind } from '../../types';
import type { CompletionTarget } from '../config';

export type AiMode = 'rules' | 'cloud' | 'docker';
export type DockerRuntime = 'ollama' | 'model-runner';

export interface AiSettings {
  mode: AiMode;
  cloud: { provider: AiProviderKind; base_url: string; model: string };
  docker: { runtime: DockerRuntime; base_url: string; model: string };
}

export const API_KEY = 'api_key';

export const PROVIDERS: Record<AiProviderKind, { label: string; baseUrl: string; model: string; keyRequired: boolean }> = {
  anthropic: { label: 'Anthropic', baseUrl: 'https://api.anthropic.com', model: 'claude-opus-5-5', keyRequired: true },
  openai: { label: 'OpenAI-compatible', baseUrl: 'https://api.openai.com/v1', model: '', keyRequired: false },
};

export const RUNTIMES: Record<DockerRuntime, { label: string; baseUrl: string; model: string; setup: string; pull: string }> = {
  ollama: {
    label: 'Ollama',
    baseUrl: 'http://ollama:11434/v1',
    model: 'llama3.2',
    setup: 'Add an Ollama service next to mailsieve in your compose file and start it:',
    pull: 'docker compose exec ollama ollama pull llama3.2',
  },
  'model-runner': {
    label: 'Docker Model Runner',
    baseUrl: 'http://model-runner.docker.internal/engines/v1',
    model: 'ai/llama3.2',
    setup: 'Turn on Docker Model Runner in Docker Desktop or Docker Engine. Containers reach it at model-runner.docker.internal.',
    pull: 'docker model pull ai/llama3.2',
  },
};

export const OLLAMA_COMPOSE = `services:
  ollama:
    image: ollama/ollama
    volumes:
      - ollama:/root/.ollama

volumes:
  ollama:`;

export const MODE_LABELS: Record<AiMode, string> = {
  rules: 'Built-in rules',
  cloud: 'AI provider',
  docker: 'Local with Docker',
};

export const DEFAULT_AI_SETTINGS: AiSettings = {
  mode: 'rules',
  cloud: { provider: 'anthropic', base_url: PROVIDERS.anthropic.baseUrl, model: PROVIDERS.anthropic.model },
  docker: { runtime: 'ollama', base_url: RUNTIMES.ollama.baseUrl, model: RUNTIMES.ollama.model },
};

export function withAiDefaults(stored: Partial<AiSettings> | null, mode: AiMode = DEFAULT_AI_SETTINGS.mode): AiSettings {
  return {
    mode: stored?.mode ?? mode,
    cloud: { ...DEFAULT_AI_SETTINGS.cloud, ...stored?.cloud },
    docker: { ...DEFAULT_AI_SETTINGS.docker, ...stored?.docker },
  };
}

export function aiTarget(settings: AiSettings, pluginId: string): CompletionTarget | null {
  if (settings.mode === 'cloud') return { kind: settings.cloud.provider, baseUrl: settings.cloud.base_url, model: settings.cloud.model, plugin: pluginId };
  if (settings.mode === 'docker') return { kind: 'openai', baseUrl: settings.docker.base_url, model: settings.docker.model };
  return null;
}
