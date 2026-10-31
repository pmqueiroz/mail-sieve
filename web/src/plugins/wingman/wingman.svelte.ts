import { api, errorMessage } from '../../lib/api';
import { percent } from '../../lib/format';
import { session } from '../../lib/session.svelte';
import { runningSync } from '../../lib/sync';
import { toasts } from '../../lib/toasts.svelte';
import type { Job } from '../../lib/types';
import type { Mood } from './Ghoul.svelte';
import { findSuggestions, type Outcome, type Suggestion } from './suggestions';

const STORAGE_KEY = 'mail-sieve.wingman';
const DAY_MS = 86_400_000;
const DONE_MS = 3000;
const ACK_MS = 2200;
const POKE_MS = 900;

export type Phase = 'idle' | 'suggest' | 'working' | 'done' | 'ack' | 'sleep';

interface Stored {
  napping: boolean;
  snoozed: Record<string, number>;
}

function readStored(): Stored {
  try {
    const parsed = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}') as Partial<Stored>;
    return { napping: parsed.napping === true, snoozed: parsed.snoozed ?? {} };
  } catch {
    return { napping: false, snoozed: {} };
  }
}

class Wingman {
  phase = $state<Phase>('idle');
  current = $state<Suggestion | null>(null);
  message = $state('');
  busy = $state(false);
  poked = $state(false);
  undoable = $state(false);
  job = $state<{ id: number; done: string } | null>(null);
  private queue: Suggestion[] = [];
  private undoAction: (() => Promise<void>) | null = null;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private stored = readStored();
  private generation = 0;

  syncJob = $derived(session.accountId === null ? null : runningSync(session.jobs, session.accountId));

  thinking = $derived.by(() => {
    if (this.phase === 'working') return this.message;
    if (this.phase !== 'idle' || !this.syncJob) return null;
    return this.syncJob.total > 0 ? `Syncing your mail · ${percent(this.syncJob.done, this.syncJob.total)}%` : 'Checking for new mail';
  });

  mood = $derived.by((): Mood => {
    if (this.poked) return 'happy';
    if (this.phase === 'sleep') return 'sleep';
    if (this.phase === 'done') return 'happy';
    if (this.phase === 'suggest') return this.current?.alert ? 'alert' : 'suggest';
    if (this.thinking) return 'think';
    return 'idle';
  });

  start() {
    this.phase = this.stored.napping ? 'sleep' : 'idle';
  }

  stop() {
    this.clearTimer();
    this.generation++;
  }

  async refresh() {
    const account = session.current;
    const generation = ++this.generation;
    if (!account) {
      this.queue = [];
      this.present();
      return;
    }
    const found = await findSuggestions(account, this.syncJob !== null);
    if (generation !== this.generation) return;
    this.queue = found;
    if (this.phase === 'suggest' && !this.pending().some((item) => item.key === this.current?.key)) this.phase = 'idle';
    if (this.phase === 'idle') this.present();
  }

  async accept() {
    const suggestion = this.current;
    if (!suggestion || this.busy) return;
    this.busy = true;
    try {
      this.snooze(suggestion.key, 1);
      this.settle(await suggestion.accept());
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
      this.next();
    } finally {
      this.busy = false;
    }
  }

  decline() {
    const suggestion = this.current;
    if (!suggestion) return;
    this.snooze(suggestion.key, suggestion.snoozeDays);
    this.message = suggestion.ack;
    this.phase = 'ack';
    this.later(ACK_MS, () => this.next());
  }

  watch(job: Job | undefined) {
    const tracked = this.job;
    if (!tracked || !job || job.id !== tracked.id || job.state === 'queued' || job.state === 'running') return;
    this.job = null;
    if (job.state !== 'done') return this.next();
    const jobId = job.id;
    this.showDone(tracked.done, async () => {
      const restore = await api.post<{ job_id: number }>(`/jobs/${jobId}/undo`);
      toasts.track({ jobId: restore.job_id, action: 'restore', count: job.done, who: 'the wingman', undoable: false });
    });
  }

  async undo() {
    const action = this.undoAction;
    const key = this.current?.key;
    this.clearTimer();
    this.undoAction = null;
    this.undoable = false;
    try {
      await action?.();
      if (key) this.unsnooze(key);
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    }
    this.next();
  }

  nap() {
    this.clearTimer();
    this.stored.napping = true;
    this.save();
    this.phase = 'sleep';
  }

  poke() {
    if (this.phase === 'sleep') {
      this.stored.napping = false;
      this.save();
      this.next();
      return;
    }
    this.poked = true;
    setTimeout(() => (this.poked = false), POKE_MS);
  }

  private settle(outcome: Outcome) {
    if (outcome.kind === 'job') {
      toasts.silence(outcome.jobId);
      this.job = { id: outcome.jobId, done: outcome.done };
      this.message = outcome.working;
      this.phase = 'working';
      return;
    }
    this.showDone(outcome.done, outcome.undo);
  }

  private showDone(message: string, undo: (() => Promise<void>) | null) {
    this.message = message;
    this.undoAction = undo;
    this.undoable = undo !== null;
    this.phase = 'done';
    this.later(DONE_MS, () => this.next());
  }

  private next() {
    this.clearTimer();
    this.current = null;
    this.undoAction = null;
    this.undoable = false;
    this.phase = 'idle';
    this.present();
  }

  private present() {
    if (this.stored.napping) {
      this.phase = 'sleep';
      return;
    }
    if (this.phase !== 'idle') return;
    const [first] = this.pending();
    this.current = first ?? null;
    if (first) this.phase = 'suggest';
  }

  private pending() {
    const now = Date.now();
    return this.queue.filter((item) => (this.stored.snoozed[item.key] ?? 0) <= now);
  }

  private snooze(key: string, days: number) {
    this.stored.snoozed[key] = Date.now() + days * DAY_MS;
    this.save();
  }

  private unsnooze(key: string) {
    delete this.stored.snoozed[key];
    this.save();
  }

  private save() {
    const now = Date.now();
    this.stored.snoozed = Object.fromEntries(Object.entries(this.stored.snoozed).filter(([, until]) => until > now));
    localStorage.setItem(STORAGE_KEY, JSON.stringify(this.stored));
  }

  private later(ms: number, fn: () => void) {
    this.clearTimer();
    this.timer = setTimeout(fn, ms);
  }

  private clearTimer() {
    if (this.timer) clearTimeout(this.timer);
    this.timer = null;
  }
}

export const wingman = new Wingman();
