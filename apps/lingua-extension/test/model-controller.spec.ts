import { describe, expect, it, vi } from "vitest";
import { ModelController, type ModelHostAccess } from "@/translate/host/model-controller.ts";
import type { ModelManifest } from "@/translate/host/model-manifest.ts";
import { MODEL_STATE_KEY, type SettingArea, TRANSLATION_HOST_KEY } from "@/translate/setting.ts";

// « Traduction étendue » as the background runs it — the scenarios of
// specs/lingua-translation/spec.md (add-lingua-translation-delivery), driven through fakes: the
// device's storage, the engine's host, and the model's database.

const manifest: ModelManifest = {
  version: "en-fr/base-memory/2.0",
  base: "https://models.example/",
  files: {
    model: { path: "m.gz", size: 700, sha256: "a".repeat(64) },
    lex: { path: "l.gz", size: 200, sha256: "b".repeat(64) },
    vocab: { path: "v.gz", size: 100, sha256: "c".repeat(64) },
  },
};

function memoryArea(seed: Record<string, unknown> = {}): SettingArea & { store: Record<string, unknown> } {
  const store: Record<string, unknown> = { ...seed };
  return {
    store,
    get: async (keys) => {
      const out: Record<string, unknown> = {};
      for (const k of Array.isArray(keys) ? keys : [keys]) if (k in store) out[k] = store[k];
      return out;
    },
    set: async (items) => void Object.assign(store, structuredClone(items)),
  };
}

function setup(
  opts: { seed?: Record<string, unknown>; offered?: boolean; stored?: boolean; downloading?: boolean } = {},
) {
  const area = memoryArea(opts.seed);
  let downloading = opts.downloading ?? false;
  let stored = opts.stored ?? false;
  const host = {
    startDownload: vi.fn(async () => void (downloading = true)),
    cancelDownload: vi.fn(async () => void (downloading = false)),
    downloading: vi.fn(async () => downloading),
    shutDown: vi.fn(async () => {}),
  } satisfies ModelHostAccess;
  const db = {
    complete: vi.fn(async () => stored),
    erase: vi.fn(async () => void (stored = false)),
  };
  const log = vi.fn();
  const controller = new ModelController({
    area,
    host,
    db,
    manifest: async () => manifest,
    offered: async () => opts.offered ?? true,
    log,
  });
  return {
    controller,
    area,
    host,
    db,
    log,
    stored: (v: boolean) => (stored = v),
    stopDownloading: () => (downloading = false),
    setting: () => ({ host: area.store[TRANSLATION_HOST_KEY], state: area.store[MODEL_STATE_KEY] }),
  };
}

