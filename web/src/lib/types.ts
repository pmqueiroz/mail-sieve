export interface AuthStatus {
  configured: boolean;
  authenticated: boolean;
  version: string;
  oauth: { google: boolean; microsoft: boolean };
}

export interface Account {
  id: number;
  label: string;
  email: string;
  imap_host: string;
  imap_port: number;
  imap_tls: boolean;
  smtp_host: string | null;
  smtp_port: number | null;
  username: string;
  auth_kind: 'password' | 'oauth2';
  oauth_provider: string | null;
  provider: 'generic' | 'gmail';
  max_connections: number;
  capabilities: string;
  sync_interval_minutes: number;
  created_at: number;
  last_synced_at: number | null;
  last_error: string | null;
  message_count: number;
  total_size: number;
}

export interface Mailbox {
  id: number;
  account_id: number;
  name: string;
  display_name: string;
  delimiter: string | null;
  special_use: string | null;
  sync_enabled: boolean;
  message_count: number;
  last_synced_at: number | null;
}

export type GroupBy = 'sender' | 'domain' | 'list';
export type SortBy = 'count' | 'size' | 'unread' | 'recent';

export type PinKind = 'sender' | 'domain';

export interface Pin {
  kind: PinKind;
  key: string;
}

export interface Group {
  key: string;
  name: string | null;
  sender_count: number;
  message_count: number;
  unread_count: number;
  total_size: number;
  last_date: number | null;
  list_id: string | null;
  unsub_http: string | null;
  unsub_mailto: string | null;
  unsub_one_click: boolean;
  unsubscribe_status: string | null;
  pinned_count: number;
  pin: Pin | null;
}

export interface MessageRow {
  id: number;
  mailbox_id: number;
  mailbox: string;
  uid: number;
  from_addr: string;
  from_name: string | null;
  subject: string | null;
  date: number;
  size: number;
  unread: boolean;
  flagged: boolean;
  list_id: string | null;
  has_unsubscribe: boolean;
  pin: Pin | null;
}

export interface MessagePage {
  messages: MessageRow[];
  next: { before_date: number; before_id: number } | null;
}

export interface Totals {
  count: number;
  bytes: number;
  pinned: number;
}

export interface ActionResponse {
  job_id: number | null;
  skipped_pinned: number;
}

export type Category = 'newsletters' | 'notifications' | 'promotions' | 'receipts' | 'social' | 'people';

export interface Overview {
  total: number;
  total_bytes: number;
  unread: number;
  unread_week: number;
  list_messages: number;
  sender_count: number;
  never_opened: { senders: number; messages: number; bytes: number };
  freed_week_bytes: number;
  categories: { category: Category; count: number }[];
}

export interface PendingLink {
  from_addr: string;
  name: string | null;
  url: string | null;
  at: number;
}

export interface StillSending {
  from_addr: string;
  name: string | null;
  method: string;
  unsubscribed_at: number;
  since_count: number;
  last_date: number;
}

export interface BlocklistMatch {
  domain: string;
  messages: number;
  sender_domains: string[];
}

export interface BlocklistPreview {
  listed: number;
  matches: BlocklistMatch[];
}

export type JobState = 'queued' | 'running' | 'done' | 'failed' | 'cancelled';

export interface Job {
  id: number;
  account_id: number | null;
  kind: 'sync' | 'action' | 'undo';
  state: JobState;
  payload: { action?: string; origin?: string; undo_of?: number; undone_by?: number };
  total: number;
  done: number;
  error: string | null;
  created_at: number;
  started_at: number | null;
  finished_at: number | null;
}

export interface Rule {
  id: number;
  account_id: number;
  name: string;
  query: string;
  action: string;
  enabled: boolean;
  interval_minutes: number | null;
  run_after_sync: boolean;
  last_run_at: number | null;
  created_at: number;
}

export type RuleConflict = 'skip' | 'replace';

export interface RuleImportReport {
  added: string[];
  existing: string[];
  invalid: { name: string; error: string }[];
  applied: boolean;
}

export interface Preview {
  subject: string | null;
  from: string[];
  to: string[];
  date: number | null;
  text: string;
  attachments: string[];
  size: number;
}

export type UnsubscribeOutcome =
  | { status: 'completed'; method: string; detail: string }
  | { status: 'open_link'; url: string }
  | { status: 'failed'; error: string };

export type ServerEvent = { type: 'job'; job: Job } | { type: 'account'; account_id: number };

export interface CommunityPluginManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  avatar: { icon?: string | null; color?: string | null } | null;
  entry_url: string | null;
  error: string | null;
}

export interface PluginList {
  states: Record<string, boolean>;
  community: CommunityPluginManifest[];
}

export interface PluginConfig<T = unknown> {
  settings: T | null;
  secrets: string[];
}

export type AiProviderKind = 'anthropic' | 'openai';
