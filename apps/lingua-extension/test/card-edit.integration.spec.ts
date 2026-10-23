import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { WasmAnalyzerPort, type WasmModule } from "@/analyzer/engine.ts";
import type { CardOp } from "@/analyzer/port.ts";
import { applyAppOps, appOpsSince } from "@/state/app-relay.ts";
import { packagedPart } from "./real-wasm.ts";

// A card edited in the Cymbra Lingua app reaches the extension (add-lingua-card-edit task 5.1), on
// the real engine — the wasm-pack build `yarn gen:wasm` makes, over the committed test pack's base
// and layer (test/real-wasm.ts): the app's session edits the card, `appOpsSince` hands the extension
// what changed (the renamed card, marked, the old one removed, the statuses it moved), and
// `applyAppOps` applies it as a pull is applied.

const here = dirname(fileURLToPath(import.meta.url));
const WASM = readFileSync(join(here, "../src/wasm/pkg/lingua_wasm_bg.wasm"));

const load = async (): Promise<WasmModule> => {
  const glue = await import("../src/wasm/pkg/lingua_wasm.js");
  return {
    ...(glue as unknown as WasmModule),
    default: async () => glue.initSync({ module: WASM }),
  };
};

const engine = (): WasmAnalyzerPort => new WasmAnalyzerPort(load, ["en-fr"]);

const OLD = "colloquially referred to as";
const NEW = "referred to as";

beforeEach(() => {
  vi.stubGlobal("chrome", { runtime: { getURL: (path: string) => path } });
  vi.stubGlobal(
    "fetch",
    vi.fn(async (url: string) => ({ arrayBuffer: async () => packagedPart(url) })),
  );
});

afterEach(() => vi.unstubAllGlobals());

/** The extension holding a captured card, and the app reviewing a copy of its state. */
async function capturedThenCopied() {
  const extension = engine();
  await extension.for("en").addCard({
    lemma: OLD,
    surface: OLD,
    sentence: "The bed is colloquially referred to as a tester.",
    url: "https://example.com",
    gloss: null,
    capturedAt: 100,
  });
  const app = engine();
  await app.restore(await extension.backup());
  expect(await app.startReview(200)).toBe(1);
  await app.reviewReveal();
  return { extension, app };
}

const byLemma = (ops: CardOp[]) => Object.fromEntries(ops.map((op) => [op.lemma, op]));

describe("a card edited in the app (add-lingua-card-edit)", () => {
  it("A card corrected in the app: the extension holds the new card, marked, and no longer the old one", async () => {
    const { extension, app } = await capturedThenCopied();
    expect(await app.reviewEdit(NEW, "dénommé", 300)).toEqual({ ok: true, changed: true });

    const ops = await appOpsSince(app, 200_000);
    expect(ops.cardOps.map((op) => [op.lemma, op.deleted, op.gloss_by_reader])).toEqual([
      [NEW, false, true],
      [OLD, true, undefined],
    ]);
    expect(ops.statusOps.map((op) => [op.lemma, op.status])).toEqual([
      [OLD, "cleared"],
      [NEW, "learning"],
    ]);
    expect(await applyAppOps(extension, ops)).toBe(4);

    const cards = byLemma(await extension.exportCardOps());
    expect(cards[NEW]).toMatchObject({ gloss: "dénommé", gloss_by_reader: true, deleted: false });
    expect(cards[OLD]).toMatchObject({ deleted: true, client_ts: 300_000 });
    expect(await extension.deckCount()).toBe(1);
    // Applied twice — the relay read again —, nothing changes.
    expect(await applyAppOps(extension, ops)).toBe(0);
  });

  it("An extension card changed after the app's edit survives it", async () => {
    const { extension, app } = await capturedThenCopied();
    await app.reviewEdit(NEW, "", 300);
    // Captured again in Safari after the edit, before the relay is read.
    await extension.for("en").addCard({
      lemma: OLD,
      surface: OLD,
      sentence: "Again.",
      url: "",
      gloss: null,
      capturedAt: 400,
    });
    await applyAppOps(extension, await appOpsSince(app, 200_000));
    const cards = byLemma(await extension.exportCardOps());
    expect(cards[OLD]).toMatchObject({ deleted: false, source_sentence: "Again." });
    expect(cards[NEW]).toMatchObject({ deleted: false });
  });

  it("refuses an empty form and changes nothing", async () => {
    const { app } = await capturedThenCopied();
    const before = await app.backup();
    expect(await app.reviewEdit(" , ", "", 300)).toEqual({ ok: false, error: "empty" });
    expect(await app.backup()).toBe(before);
  });
});
