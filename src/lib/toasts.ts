import { defineStore } from "pinia";

export type ToastKind = "info" | "success" | "warning" | "error";

export interface Toast {
  id: string;
  kind: ToastKind;
  title: string;
  body?: string;
  actionLabel?: string;
  onAction?: () => void;
  timeoutMs?: number | null;
}

const DEFAULT_TIMEOUT_MS: Record<ToastKind, number | null> = {
  info: 5000,
  success: 5000,
  warning: 8000,
  error: null,
};

const MAX_VISIBLE = 4;

export const useToasts = defineStore("toasts", {
  state: () => ({
    toasts: [] as Toast[],
  }),
  getters: {
    visible(): Toast[] {
      return this.toasts.slice(-MAX_VISIBLE);
    },
    overflowCount(): number {
      return Math.max(0, this.toasts.length - MAX_VISIBLE);
    },
  },
  actions: {
    push(toast: Omit<Toast, "id"> & { id?: string }): string {
      const id = toast.id ?? crypto.randomUUID();
      const timeoutMs = toast.timeoutMs !== undefined ? toast.timeoutMs : DEFAULT_TIMEOUT_MS[toast.kind];
      const entry: Toast = { ...toast, id, timeoutMs };
      this.toasts.push(entry);
      if (timeoutMs !== null) {
        setTimeout(() => this.dismiss(id), timeoutMs);
      }
      return id;
    },
    dismiss(id: string): void {
      const idx = this.toasts.findIndex((t) => t.id === id);
      if (idx !== -1) this.toasts.splice(idx, 1);
    },
    clear(): void {
      this.toasts = [];
    },
  },
});
