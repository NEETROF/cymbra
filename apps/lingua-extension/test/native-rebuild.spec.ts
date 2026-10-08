import { describe, expect, it, vi } from "vitest";
import { followNativeLanguage, type PageSession } from "@/reading/native-rebuild.ts";
import { ROOT_KEY } from "@/state/storage.ts";
import type { StoreChangeReason } from "@/state/store.ts";

// The content script's reading session follows the native language (add-lingua-native-language-choice
// D3): taken down and built anew on the announced change, never on a plain write.

const settle = async (): Promise<void> => {
  for (let i = 0; i < 4; i++) await new Promise((resolve) => setTimeout(resolve, 0));
};

function harness() {
  let listener: ((keys: string[], reason?: StoreChangeReason) => void) | null = null;
  const sessions: (PageSession & { id: number; stopped: boolean })[] = [];
  const build = vi.fn(async () => {
    const session = {
      id: sessions.length + 1,
      stopped: false,
      stop() {
        this.stopped = true;
      },
    };
    sessions.push(session);
    return session;
  });
  return {
    sessions,
    build,
    watch: (fn: NonNullable<typeof listener>): void => {
      listener = fn;
    },
    change: (reason?: StoreChangeReason) => listener?.([ROOT_KEY], reason),
  };
}

describe("the content script's session", () => {
  it("is built anew, the previous one taken down, on a change of native language", async () => {
    const h = harness();
    await followNativeLanguage(h.build, h.watch);
    expect(h.sessions.map((s) => s.id)).toEqual([1]);

    h.change(); // a sync, a status: the session restores it itself
    await settle();
    expect(h.build).toHaveBeenCalledOnce();

    h.change({ type: "native-language", native: "en" });
    await settle();
    expect(h.sessions.map((s) => [s.id, s.stopped])).toEqual([
      [1, true],
      [2, false],
    ]);
  });

  it("builds one after the other when two changes come at once", async () => {
    const h = harness();
    await followNativeLanguage(h.build, h.watch);
    h.change({ type: "native-language", native: "en" });
    h.change({ type: "native-language", native: "fr" });
    await settle();
    expect(h.sessions.map((s) => [s.id, s.stopped])).toEqual([
      [1, true],
      [2, true],
      [3, false],
    ]);
  });

  it("builds a fresh one after a rebuild that failed, and says the failure", async () => {
    const error = vi.spyOn(console, "error").mockImplementation(() => {});
    const h = harness();
    await followNativeLanguage(h.build, h.watch);
    h.build.mockRejectedValueOnce(new Error("no engine"));
    h.change({ type: "native-language", native: "en" });
    await settle();
    expect(error).toHaveBeenCalledOnce();
    h.change({ type: "native-language", native: "en" });
    await settle();
    expect(h.sessions.map((s) => [s.id, s.stopped])).toEqual([
      [1, true],
      [2, false],
    ]);
    error.mockRestore();
  });

  it("watches nothing once the first session could not start", async () => {
    const h = harness();
    const unwatch = vi.fn();
    h.build.mockRejectedValueOnce(new Error("blocked"));
    await expect(
      followNativeLanguage(h.build, (fn) => {
        h.watch(fn);
        return unwatch;
      }),
    ).rejects.toThrow("blocked");
    expect(unwatch).toHaveBeenCalledOnce();
    // A change the listener still hears builds nothing: the next injection starts the reader.
    h.change({ type: "native-language", native: "en" });
    await settle();
    expect(h.build).toHaveBeenCalledOnce();
  });

  it("takes down a first session the change was announced under, once it is up", async () => {
    const h = harness();
    let started!: () => void;
    h.build.mockImplementationOnce(async () => {
      await new Promise<void>((resolve) => (started = resolve));
      const session = {
        id: 1,
        stopped: false,
        stop() {
          this.stopped = true;
        },
      };
      h.sessions.push(session);
      return session;
    });
    const following = followNativeLanguage(h.build, h.watch);

    // The reader chose English while this page's session was still starting on the French backup.
    h.change({ type: "native-language", native: "en" });
    started();
    await following;
    await settle();

    expect(h.sessions.map((s) => [s.id, s.stopped])).toEqual([
      [1, true],
      [2, false],
    ]);
  });

  it("does not rebuild for a change announced under a first session that then failed", async () => {
    const h = harness();
    let fail!: (e: Error) => void;
    h.build.mockImplementationOnce(() => new Promise<never>((_resolve, reject) => (fail = reject)));
    const following = followNativeLanguage(h.build, h.watch);
    h.change({ type: "native-language", native: "en" });
    fail(new Error("blocked"));
    await expect(following).rejects.toThrow("blocked");
    await settle();
    expect(h.build).toHaveBeenCalledOnce();
  });
});
