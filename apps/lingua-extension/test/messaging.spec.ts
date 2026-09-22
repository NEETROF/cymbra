import { describe, expect, it, vi } from "vitest";
import { MessagingLinguaPort } from "@/analyzer/messaging-port.ts";
import { handleRpc, isRpcRequest } from "@/analyzer/rpc-host.ts";
import { RPC_TYPE, type RpcRequest } from "@/analyzer/rpc.ts";
import { makeFakePort } from "./helpers.ts";

describe("MessagingLinguaPort", () => {
  it("forwards each call as an RPC and returns the transport result", async () => {
    const send = vi.fn(async (method: string) => {
      if (method === "analyse")
        return { analysable: true, tokens: [], counted: 1, known: 1, percent: 100, analyzer_version: "1.0.0" };
      if (method === "deckCount") return 3;
      if (method === "backup") return "BACKUP";
      if (method === "licences") return ["A", "B"];
      return undefined;
    });
    const port = new MessagingLinguaPort(send);

    expect((await port.analyse(["a", "b"])).percent).toBe(100);
    expect(send).toHaveBeenCalledWith("analyse", [["a", "b"]]);
    expect(await port.deckCount()).toBe(3);
    expect(await port.backup()).toBe("BACKUP");
    expect(await port.licences()).toEqual(["A", "B"]);

    await port.addCard({ lemma: "seldom", surface: "seldom", sentence: "s", url: "u", gloss: null, capturedAt: 1 });
    expect(send).toHaveBeenCalledWith("addCard", [
      { lemma: "seldom", surface: "seldom", sentence: "s", url: "u", gloss: null, capturedAt: 1 },
    ]);

    await port.reviewGrade("good", 42);
    expect(send).toHaveBeenCalledWith("reviewGrade", ["good", 42]);
  });

  it("forwards a selection to the engine's phrase gloss", async () => {
    const answer = {
      tokens: [{ surface: "gave", lemma: "give", class: "Known", gloss: "donner", function_word: false }],
    };
    const send = vi.fn(async (method: string) => (method === "phraseGloss" ? answer : undefined));
    const port = new MessagingLinguaPort(send);

    expect(await port.phraseGloss("gave up")).toBe(answer);
    expect(send).toHaveBeenCalledWith("phraseGloss", ["gave up"]);
  });

  it("forwards the sync methods", async () => {
    const send = vi.fn(async (method: string) =>
      method === "applyStatusChanges" || method === "applyCardOps" ? 2 : [],
    );
    const port = new MessagingLinguaPort(send);

    await port.setStatusAt("seldom", "known", 1700);
    expect(send).toHaveBeenCalledWith("setStatusAt", ["seldom", "known", 1700]);
    await port.exportStatusOps();
    expect(send).toHaveBeenCalledWith("exportStatusOps", []);
    expect(await port.applyStatusChanges([{ language: "en", lemma: "run", status: "known", updated_at: 5 }])).toBe(2);
    await port.exportCardOps();
    expect(send).toHaveBeenCalledWith("exportCardOps", []);
    expect(await port.applyCardOps([])).toBe(2);
  });
  // The host resolves a call by looking the method name up ON the port (`port[method]`),
  // so the wire name has to BE the method name. Nothing else checks that: a forwarder
  // sending "reviewMark" for reviewMarkKnown type-checks, and breaks only on the two
  // browsers that host the engine in the event page. So walk every method there is.
  it("forwards every method under its own name, with its arguments untouched", async () => {
    const forwarded: Array<[string, unknown[]]> = [];
    const send = vi.fn(async (method: string, args: unknown[]) => {
      forwarded.push([method, args]);
      return `answer:${method}`;
    });
    const port = new MessagingLinguaPort(send) as unknown as Record<string, (...args: unknown[]) => Promise<unknown>>;

    const methods = Object.getOwnPropertyNames(MessagingLinguaPort.prototype).filter(
      (m) => m !== "constructor" && m !== "rpc",
    );
    expect(methods.length).toBeGreaterThan(30); // the seam is wide; this guards the filter

    for (const name of methods) {
      forwarded.length = 0;
      // Sentinels, not realistic values: what is under test is that they arrive in order.
      const args = Array.from({ length: port[name]!.length }, (_, i) => `${name}#${i}`);
      const answer = await port[name]!(...args);
      expect([name, forwarded]).toEqual([name, [[name, args]]]);
      expect(answer).toBe(`answer:${name}`);
    }
  });
});

describe("rpc host", () => {
  const req = (method: string, args: unknown[] = []): RpcRequest => ({ type: RPC_TYPE, method, args });

  it("recognises well-formed RPC requests only", () => {
    expect(isRpcRequest(req("analyse", [[]]))).toBe(true);
    expect(isRpcRequest({ type: "stats", pct: 1 })).toBe(false);
    expect(isRpcRequest(null)).toBe(false);
    expect(isRpcRequest({ type: RPC_TYPE, method: "x" })).toBe(false); // no args array
  });

  it("hydrates once, then dispatches to the engine port", async () => {
    const { port, calls } = makeFakePort();
    const ensure = vi.fn(async () => {});
    const res1 = await handleRpc(port, ensure, req("setCalibration", [2500]));
    const res2 = await handleRpc(port, ensure, req("calibration", []));
    expect(res1).toEqual({ ok: true, result: undefined });
    expect(res2).toEqual({ ok: true, result: 2500 });
    expect(calls.setCalibration).toEqual([2500]);
    expect(ensure).toHaveBeenCalledTimes(2); // ensure() is idempotent; caller memoises
  });

  it("returns an error result for an unknown method", async () => {
    const { port } = makeFakePort();
    const res = await handleRpc(port, async () => {}, req("nope", []));
    expect(res).toEqual({ ok: false, error: "unknown method: nope" });
  });

  it("captures a thrown error as a failed result", async () => {
    const { port } = makeFakePort();
    const boom = async () => {
      throw new Error("kaboom");
    };
    const res = await handleRpc(port, boom, req("calibration", []));
    expect(res).toEqual({ ok: false, error: "kaboom" });
  });
});
