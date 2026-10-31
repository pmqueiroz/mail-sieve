import { api, errorMessage, onUnauthorized } from './api';
import { formatNumber } from './format';
import { toasts } from './toasts.svelte';
import type { Account, AuthStatus, Job, ServerEvent } from './types';

const ACCOUNT_KEY = 'mail-sieve.account';

class Session {
  status = $state<AuthStatus | null>(null);
  accounts = $state<Account[]>([]);
  accountId = $state<number | null>(null);
  jobs = $state<Job[]>([]);
  dataVersion = $state(0);
  current = $derived(this.accounts.find((account) => account.id === this.accountId) ?? null);
  activeJobs = $derived(this.jobs.filter((job) => job.state === 'queued' || job.state === 'running'));
  private events: EventSource | null = null;
  private accountRefresh: ReturnType<typeof setTimeout> | null = null;

  constructor() {
    onUnauthorized(() => {
      if (this.status) this.status = { ...this.status, authenticated: false };
      this.disconnect();
    });
  }

  async boot() {
    this.status = await api.get<AuthStatus>('/auth/status');
    if (this.status.authenticated) await this.start();
  }

  async start() {
    await Promise.all([this.refreshAccounts(), this.refreshJobs()]);
    this.connect();
  }

  async refreshAccounts() {
    this.accounts = await api.get<Account[]>('/accounts');
    const stored = Number(localStorage.getItem(ACCOUNT_KEY));
    const preferred = this.accounts.find((account) => account.id === (this.accountId ?? stored));
    this.selectAccount(preferred?.id ?? this.accounts[0]?.id ?? null);
  }

  async refreshJobs() {
    this.jobs = await api.get<Job[]>('/jobs?limit=100');
  }

  selectAccount(id: number | null) {
    this.accountId = id;
    if (id !== null) localStorage.setItem(ACCOUNT_KEY, String(id));
  }

  async logout() {
    await api.post('/auth/logout').catch(() => undefined);
    this.disconnect();
    this.accounts = [];
    this.jobs = [];
    if (this.status) this.status = { ...this.status, authenticated: false };
  }

  private connect() {
    this.disconnect();
    const events = new EventSource('/api/events');
    events.onmessage = (message) => this.handle(JSON.parse(message.data) as ServerEvent);
    this.events = events;
  }

  private disconnect() {
    this.events?.close();
    this.events = null;
  }

  private handle(event: ServerEvent) {
    if (event.type === 'job') {
      const index = this.jobs.findIndex((job) => job.id === event.job.id);
      const previous = index >= 0 ? this.jobs[index] : undefined;
      if (index >= 0) this.jobs[index] = event.job;
      else this.jobs.unshift(event.job);
      const finished = previous?.state !== event.job.state && ['done', 'failed', 'cancelled'].includes(event.job.state);
      if (finished) this.onJobFinished(event.job);
      else if (event.job.kind === 'sync' && event.job.done > 0 && event.job.done % 2000 < 500) this.dataVersion++;
    } else {
      this.scheduleAccountRefresh();
    }
  }

  private onJobFinished(job: Job) {
    this.dataVersion++;
    this.scheduleAccountRefresh();
    if (job.state === 'failed') toasts.show(`Job #${job.id} failed: ${job.error ?? 'unknown error'}`, 'error', 10000);
    if (job.state === 'done' && job.kind === 'action') toasts.show(`Done: ${formatNumber(job.done)} messages updated.`, 'success');
  }

  private scheduleAccountRefresh() {
    if (this.accountRefresh) clearTimeout(this.accountRefresh);
    this.accountRefresh = setTimeout(() => {
      this.refreshAccounts().catch((error) => toasts.show(errorMessage(error), 'error'));
    }, 400);
  }
}

export const session = new Session();