const ON_READY = { [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: { phase: "ready" } };

describe("ModelController", () => {
  it("is off on a fresh install, and says nothing is downloaded", async () => {
    const { controller, host } = setup();
    await expect(controller.status()).resolves.toEqual({ offered: true, host: "none", state: { phase: "absent" } });
    expect(host.startDownload).not.toHaveBeenCalled();
    expect(await controller.ready()).toBe(false);
  });

  describe("turning it on", () => {
    it("records the choice and starts the download, stating its size", async () => {
      const { controller, host, setting } = setup();
      const status = await controller.enable();
      expect(host.startDownload).toHaveBeenCalledOnce();
      expect(setting()).toEqual({ host: "local", state: { phase: "downloading", received: 0, total: 1000 } });
      expect(status).toMatchObject({ offered: true, host: "local", state: { phase: "downloading" } });
    });

    it("follows the download's progress, then its completion", async () => {
      const { controller, setting } = setup();
      await controller.enable();
      await controller.onEvent({ kind: "progress", received: 400, total: 1000 });
      expect(setting().state).toEqual({ phase: "downloading", received: 400, total: 1000 });
      await controller.onEvent({ kind: "done" });
      expect(setting().state).toEqual({ phase: "ready" });
      expect(await controller.ready()).toBe(true);
    });

    it("never loads the engine: the first translation asked does (D6)", async () => {
      const { controller, host } = setup();
      await controller.enable();
      await controller.onEvent({ kind: "done" });
      expect(host.shutDown).not.toHaveBeenCalled();
      // The controller has no way to load it at all — only startDownload was ever asked of the host.
      expect(
        Object.keys(host).filter(
          (k) => (host as Record<string, { mock?: { calls: unknown[] } }>)[k]!.mock?.calls.length,
        ),
      ).toEqual(["startDownload"]);
    });

    it("records a failure for the setting to explain, and leaves it on", async () => {
      const { controller, setting } = setup();
      await controller.enable();
      await controller.onEvent({ kind: "failed", reason: "not-the-model" });
      expect(setting()).toEqual({ host: "local", state: { phase: "failed", reason: "not-the-model" } });
      expect(await controller.ready()).toBe(false);
    });

    it("records a download that could not even start as failed", async () => {
      const { controller, host, setting } = setup();
      host.startDownload.mockRejectedValueOnce(new Error("offscreen refused"));
      await controller.enable();
      expect(setting().state).toEqual({ phase: "failed", reason: "unknown" });
    });

    it("does nothing where the setting is not offered — Firefox for Android (D8)", async () => {
      const { controller, host, setting } = setup({ offered: false });
      await expect(controller.enable()).resolves.toMatchObject({ offered: false, host: "none" });
      expect(host.startDownload).not.toHaveBeenCalled();
      expect(setting().host).toBeUndefined();
    });

    it("does not start a second download when it is already on", async () => {
      const { controller, host } = setup();
      await controller.enable();
      await controller.enable();
      expect(host.startDownload).toHaveBeenCalledOnce();
    });
  });

  describe("turning it off", () => {
    it("stops the engine and the download and deletes the model", async () => {
      const { controller, host, db, setting } = setup({ seed: ON_READY, stored: true });
      const status = await controller.disable();
      expect(host.cancelDownload).toHaveBeenCalled();
      expect(host.shutDown).toHaveBeenCalled();
      expect(db.erase).toHaveBeenCalled();
      expect(setting()).toEqual({ host: "none", state: { phase: "absent" } });
      expect(status).toEqual({ offered: true, host: "none", state: { phase: "absent" } });
    });

    it("cancelling a download keeps nothing and leaves the setting off", async () => {
      const { controller, db, setting } = setup();
      await controller.enable();
      await controller.onEvent({ kind: "progress", received: 400, total: 1000 });
      await controller.disable();
      expect(db.erase).toHaveBeenCalled();
      expect(setting().host).toBe("none");
    });

    it("ignores a report that arrives after it: a setting that is off stays off", async () => {
      const { controller, setting } = setup();
      await controller.enable();
      await controller.disable();
      await controller.onEvent({ kind: "progress", received: 900, total: 1000 });
      await controller.onEvent({ kind: "done" });
      expect(setting()).toEqual({ host: "none", state: { phase: "absent" } });
    });

    it("runs commands in the order they came: off right after on is not overtaken", async () => {
      const { controller, setting } = setup();
      const on = controller.enable();
      const report = controller.onEvent({ kind: "done" });
      const off = controller.disable();
      await Promise.all([on, report, off]);
      expect(setting()).toEqual({ host: "none", state: { phase: "absent" } });
    });

    it("finishes even when the host or the database fails, and says why in the log", async () => {
      const { controller, host, db, log, setting } = setup({ seed: ON_READY });
      host.shutDown.mockRejectedValueOnce(new Error("no document"));
      db.erase.mockRejectedValueOnce(new Error("blocked"));
      await controller.disable();
      expect(setting().host).toBe("none");
      expect(log).toHaveBeenCalledTimes(2);
    });

    it("turning it on again downloads again", async () => {
      const { controller, host } = setup({ seed: ON_READY, stored: true });
      await controller.disable();
      await controller.enable();
      expect(host.startDownload).toHaveBeenCalledOnce();
    });
  });

  describe("nothing restarts on its own", () => {
    it("a download its host dropped is interrupted, and waits for the reader", async () => {
      const { controller, host, stopDownloading, setting } = setup();
      await controller.enable();
      await controller.onEvent({ kind: "progress", received: 400, total: 1000 });
      stopDownloading(); // the event page was torn down
      await expect(controller.status()).resolves.toMatchObject({
        state: { phase: "interrupted", received: 400, total: 1000 },
      });
      expect(setting().state).toEqual({ phase: "interrupted", received: 400, total: 1000 });
      expect(host.startDownload).toHaveBeenCalledOnce(); // not again
    });

    it("a download still running stays downloading", async () => {
      const { controller } = setup();
      await controller.enable();
      await expect(controller.status()).resolves.toMatchObject({ state: { phase: "downloading" } });
    });

    it("a model the browser removed is reported removed, and nothing is fetched", async () => {
      const { controller, host, setting } = setup({ seed: ON_READY, stored: false });
      await expect(controller.status()).resolves.toEqual({ offered: true, host: "local", state: { phase: "removed" } });
      expect(setting().state).toEqual({ phase: "removed" });
      expect(host.startDownload).not.toHaveBeenCalled();
      expect(await controller.ready()).toBe(false);
    });

    it("a model that cannot be read counts as removed", async () => {
      const { controller, db, log } = setup({ seed: ON_READY });
      db.complete.mockRejectedValueOnce(new Error("IndexedDB unavailable"));
      await expect(controller.status()).resolves.toMatchObject({ state: { phase: "removed" } });
      expect(log).toHaveBeenCalled();
    });

    it("a model still stored stays ready", async () => {
      const { controller } = setup({ seed: ON_READY, stored: true });
      await expect(controller.status()).resolves.toMatchObject({ state: { phase: "ready" } });
    });

    it("on, with its state lost, is removed — never downloaded behind the reader's back", async () => {
      const { controller, host } = setup({ seed: { [TRANSLATION_HOST_KEY]: "local" } });
      await expect(controller.status()).resolves.toMatchObject({ state: { phase: "removed" } });
      expect(host.startDownload).not.toHaveBeenCalled();
    });

    it("off, with a leftover state, is tidied to absent", async () => {
      const { controller, setting } = setup({ seed: { [MODEL_STATE_KEY]: { phase: "ready" } } });
      await controller.status();
      expect(setting().state).toEqual({ phase: "absent" });
    });

    it("a failure stays failed until the reader asks", async () => {
      const { controller } = setup({
        seed: { [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: { phase: "failed", reason: "network" } },
      });
      await expect(controller.status()).resolves.toMatchObject({ state: { phase: "failed", reason: "network" } });
    });
  });

  describe("resume — try again, resume, download again", () => {
    for (const state of [
      { phase: "failed", reason: "network" },
      { phase: "interrupted", received: 400, total: 1000 },
      { phase: "removed" },
    ]) {
      it(`starts the download again after ${state.phase}`, async () => {
        const { controller, host, setting } = setup({
          seed: { [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: state },
        });
        await controller.handle("resume");
        expect(host.startDownload).toHaveBeenCalledOnce();
        expect(setting().state).toMatchObject({ phase: "downloading" });
      });
    }

    it("does nothing while a download runs, when the model is ready, or when the setting is off", async () => {
      const running = setup({ downloading: true });
      await running.controller.enable();
      await running.controller.resume();
      expect(running.host.startDownload).toHaveBeenCalledOnce();

      const ready = setup({ seed: ON_READY, stored: true });
      await ready.controller.resume();
      expect(ready.host.startDownload).not.toHaveBeenCalled();

      const off = setup();
      await off.controller.resume();
      expect(off.host.startDownload).not.toHaveBeenCalled();
    });

    it("restarts a download its host dropped", async () => {
      const { controller, host, stopDownloading } = setup();
      await controller.enable();
      stopDownloading();
      await controller.resume();
      expect(host.startDownload).toHaveBeenCalledTimes(2);
    });
  });

  it("routes each command", async () => {
    const { controller, host } = setup();
    await controller.handle("status");
    await controller.handle("enable");
    await controller.handle("disable");
    expect(host.startDownload).toHaveBeenCalledOnce();
    expect(host.shutDown).toHaveBeenCalledOnce();
  });

  it("states a total of 0 when the manifest cannot be read, rather than fail to start", async () => {
    const area = memoryArea();
    const controller = new ModelController({
      area,
      host: { startDownload: vi.fn(async () => {}), cancelDownload: vi.fn(), downloading: vi.fn(), shutDown: vi.fn() },
      db: { complete: vi.fn(), erase: vi.fn() },
      manifest: async () => Promise.reject(new Error("missing")),
      offered: async () => true,
    });
    await controller.enable();
    expect(area.store[MODEL_STATE_KEY]).toEqual({ phase: "downloading", received: 0, total: 0 });
  });
});
