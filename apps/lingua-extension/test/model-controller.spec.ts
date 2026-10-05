import { describe, expect, it, vi } from "vitest";
import { ModelController, type ModelHostAccess } from "@/translate/host/model-controller.ts";
import type { ModelCatalogue, ModelManifest } from "@/translate/host/model-manifest.ts";
import { MODEL_STATE_KEY, type SettingArea, TRANSLATION_HOST_KEY } from "@/translate/setting.ts";

// « Traduction étendue » as the background runs it — the scenarios of
// specs/lingua-translation/spec.md (add-lingua-translation-delivery, generalise-lingua-translation-
// model-state), driven through fakes: the device's storage, the engine's host, the models' database,
// the package's catalogue and the reader's languages.

const EN_FR = "en-fr/base-memory/2.0";
const ES_EN = "es-en/base-memory/2.0";

const files = (seed: string, sizes: [number, number, number]) => ({
  model: { path: `${seed}/m.gz`, size: sizes[0], unpacked: sizes[0] + 200, sha256: seed[0]!.repeat(64) },
  lex: { path: `${seed}/l.gz`, size: sizes[1], unpacked: sizes[1] + 100, sha256: seed[1]!.repeat(64) },
  vocab: { path: `${seed}/v.gz`, size: sizes[2], unpacked: sizes[2] + 50, sha256: seed[2]!.repeat(64) },
});

/** English goes straight to French; Spanish goes through English, sharing en-fr. */
const CATALOGUE: ModelCatalogue = {
  base: "https://models.example/",
  models: {
    [EN_FR]: { from: "en", to: "fr", files: files("abc", [700, 200, 100]) },
    [ES_EN]: { from: "es", to: "en", files: files("def", [500, 300, 200]) },
  },
  routes: { en: [EN_FR], es: [ES_EN, EN_FR] },
};

/** What en-fr costs, from the catalogue: its files as served, and decompressed. */
const COST = { download: 1000, stored: 1350 };
/** What English and Spanish together cost: en-fr once, and es-en; Spanish's route pivots through English. */
const BOTH = { download: 2000, stored: 2700, pivot: true };

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
  opts: {
    seed?: Record<string, unknown>;
    /** The models complete on the device. */
    stored?: string[];
    downloading?: boolean;
    languages?: string[];
    catalogue?: () => Promise<ModelCatalogue>;
  } = {},
) {
  const area = memoryArea(opts.seed);
  let downloading = opts.downloading ?? false;
  const stored = new Set(opts.stored ?? []);
  let languages = opts.languages ?? ["en"];
  const host = {
    startDownload: vi.fn<(models: string[]) => Promise<void>>(async () => void (downloading = true)),
    cancelDownload: vi.fn(async () => void (downloading = false)),
    downloading: vi.fn(async () => downloading),
    shutDown: vi.fn(async () => {}),
  } satisfies ModelHostAccess;
  const db = {
    complete: vi.fn(async (model: ModelManifest) => stored.has(model.version)),
    prune: vi.fn(async (keep: ModelManifest[]) => {
      for (const id of [...stored]) if (!keep.some((m) => m.version === id)) stored.delete(id);
    }),
    erase: vi.fn(async () => void stored.clear()),
  };
  const log = vi.fn();
  const controller = new ModelController({
    area,
    host,
    db,
    catalogue: opts.catalogue ?? (async () => CATALOGUE),
    languages: async () => languages,
    log,
  });
  return {
    controller,
    area,
    host,
    db,
    log,
    stored,
    /** The download finished: every model asked for is now complete. */
    downloaded: async () => {
      for (const id of host.startDownload.mock.calls.at(-1)?.[0] ?? []) stored.add(id);
      downloading = false;
      await controller.onEvent({ kind: "done" });
    },
    stopDownloading: () => (downloading = false),
    study: (next: string[]) => (languages = next),
    setting: () => ({ host: area.store[TRANSLATION_HOST_KEY], state: area.store[MODEL_STATE_KEY] }),
  };
}

