export type ToastKind = 'info' | 'success' | 'error';

export interface MessageToast {
  id: number;
  type: 'message';
  kind: ToastKind;
  message: string;
}

export interface JobToast {
  id: number;
  type: 'job';
  jobId: number;
  action: string;
  count: number | null;
  who: string;
  undoable: boolean;
}

export type Toast = MessageToast | JobToast;

const MESSAGE_TIMEOUT = 5000;
const FINISHED_JOB_TIMEOUT = 8000;

class Toasts {
  items = $state<Toast[]>([]);
  raised = $state(false);
  private next = 1;
  private expiring = new Set<number>();

  show(message: string, kind: ToastKind = 'info', timeout = MESSAGE_TIMEOUT) {
    const id = this.next++;
    this.items.push({ id, type: 'message', kind, message });
    if (timeout > 0) setTimeout(() => this.dismiss(id), timeout);
  }

  track(job: Omit<JobToast, 'id' | 'type'>) {
    this.items = this.items.filter((toast) => toast.type !== 'job');
    this.items.push({ id: this.next++, type: 'job', ...job });
  }

  tracks(jobId: number): boolean {
    return this.items.some((toast) => toast.type === 'job' && toast.jobId === jobId);
  }

  expire(id: number) {
    if (this.expiring.has(id)) return;
    this.expiring.add(id);
    setTimeout(() => this.dismiss(id), FINISHED_JOB_TIMEOUT);
  }

  dismiss(id: number) {
    this.expiring.delete(id);
    this.items = this.items.filter((toast) => toast.id !== id);
  }
}

export const toasts = new Toasts();
