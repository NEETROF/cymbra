import { ref } from "vue";
import { defineStore } from "pinia";

// Transient, action-result notifications shown as dismissible toasts (a single
// ToastHost renders them, mounted in App.vue). Any store/view pushes one via
// `success`/`error`/`push`; each auto-dismisses after TOAST_MS and carries an
// explicit close. Ephemeral UI state — never persisted.

export type ToastVariant = "success" | "error" | "info";

export interface Toast {
  readonly id: number;
  readonly message: string;
  readonly variant: ToastVariant;
}

/** Auto-dismiss delay in milliseconds. */
export const TOAST_MS = 3000;

export const useToastsStore = defineStore("toasts", () => {
  const items = ref<Toast[]>([]);
  let seq = 0;
  const timers = new Map<number, ReturnType<typeof setTimeout>>();

  /** Remove a toast (called by the auto-dismiss timer and the close button). */
  function dismiss(id: number): void {
    const timer = timers.get(id);
    if (timer) {
      clearTimeout(timer);
      timers.delete(id);
    }
    items.value = items.value.filter((t) => t.id !== id);
  }

  /** Show a toast; returns its id. Auto-dismisses after TOAST_MS. */
  function push(message: string, variant: ToastVariant = "info"): number {
    const id = ++seq;
    items.value = [...items.value, { id, message, variant }];
    timers.set(
      id,
      setTimeout(() => dismiss(id), TOAST_MS),
    );
    return id;
  }

  const success = (message: string) => push(message, "success");
  const error = (message: string) => push(message, "error");

  return { items, push, success, error, dismiss };
});
