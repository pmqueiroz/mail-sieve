export const PLUGIN_ID = 'blocklists';

export interface BlocklistSettings {
  lists: { url: string }[];
}

export function withBlocklistDefaults(stored: Partial<BlocklistSettings> | null): BlocklistSettings {
  return { lists: stored?.lists ?? [] };
}

export function domainQuery(domain: string): string {
  return `domain:${domain}`;
}
