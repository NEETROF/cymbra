import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it, vi } from "vitest";
import { readerPairs } from "@/analyzer/pairs.ts";
import { ModelController, type ModelHostAccess } from "@/translate/host/model-controller.ts";
import { type ModelCatalogue, type ModelManifest, parseCatalogue } from "@/translate/host/model-manifest.ts";
import { MODEL_STATE_KEY, type SettingArea, TRANSLATION_HOST_KEY } from "@/translate/setting.ts";

// « Traduction étendue » as the background runs it — the scenarios of
// specs/lingua-translation/spec.md (add-lingua-translation-delivery, generalise-lingua-translation-
// model-state, generalise-lingua-translation-routes-by-pair), driven through fakes: the device's
// storage, the engine's host, the models' database, the package's catalogue and the reader's pairs.

const EN_FR = "en-fr/base-memory/2.0";
const ES_EN = "es-en/base-memory/2.0";
const EN_ES = "en-es/base-memory/2.1";

const files = (seed: string, sizes: [number, number, number]) => ({
  model: { path: `${seed}/m.gz`, size: sizes[0], unpacked: sizes[0] + 200, sha256: seed[0]!.repeat(64) },
  lex: { path: `${seed}/l.gz`, size: sizes[1], unpacked: sizes[1] + 100, sha256: seed[1]!.repeat(64) },
  vocab: { path: `${seed}/v.gz`, size: sizes[2], unpacked: sizes[2] + 50, sha256: seed[2]!.repeat(64) },
});

/** en-fr goes straight to French; es-fr goes through English, sharing en-fr. Routes are keyed by pair (routes-by-pair D1). */
const CATALOGUE: ModelCatalogue = {
  base: "https://models.example/",
  models: {
    [EN_FR]: { from: "en", to: "fr", files: files("abc", [700, 200, 100]) },
    [ES_EN]: { from: "es", to: "en", files: files("def", [500, 300, 200]) },
  },
  routes: { "en-fr": [EN_FR], "es-fr": [ES_EN, EN_FR] },
};

/** The catalogue once en-es ships: its route listed (change 25), its pair a reader's (changes 22, 35). */
const WITH_EN_ES: ModelCatalogue = {
  ...CATALOGUE,
  models: { ...CATALOGUE.models, [EN_ES]: { from: "en", to: "es", files: files("ghi", [600, 100, 100]) } },
  routes: { ...CATALOGUE.routes, "en-es": [EN_ES] },
};

/** What en-fr costs, from the catalogue: its files as served, and decompressed. */
const COST = { download: 1000, stored: 1350 };
/** What en-fr and es-fr together cost: en-fr once, and es-en; es-fr's route pivots through English. */
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
    /** The reader's pairs: their accepted languages, each through the shipped pair of their native language. */
    pairs?: string[];
    catalogue?: () => Promise<ModelCatalogue>;
  } = {},
) {
  const area = memoryArea(opts.seed);
  let downloading = opts.downloading ?? false;
  const stored = new Set(opts.stored ?? []);
  let pairs = opts.pairs ?? ["en-fr"];
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
    pairs: async () => pairs,
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
    /** The reader's pairs changed: a language added or removed, or the native language changed. */
    study: (next: string[]) => (pairs = next),
    setting: () => ({ host: area.store[TRANSLATION_HOST_KEY], state: area.store[MODEL_STATE_KEY] }),
  };
}

