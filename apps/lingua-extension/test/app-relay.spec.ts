import { describe, expect, it } from "vitest";
import type { CardOp, StatusOp } from "@/analyzer/port.ts";
import {
  applyAppOps,
  appOpsSince,
  encodeAppOps,
  encodeSharedState,
  parseAppOps,
  parseSharedState,
  RELAY_VERSION,
  toSecond,
} from "@/state/app-relay.ts";
import { makeFakePort } from "./helpers.ts";

const card = (client_ts: number, lemma = "dog"): CardOp => ({
  client_id: lemma,
  language: "en",
  lemma,
  surface_form: lemma,
  source_sentence: "",
  source: "",
  gloss: "",
  fsrs_state: "{}",
  deleted: false,
  client_ts,
  device_id: "",
});
const status = (updated_at: number, lemma = "dog"): StatusOp => ({
  language: "en",
  lemma,
  status: "known",
  provenance: "srs",
  updated_at,
});

describe("the shared state (add-lingua-apple-review D3)", () => {
  it("reads back what it wrote", () => {
    const raw = encodeSharedState({ backup: "B", savedAt: 42, prefs: { a: 1 }, storeItems: { d: 2 } });
    expect(parseSharedState(raw)).toEqual({
      v: RELAY_VERSION,
      backup: "B",
      savedAt: 42,
      prefs: { a: 1 },
      storeItems: { d: 2 },
    });
  });

  it("reads nothing, another version, an empty backup or garbage as no state", () => {
    for (const raw of [
      undefined,
      null,
      3,
      "",
      "not json",
      "[]",
      JSON.stringify({ v: 2, backup: "B" }),
      JSON.stringify({ v: 1, backup: "" }),
    ]) {
      expect(parseSharedState(raw)).toBeNull();
    }
  });

  it("defaults a missing date and malformed maps", () => {
    expect(parseSharedState(JSON.stringify({ v: 1, backup: "B", prefs: [1], storeItems: null }))).toEqual({
      v: 1,
      backup: "B",
      savedAt: 0,
      prefs: {},
      storeItems: {},
    });
  });
});

describe("the app's changes", () => {
  it("reads back what it wrote, and refuses what is not ops", () => {
    const ops = { since: 1000, cardOps: [card(2000)], statusOps: [status(3000)] };
    expect(parseAppOps(encodeAppOps(ops))).toEqual({ v: 1, ...ops });
    expect(parseAppOps(JSON.stringify({ v: 1, cardOps: [] }))).toBeNull();
    expect(parseAppOps(JSON.stringify({ v: 9, cardOps: [], statusOps: [] }))).toBeNull();
    expect(parseAppOps("{")).toBeNull();
    expect(parseAppOps(JSON.stringify({ v: 1, cardOps: [], statusOps: [] }))?.since).toBe(0);
  });

  it("dates to the second, as the deck does", () => {
    expect(toSecond(12_345)).toBe(12_000);
  });

  it("exports only what changed from `since` on", async () => {
    const { port } = makeFakePort();
    port.exportCardOps = async () => [card(1000, "old"), card(5000, "new")];
    port.exportStatusOps = async () => [status(999, "old"), status(5000, "new")];
    const ops = await appOpsSince(port, 5000);
    expect(ops.since).toBe(5000);
    expect(ops.cardOps.map((o) => o.lemma)).toEqual(["new"]);
    expect(ops.statusOps.map((o) => o.lemma)).toEqual(["new"]);
  });

  it("applies statuses and cards as a pull does, and counts what changed", async () => {
    const { port } = makeFakePort();
    const applied: string[] = [];
    port.applyStatusChanges = async (c) => (applied.push("statuses"), c.length);
    port.applyCardOps = async (c) => (applied.push("cards"), c.length);
    expect(await applyAppOps(port, { v: 1, since: 0, cardOps: [card(1)], statusOps: [status(1), status(2)] })).toBe(3);
    expect(applied).toEqual(["statuses", "cards"]);
    applied.length = 0;
    expect(await applyAppOps(port, { v: 1, since: 0, cardOps: [], statusOps: [] })).toBe(0);
    expect(applied).toEqual([]);
  });
});