const READY_EN = { phase: "ready", models: [EN_FR], languages: ["en"] };
const ON_READY = { [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: READY_EN };

describe("ModelController", () => {
  it("is off on a fresh install, and says nothing is downloaded", async () => {
    const { controller, host } = setup();
    await expect(controller.status()).resolves.toEqual({
      offered: true,
      host: "none",
      state: { phase: "absent" },
      cost: COST,
    });
    expect(host.startDownload).not.toHaveBeenCalled();
    expect(await controller.ready("en")).toBe(false);
  });

  it("states what the models cost, from the catalogue (generalise-lingua-translation-catalogue)", async () => {
    expect((await setup().controller.status()).cost).toEqual(COST);
    expect((await setup({ languages: ["en", "es"] }).controller.status()).cost).toEqual(BOTH);
  });

  it("states no cost when the catalogue cannot be read, and still answers", async () => {
    const { controller, log } = setup({
      catalogue: async () => {
        throw new Error("model-manifest.json: 404");
      },
    });
    const status = await controller.status();
    expect(status).toEqual({ offered: true, host: "none", state: { phase: "absent" } });
    expect(log).toHaveBeenCalled();
  });

  it("is not offered while none of the reader's languages has a route (model-state D3)", async () => {
    const { controller } = setup({ languages: ["de"] });
    expect(await controller.status()).toEqual({ offered: false, host: "none", state: { phase: "absent" } });
  });

  describe("turning it on", () => {
    it("records the choice and starts downloading what the reader's languages need, stating its size", async () => {
      const { controller, host, setting } = setup();
      const status = await controller.enable();
      expect(host.startDownload).toHaveBeenCalledWith([EN_FR]);
      expect(setting()).toEqual({ host: "local", state: { phase: "downloading", received: 0, total: 1000 } });
      expect(status).toMatchObject({ offered: true, host: "local", state: { phase: "downloading" } });
    });

    it("downloads every model a reader of two languages needs, a shared one once", async () => {
      const { controller, host, setting } = setup({ languages: ["en", "es"] });
      await controller.enable();
      expect(host.startDownload).toHaveBeenCalledWith([EN_FR, ES_EN]);
      expect(setting().state).toEqual({ phase: "downloading", received: 0, total: 2000 });
    });

    it("follows the download's progress, then its completion", async () => {
      const { controller, setting, downloaded } = setup();
      await controller.enable();
      await controller.onEvent({ kind: "progress", received: 400, total: 1000 });
      expect(setting().state).toEqual({ phase: "downloading", received: 400, total: 1000 });
      await downloaded();
      expect(setting().state).toEqual(READY_EN);
      expect(await controller.ready("en")).toBe(true);
      expect(await controller.ready("es")).toBe(false);
    });

    it("never loads the engine: the first translation asked does (D6)", async () => {
      const { controller, host, downloaded } = setup();
      await controller.enable();
      await downloaded();
      expect(host.shutDown).not.toHaveBeenCalled();
      // The controller has no way to load it at all — only the download was ever asked of the host.
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
      expect(await controller.ready("en")).toBe(false);
    });

    it("records a download that could not even start as failed", async () => {
      const { controller, host, setting } = setup();
      host.startDownload.mockRejectedValueOnce(new Error("offscreen refused"));
      await controller.enable();
      expect(setting().state).toEqual({ phase: "failed", reason: "unknown" });
    });

    it("is offered in every state it answers — Firefox for Android included (android D1)", async () => {
      const { controller } = setup();
      const answers = [
        await controller.status(),
        await controller.enable(),
        await controller.resume(),
        await controller.disable(),
      ];
      expect(answers.map((a) => a.offered)).toEqual([true, true, true, true]);
    });

    it("does not start a second download when it is already on", async () => {
      const { controller, host } = setup();
      await controller.enable();
      await controller.enable();
      expect(host.startDownload).toHaveBeenCalledOnce();
    });

    it("states a total of 0 when the catalogue cannot be read, rather than fail to start", async () => {
      const { controller, setting } = setup({ catalogue: async () => Promise.reject(new Error("missing")) });
      await controller.enable();
      expect(setting().state).toEqual({ phase: "downloading", received: 0, total: 0 });
    });
  });

  describe("turning it off", () => {
    it("stops the engine and the download and deletes every model", async () => {
      const { controller, host, db, setting } = setup({ seed: ON_READY, stored: [EN_FR] });
      const status = await controller.disable();
      expect(host.cancelDownload).toHaveBeenCalled();
      expect(host.shutDown).toHaveBeenCalled();
      expect(db.erase).toHaveBeenCalled();
      expect(setting()).toEqual({ host: "none", state: { phase: "absent" } });
      expect(status).toEqual({ offered: true, host: "none", state: { phase: "absent" }, cost: COST });
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
      const { controller, host } = setup({ seed: ON_READY, stored: [EN_FR] });
      await controller.disable();
      await controller.enable();
      expect(host.startDownload).toHaveBeenCalledOnce();
    });
  });

  describe("the models follow the reader's languages (model-state D2, D3)", () => {
    it("a language added while the setting is on is missing: nothing is fetched, the cost is stated", async () => {
      const { controller, host, setting, study } = setup({ seed: ON_READY, stored: [EN_FR] });
      study(["en", "es"]);
      const status = await controller.status();
      expect(setting().state).toEqual({ phase: "missing", models: [EN_FR], languages: ["en"], total: 1000 });
      expect(status.cost).toEqual(BOTH);
      expect(host.startDownload).not.toHaveBeenCalled();
      // English is still translated; Spanish waits for its model.
      expect(await controller.ready("en")).toBe(true);
      expect(await controller.ready("es")).toBe(false);
    });

    it("« Télécharger » downloads what is missing, then both are ready", async () => {
      const { controller, host, setting, study, downloaded } = setup({ seed: ON_READY, stored: [EN_FR] });
      study(["en", "es"]);
      await controller.status();
      await controller.handle("resume");
      expect(host.startDownload).toHaveBeenCalledWith([EN_FR, ES_EN]); // the worker skips what is stored
      await downloaded();
      expect(setting().state).toEqual({ phase: "ready", models: [EN_FR, ES_EN], languages: ["en", "es"] });
      expect(await controller.ready("es")).toBe(true);
    });

    it("a language removed deletes the models only it needed, and keeps the shared one", async () => {
      const { controller, db, stored, setting, study } = setup({
        seed: {
          [TRANSLATION_HOST_KEY]: "local",
          [MODEL_STATE_KEY]: { phase: "ready", models: [EN_FR, ES_EN], languages: ["en", "es"] },
        },
        stored: [EN_FR, ES_EN],
        languages: ["en", "es"],
      });
      study(["en"]);
      await controller.status();
      expect(db.prune).toHaveBeenLastCalledWith([expect.objectContaining({ version: EN_FR })]);
      expect([...stored]).toEqual([EN_FR]);
      expect(setting().state).toEqual(READY_EN);
    });

    it("keeps en-fr for a reader who keeps Spanish alone, since Spanish goes through it", async () => {
      const { controller, stored, study } = setup({
        seed: {
          [TRANSLATION_HOST_KEY]: "local",
          [MODEL_STATE_KEY]: { phase: "ready", models: [EN_FR, ES_EN], languages: ["en", "es"] },
        },
        stored: [EN_FR, ES_EN],
      });
      study(["es"]);
      await controller.status();
      expect([...stored].sort()).toEqual([EN_FR, ES_EN].sort());
    });

    it("reads a `ready` written before the models were named as English's, and names them", async () => {
      // The release before recorded `ready` only once the English model was there.
      const { controller, setting } = setup({
        seed: { [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: { phase: "ready" } },
        stored: [EN_FR],
      });
      expect(await controller.ready("en")).toBe(true); // before any reconcile: English still translated
      await controller.status();
      expect(setting().state).toEqual(READY_EN);
    });

    it("reads a missing model under such a `ready` as removed by the browser, not as a new language", async () => {
      const { controller } = setup({
        seed: { [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: { phase: "ready" } },
        stored: [],
      });
      await expect(controller.status()).resolves.toMatchObject({ state: { phase: "removed" } });
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

    it("a background that starts recovers a dropped download without reading the catalogue", async () => {
      const catalogue = vi.fn(async () => CATALOGUE);
      const { controller, setting } = setup({
        seed: { [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: { phase: "downloading", received: 4, total: 9 } },
        catalogue,
      });
      await controller.recover();
      expect(setting().state).toEqual({ phase: "interrupted", received: 4, total: 9 });
      expect(catalogue).not.toHaveBeenCalled();
    });

    it("recovering leaves a running download, a ready model and an off setting alone", async () => {
      const running = setup({
        seed: { [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: { phase: "downloading", received: 4, total: 9 } },
        downloading: true,
      });
      await running.controller.recover();
      expect(running.setting().state).toMatchObject({ phase: "downloading" });
      const ready = setup({ seed: ON_READY, stored: [EN_FR] });
      await ready.controller.recover();
      expect(ready.setting().state).toEqual(READY_EN);
      const off = setup();
      await off.controller.recover();
      expect(off.setting()).toEqual({ host: undefined, state: undefined });
    });

    it("a download still running stays downloading", async () => {
      const { controller } = setup();
      await controller.enable();
      await expect(controller.status()).resolves.toMatchObject({ state: { phase: "downloading" } });
    });

    it("a model the browser removed is reported removed, and nothing is fetched", async () => {
      const { controller, host, setting } = setup({ seed: ON_READY, stored: [] });
      await expect(controller.status()).resolves.toEqual({
        offered: true,
        host: "local",
        state: { phase: "removed" },
        cost: COST,
      });
      expect(setting().state).toEqual({ phase: "removed" });
      expect(host.startDownload).not.toHaveBeenCalled();
      expect(await controller.ready("en")).toBe(false);
    });

    it("a model that cannot be read counts as removed", async () => {
      const { controller, db, log } = setup({ seed: ON_READY });
      db.complete.mockRejectedValueOnce(new Error("IndexedDB unavailable"));
      await expect(controller.status()).resolves.toMatchObject({ state: { phase: "removed" } });
      expect(log).toHaveBeenCalled();
    });

    it("a model still stored stays ready", async () => {
      const { controller } = setup({ seed: ON_READY, stored: [EN_FR] });
      await expect(controller.status()).resolves.toMatchObject({ state: { phase: "ready" } });
    });

    it("on, with its state lost, is removed — never downloaded behind the reader's back", async () => {
      const { controller, host } = setup({ seed: { [TRANSLATION_HOST_KEY]: "local" } });
      await expect(controller.status()).resolves.toMatchObject({ state: { phase: "removed" } });
      expect(host.startDownload).not.toHaveBeenCalled();
    });

    it("off, with a leftover state, is tidied to absent", async () => {
      const { controller, setting } = setup({ seed: { [MODEL_STATE_KEY]: READY_EN } });
      await controller.status();
      expect(setting().state).toEqual({ phase: "absent" });
    });

    it("a failure stays failed until the reader asks", async () => {
      const { controller } = setup({
        seed: { [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: { phase: "failed", reason: "network" } },
      });
      await expect(controller.status()).resolves.toMatchObject({ state: { phase: "failed", reason: "network" } });
    });

    it("changes and deletes nothing when the catalogue cannot be read", async () => {
      const { controller, db, setting } = setup({
        seed: ON_READY,
        stored: [EN_FR],
        catalogue: async () => Promise.reject(new Error("missing")),
      });
      await controller.status();
      expect(setting().state).toEqual(READY_EN);
      expect(db.prune).not.toHaveBeenCalled();
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

    it("does nothing while a download runs, when every model is ready, or when the setting is off", async () => {
      const running = setup({ downloading: true });
      await running.controller.enable();
      await running.controller.resume();
      expect(running.host.startDownload).toHaveBeenCalledOnce();

      const ready = setup({ seed: ON_READY, stored: [EN_FR] });
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
});
