import { afterEach, describe, expect, it, vi } from "vitest";
import { INTERFACE_LANGUAGE_READ_TIMEOUT_MS, interfaceLanguage } from "@/i18n/language.ts";

// A storage read that never settles (localise-lingua-account-onboarding): the surface shows French
// after a bounded wait rather than staying hidden behind `data-copy-pending`.

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

describe("a read that hangs", () => {
  it("answers French after the timeout, with a [Cymbra Lingua] warning", async () => {
    vi.useFakeTimers();
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const never = { get: () => new Promise<Record<string, unknown>>(() => {}) };
    let answer: string | null = null;
    void interfaceLanguage(never).then((language) => (answer = language));
    await vi.advanceTimersByTimeAsync(INTERFACE_LANGUAGE_READ_TIMEOUT_MS - 1);
    expect(answer).toBeNull();
    await vi.advanceTimersByTimeAsync(1);
    expect(answer).toBe("fr");
    expect(INTERFACE_LANGUAGE_READ_TIMEOUT_MS).toBeLessThanOrEqual(500);
    expect(warn).toHaveBeenCalledTimes(1);
    expect(warn.mock.calls[0]![0]).toMatch(/^\[Cymbra Lingua\] /);
  });

  it("a read that answers in time is not cut short, and leaves no timer behind", async () => {
    vi.useFakeTimers();
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const area = { get: async () => ({ "cymbra-lingua-interface-language": "en" }) };
    expect(await interfaceLanguage(area)).toBe("en");
    expect(vi.getTimerCount()).toBe(0);
    expect(warn).not.toHaveBeenCalled();
  });
});
