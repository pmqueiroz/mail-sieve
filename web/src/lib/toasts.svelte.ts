export type ToastKind = 'info' | 'success' | 'error';

export interface Toast {
  id: number;
  kind: ToastKind;
  message: string;
}

class Toasts {
  items = $state<Toast[]>([]);
  private next = 1;

  show(message: string, kind: ToastKind = 'info', timeout = 5000) {
    const id = this.next++;
    this.items.push({ id, kind, message });
    if (timeout > 0) setTimeout(() => this.dismiss(id), timeout);
  }

  dismiss(id: number) {
    this.items = this.items.filter((toast) => toast.id !== id);
  }
}

export const toasts = new Toasts();
