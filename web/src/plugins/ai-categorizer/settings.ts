import { type AiSettings, withAiDefaults } from '../../lib/plugins/ai/settings';

export const PLUGIN_ID = 'ai-categorizer';
export const OTHER = 'Other';

export interface SortedAccount {
  at: number;
  senders: Record<string, string>;
}

export interface CategorizerSettings extends AiSettings {
  categories: string;
  sorted: Record<string, SortedAccount>;
}

const DEFAULT_CATEGORIES = 'Work, Family and friends, Finance, Travel, Shopping, Services';

export function withCategorizerDefaults(stored: Partial<CategorizerSettings> | null): CategorizerSettings {
  const ai = withAiDefaults(stored, 'cloud');
  return {
    ...ai,
    mode: ai.mode === 'rules' ? 'cloud' : ai.mode,
    categories: stored?.categories ?? DEFAULT_CATEGORIES,
    sorted: stored?.sorted ?? {},
  };
}

export function categoryList(settings: Pick<CategorizerSettings, 'categories'>): string[] {
  const names = settings.categories
    .split(',')
    .map((name) => name.trim())
    .filter(Boolean);
  return [...new Set([...names.filter((name) => name.toLowerCase() !== OTHER.toLowerCase()), OTHER])];
}
