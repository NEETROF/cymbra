import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { TOAST_MS, useToastsStore } from "@/stores/toasts";

describe("toasts store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("pushes success/error toasts with the right variant", () => {
    const toasts = useToastsStore();
    toasts.success("Sample generated.");
    toasts.error("Boom.");
    expect(toasts.items.map((t) => [t.message, t.variant])).toEqual([
      ["Sample generated.", "success"],
      ["Boom.", "error"],
    ]);
  });

  it("auto-dismisses after TOAST_MS", () => {
    const toasts = useToastsStore();
    toasts.success("Gone soon.");
    expect(toasts.items).toHaveLength(1);
    vi.advanceTimersByTime(TOAST_MS - 1);
    expect(toasts.items).toHaveLength(1);
    vi.advanceTimersByTime(1);
    expect(toasts.items).toHaveLength(0);
  });

  it("dismisses on demand and clears its timer (no double-remove)", () => {
    const toasts = useToastsStore();
    const id = toasts.push("Close me.");
    toasts.dismiss(id);
    expect(toasts.items).toHaveLength(0);
    // The auto-dismiss timer must have been cleared — advancing time is a no-op.
    vi.advanceTimersByTime(TOAST_MS * 2);
    expect(toasts.items).toHaveLength(0);
  });

  it("keeps each toast independent (ids are unique)", () => {
    const toasts = useToastsStore();
    const a = toasts.push("A");
    const b = toasts.push("B");
    expect(a).not.toBe(b);
    toasts.dismiss(a);
    expect(toasts.items.map((t) => t.message)).toEqual(["B"]);
  });
});