const READY_EN = { phase: "ready", models: [EN_FR], pairs: ["en-fr"] };
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
    expect(await controller.ready("en-fr")).toBe(false);
  });

  it("states what the models cost, from the catalogue (generalise-lingua-translation-catalogue)", async () => {
    expect((await setup().controller.status()).cost).toEqual(COST);
    expect((await setup({ pairs: ["en-fr", "es-fr"] }).controller.status()).cost).toEqual(BOTH);
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

  it("is not offered while none of the reader's pairs has a route (model-state D3)", async () => {
    const { controller } = setup({ pairs: ["de-fr"] });
    expect(await controller.status()).toEqual({ offered: false, host: "none", state: { phase: "absent" } });
  });

  it("A pair without a route: en-es is never recorded ready, so the background's gate answers false", async () => {
    // A reader of Spanish on an English page, with no en-es in the catalogue: nothing is needed,
    // nothing is offered, and `ready` — what the background asks before the engine — says no.
    const { controller, host } = setup({ pairs: ["en-es"], seed: ON_READY, stored: [EN_FR] });
    expect(await controller.status()).toMatchObject({ offered: false, state: { phase: "ready", pairs: [] } });
    expect(await controller.ready("en-es")).toBe(false);
    expect(host.startDownload).not.toHaveBeenCalled();
  });

  describe("turning it on", () => {
    it("Every reader today: a reader of English alone, native French, ticks the setting — en-fr is downloaded, and nothing else", async () => {
      const { controller, host, setting, downloaded } = setup({ pairs: ["en-fr"] });
      const status = await controller.enable();
      expect(host.startDownload).toHaveBeenCalledWith([EN_FR]);
      expect(setting()).toEqual({ host: "local", state: { phase: "downloading", received: 0, total: 1000 } });
      expect(status).toMatchObject({ offered: true, host: "local", state: { phase: "downloading" } });
      await downloaded();
      expect(setting().state).toEqual(READY_EN);
      expect(host.startDownload).toHaveBeenCalledOnce();
    });

    it("downloads every model a reader of two pairs needs, a shared one once", async () => {
      const { controller, host, setting } = setup({ pairs: ["en-fr", "es-fr"] });
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
      expect(await controller.ready("en-fr")).toBe(true);
      expect(await controller.ready("es-fr")).toBe(false);
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
      expect(await controller.ready("en-fr")).toBe(false);
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

  describe("the models follow the reader's pairs (model-state D2, D3; routes-by-pair D5)", () => {
    it("A language added while the setting is on: its pair's model is missing — nothing is fetched, the cost is stated", async () => {
      const { controller, host, setting, study } = setup({ seed: ON_READY, stored: [EN_FR] });
      study(["en-fr", "es-fr"]);
      const status = await controller.status();
      expect(setting().state).toEqual({ phase: "missing", models: [EN_FR], pairs: ["en-fr"], total: 1000 });
      expect(status.cost).toEqual(BOTH);
      expect(host.startDownload).not.toHaveBeenCalled();
      // en-fr is still translated; es-fr waits for its model.
      expect(await controller.ready("en-fr")).toBe(true);
      expect(await controller.ready("es-fr")).toBe(false);
    });

    it("« Télécharger » downloads what is missing, then both pairs are ready", async () => {
      const { controller, host, setting, study, downloaded } = setup({ seed: ON_READY, stored: [EN_FR] });
      study(["en-fr", "es-fr"]);
      await controller.status();
      await controller.handle("resume");
      expect(host.startDownload).toHaveBeenCalledWith([EN_FR, ES_EN]); // the worker skips what is stored
      await downloaded();
      expect(setting().state).toEqual({ phase: "ready", models: [EN_FR, ES_EN], pairs: ["en-fr", "es-fr"] });
      expect(await controller.ready("es-fr")).toBe(true);
    });

    it("A language removed: the models only es-fr needed are deleted, and en-fr stays", async () => {
      const { controller, db, stored, setting, study } = setup({
        seed: {
          [TRANSLATION_HOST_KEY]: "local",
          [MODEL_STATE_KEY]: { phase: "ready", models: [EN_FR, ES_EN], pairs: ["en-fr", "es-fr"] },
        },
        stored: [EN_FR, ES_EN],
        pairs: ["en-fr", "es-fr"],
      });
      study(["en-fr"]);
      await controller.status();
      expect(db.prune).toHaveBeenLastCalledWith([expect.objectContaining({ version: EN_FR })]);
      expect([...stored]).toEqual([EN_FR]);
      expect(setting().state).toEqual(READY_EN);
    });

    it("keeps en-fr for a reader who keeps es-fr alone, since es-fr goes through it", async () => {
      const { controller, stored, study } = setup({
        seed: {
          [TRANSLATION_HOST_KEY]: "local",
          [MODEL_STATE_KEY]: { phase: "ready", models: [EN_FR, ES_EN], pairs: ["en-fr", "es-fr"] },
        },
        stored: [EN_FR, ES_EN],
      });
      study(["es-fr"]);
      await controller.status();
      expect([...stored].sort()).toEqual([EN_FR, ES_EN].sort());
    });

    it("The native language changes: a reader of English now native in Spanish, with en-es listed — en-es's models are needed, en-fr's deleted, nothing fetched", async () => {
      const { controller, host, stored, setting, study } = setup({
        seed: ON_READY,
        stored: [EN_FR],
        catalogue: async () => WITH_EN_ES,
      });
      study(["en-es"]); // the pairs follow the native language (the choice itself is change 20)
      const status = await controller.status();
      expect([...stored]).toEqual([]); // en-fr, which only en-fr's route needed
      expect(setting().state).toEqual({ phase: "missing", models: [], pairs: [], total: 800 });
      expect(status.cost).toEqual({ download: 800, stored: 1150 });
      expect(host.startDownload).not.toHaveBeenCalled();
      expect(await controller.ready("en-fr")).toBe(false);
      expect(await controller.ready("en-es")).toBe(false);
      // The reader asks: en-es's model comes, and en-es alone is ready.
      await controller.handle("resume");
      expect(host.startDownload).toHaveBeenCalledWith([EN_ES]);
    });

    it("A model stored before the update: a bare `ready` is read as en-fr's, the model is still complete, and English is still translated", async () => {
      // The release before recorded `ready` only once the English model was there.
      const { controller, setting, host } = setup({
        seed: { [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: { phase: "ready" } },
        stored: [EN_FR],
      });
      expect(await controller.ready("en-fr")).toBe(true); // before any reconcile: English still translated
      await controller.status();
      expect(setting().state).toEqual(READY_EN);
      expect(host.startDownload).not.toHaveBeenCalled();
    });

    it("A state recorded before pairs: `languages` en and es are read as en-fr and es-fr, and the next reconciliation records the pairs", async () => {
      const { controller, setting, host, db } = setup({
        seed: {
          [TRANSLATION_HOST_KEY]: "local",
          [MODEL_STATE_KEY]: { phase: "ready", models: [EN_FR, ES_EN], languages: ["en", "es"] },
        },
        stored: [EN_FR, ES_EN],
        pairs: ["en-fr", "es-fr"],
      });
      // Before any reconcile: English and Spanish pages are translated as before.
      expect(await controller.ready("en-fr")).toBe(true);
      expect(await controller.ready("es-fr")).toBe(true);
      await controller.status();
      expect(setting().state).toEqual({ phase: "ready", models: [EN_FR, ES_EN], pairs: ["en-fr", "es-fr"] });
      expect(setting().state).not.toHaveProperty("languages");
      expect(host.startDownload).not.toHaveBeenCalled();
      expect(db.erase).not.toHaveBeenCalled();
    });

    it("rewrites a state recorded without pairs once, then leaves it: `languages`, then a bare `ready`", async () => {
      const { controller, area, setting } = setup({
        seed: {
          [TRANSLATION_HOST_KEY]: "local",
          [MODEL_STATE_KEY]: { phase: "ready", models: [EN_FR], languages: ["en"] },
        },
        stored: [EN_FR],
      });
      const set = vi.spyOn(area, "set");
      await controller.status();
      expect(set).toHaveBeenCalledOnce();
      expect(setting().state).toEqual(READY_EN);
      await controller.status();
      expect(set).toHaveBeenCalledOnce(); // the pairs are stored now: nothing to rewrite
      const bare = setup({
        seed: { [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: { phase: "ready" } },
        stored: [EN_FR],
      });
      const bareSet = vi.spyOn(bare.area, "set");
      await bare.controller.status();
      await bare.controller.status();
      expect(bareSet).toHaveBeenCalledOnce();
      expect(bare.setting().state).toEqual(READY_EN);
    });

    it("does not rewrite a state the storage hands back with its keys in another order", async () => {
      // Chrome's storage returns an object's keys in its own order; what is compared is the state read,
      // not its text — or `ready` would be rewritten on every status.
      const { controller, area } = setup({
        seed: {
          [TRANSLATION_HOST_KEY]: "local",
          [MODEL_STATE_KEY]: { models: [EN_FR], pairs: ["en-fr"], phase: "ready" },
        },
        stored: [EN_FR],
      });
      const set = vi.spyOn(area, "set");
      await controller.status();
      await controller.status();
      expect(set).not.toHaveBeenCalled();
    });

    it("drops at the reconcile a language recorded before pairs that no shipped pair studies: `de` read as de-fr", async () => {
      const { controller, setting } = setup({
        seed: {
          [TRANSLATION_HOST_KEY]: "local",
          [MODEL_STATE_KEY]: { phase: "ready", models: [EN_FR], languages: ["en", "de"] },
        },
        stored: [EN_FR],
        pairs: ["en-fr"],
      });
      expect(await controller.ready("de-fr")).toBe(true); // as recorded, until reconciled
      await controller.status();
      expect(setting().state).toEqual(READY_EN);
      expect(await controller.ready("de-fr")).toBe(false);
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
      expect(await controller.ready("en-fr")).toBe(false);
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

describe("the committed catalogue: a reader of French downloads, keeps and loads as before (add-lingua-translation-matrix-models D2)", () => {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const committed = parseCatalogue(JSON.parse(readFileSync(join(root, "model-manifest.json"), "utf8")));
  /** The reader's pairs as the background forms them: the shipped pairs of their native language. */
  const of = (languages: string[], native: string, pairs?: readonly string[]) => readerPairs(languages, native, pairs);
  /** The shipped pairs before change 34 (enable-lingua-english-speakers): French-native only. */
  const FRENCH_NATIVE = ["en-fr", "es-fr"];
  /** The shipped pairs in change 34: es-en beside them, no pair glossed in Spanish yet. */
  const WITH_ES_EN = [...FRENCH_NATIVE, "es-en"];

  it("Every reader today: English and Spanish, native French — en-fr and es-en are downloaded, 51 993 524 bytes, and nothing of en-es", async () => {
    const { controller, host, setting, downloaded } = setup({
      pairs: of(["en", "es"], "fr"),
      catalogue: async () => committed,
    });
    const status = await controller.enable();
    expect(host.startDownload).toHaveBeenCalledWith([EN_FR, ES_EN]);
    expect(status.cost).toEqual({ download: 51_993_524, stored: 73_763_216, pivot: true });
    await downloaded();
    expect(setting().state).toEqual({ phase: "ready", models: [EN_FR, ES_EN], pairs: ["en-fr", "es-fr"] });
    // The routes it loads are en-fr's and es-fr's; es-en and en-es are no pair of this reader's.
    expect(await controller.ready("es-en")).toBe(false);
    expect(await controller.ready("en-es")).toBe(false);
  });

  it("Every reader today: English alone, native French — en-fr alone, 25 752 472 bytes", async () => {
    const { controller, host } = setup({ pairs: of(["en"], "fr"), catalogue: async () => committed });
    expect((await controller.status()).cost).toEqual({ download: 25_752_472, stored: 36_749_127 });
    await controller.enable();
    expect(host.startDownload).toHaveBeenCalledWith([EN_FR]);
  });

  it("A route of a pair not shipped: the es-en model is kept only as es-fr's first model — Spanish removed, it is deleted although es-en is routed", async () => {
    const { controller, stored, setting, study } = setup({
      seed: {
        [TRANSLATION_HOST_KEY]: "local",
        [MODEL_STATE_KEY]: { phase: "ready", models: [EN_FR, ES_EN], pairs: ["en-fr", "es-fr"] },
      },
      stored: [EN_FR, ES_EN],
      pairs: of(["en", "es"], "fr"),
      catalogue: async () => committed,
    });
    study(of(["en"], "fr"));
    await controller.status();
    expect([...stored]).toEqual([EN_FR]);
    expect(setting().state).toEqual(READY_EN);
  });

  it("A route of a pair not shipped: a reader native in Spanish or in English has no shipped pair — not offered, nothing downloaded", async () => {
    // The scenario's list: en-fr and es-fr, as before change 34; then change 34's, which shipped es-en and no
    // pair glossed in Spanish. Since change 35, es-en and en-es both ship (below).
    for (const pairs of [
      of(["en"], "es", FRENCH_NATIVE),
      of(["es"], "en", FRENCH_NATIVE),
      of(["en"], "es", WITH_ES_EN),
    ]) {
      expect(pairs).toEqual([]);
      const { controller, host } = setup({ pairs, catalogue: async () => committed });
      expect(await controller.status()).toEqual({ offered: false, host: "none", state: { phase: "absent" } });
      expect(host.startDownload).not.toHaveBeenCalled();
    }
  });

  it("A route of a pair studying French: fr-en and fr-es are routed, no reader's pairs need them — nothing of fr-en is downloaded (add-lingua-french-translation D2)", async () => {
    expect(Object.keys(committed.routes)).toEqual(expect.arrayContaining(["fr-en", "fr-es"]));
    // A reader of French whose native language is English or Spanish has no shipped pair until change 52.
    // The English-native reader of Spanish and French on the list before change 34: es-en alone since it (below);
    // the Spanish-native reader of English and French on change 34's: en-es alone since change 35 (below).
    for (const pairs of [
      of(["fr"], "en"),
      of(["fr"], "es"),
      of(["es", "fr"], "en", FRENCH_NATIVE),
      of(["en", "fr"], "es", WITH_ES_EN),
    ]) {
      expect(pairs).toEqual([]);
      const { controller, host } = setup({ pairs, catalogue: async () => committed });
      expect(await controller.status()).toEqual({ offered: false, host: "none", state: { phase: "absent" } });
      expect(host.startDownload).not.toHaveBeenCalled();
    }
    // A reader of French who also reads French pages downloads en-fr and es-en, as before.
    const { controller, host } = setup({ pairs: of(["en", "es", "fr"], "fr"), catalogue: async () => committed });
    expect((await controller.enable()).cost).toEqual({ download: 51_993_524, stored: 73_763_216, pivot: true });
    expect(host.startDownload).toHaveBeenCalledWith([EN_FR, ES_EN]);
    expect(await controller.ready("fr-en")).toBe(false);
    expect(await controller.ready("fr-es")).toBe(false);
  });

  it("An English-native reader of Spanish, es-en shipping (change 34): the es-en model alone, nothing of en-fr", async () => {
    // English is never an English-native reader's pair: es-en alone, whatever else they list.
    for (const languages of [["es"], ["en", "es"], ["es", "en"], ["es", "fr"]]) {
      const pairs = of(languages, "en");
      expect(pairs).toEqual(["es-en"]);
      const { controller, host, setting, downloaded } = setup({ pairs, catalogue: async () => committed });
      const status = await controller.enable();
      expect(host.startDownload).toHaveBeenCalledWith([ES_EN]);
      expect(status.cost).toEqual({ download: 26_241_052, stored: 37_014_089 });
      await downloaded();
      expect(setting().state).toEqual({ phase: "ready", models: [ES_EN], pairs: ["es-en"] });
      expect(await controller.ready("es-en")).toBe(true);
      expect(await controller.ready("en-fr")).toBe(false);
      expect(await controller.ready("fr-en")).toBe(false);
    }
  });

  it("A Spanish-native reader of English, en-es shipping (change 35): the en-es model alone, nothing of en-fr or es-en", async () => {
    // Spanish is never a Spanish-native reader's pair: en-es alone, whatever else they list.
    for (const languages of [["en"], ["en", "es"], ["es", "en"], ["en", "fr"]]) {
      const pairs = of(languages, "es");
      expect(pairs).toEqual(["en-es"]);
      const { controller, host, setting, downloaded } = setup({ pairs, catalogue: async () => committed });
      const status = await controller.enable();
      expect(host.startDownload).toHaveBeenCalledWith([EN_ES]);
      expect(status.cost).toEqual({ download: 25_373_354, stored: 36_576_277 });
      await downloaded();
      expect(setting().state).toEqual({ phase: "ready", models: [EN_ES], pairs: ["en-es"] });
      expect(await controller.ready("en-es")).toBe(true);
      expect(await controller.ready("es-en")).toBe(false);
      expect(await controller.ready("en-fr")).toBe(false);
      expect(await controller.ready("fr-es")).toBe(false);
    }
  });
});
