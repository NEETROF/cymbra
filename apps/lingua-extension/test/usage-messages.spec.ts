import { afterEach, describe, expect, it, vi } from "vitest";
import { USAGE_COUNTS_KEY, USAGE_COUNTS_OFF_KEY } from "@/usage/counts.ts";
import { dataCollectionOf, grantConsent, switchConsent, touchesDataCollection } from "@/usage/consent.ts";
import {
  isUsageMessage,
  reportSetUp,
  runtimeNoticeControls,
  runtimeUsageSwitch,
  type DataCollectionPermissions,
} from "@/usage/messages.ts";

// The counts' messages and the places each browser keeps the reader's choice
// (add-lingua-usage-counts D6, D7): a surface says what happened and the background decides; a
// surface whose background is unreachable never fails.

afterEach(() => vi.unstubAllGlobals());

describe("the surfaces' side", () => {
  it("recognises its own messages only", () => {
    expect(isUsageMessage({ type: "usage:status" })).toBe(true);
    expect(isUsageMessage({ type: "usage:switch", on: true })).toBe(true);
    expect(isUsageMessage({ type: "sync:request" })).toBe(false);
    expect(isUsageMessage(null)).toBe(false);
  });

  it("a level saved says « set up », and never fails without a background", async () => {
    const send = vi.fn(async () => undefined);
    await reportSetUp(send);
    expect(send).toHaveBeenCalledWith({ type: "usage:step", step: "set_up" });
    await expect(reportSetUp(async () => Promise.reject(new Error("no receiver")))).resolves.toBeUndefined();
    // No runtime at all (a spec, an orphaned page).
    await expect(reportSetUp()).resolves.toBeUndefined();
  });

  it("the sentence's controls ask, tell and answer the background", async () => {
    const sent: unknown[] = [];
    const send = async (m: unknown) => {
      sent.push(m);
      return (m as { type: string }).type === "usage:status" ? { notice: "ask" } : { ok: true };
    };
    const c = runtimeNoticeControls(send);
    expect(await c.status()).toBe("ask");
    await c.shown();
    await c.answer(false);
    expect(sent).toEqual([
      { type: "usage:status" },
      { type: "usage:notice-shown" },
      { type: "usage:consent", yes: false },
    ]);
    // Unreachable: nothing is due.
    expect(await runtimeNoticeControls(async () => undefined).status()).toBe("none");
  });

  it("the switch's controls read, move and forget through the background", async () => {
    const sent: unknown[] = [];
    const send = async (m: unknown) => {
      sent.push(m);
      const type = (m as { type: string }).type;
      if (type === "usage:switch-state") return { shown: true, on: true };
      if (type === "usage:switch") return { view: { shown: true, on: false } };
      return { ok: true };
    };
    const c = runtimeUsageSwitch(send, undefined);
    expect(await c.state()).toEqual({ shown: true, on: true });
    expect(await c.set(false)).toEqual({ view: { shown: true, on: false } });
    await c.forget();
    expect(sent).toEqual([
      { type: "usage:switch-state" },
      { type: "usage:switch", on: false },
      { type: "usage:forget" },
    ]);
    // Unreachable: no switch shown.
    const dead = runtimeUsageSwitch(async () => undefined, undefined);
    expect(await dead.state()).toEqual({ shown: false, on: false });
    expect(await dead.set(true)).toEqual({ view: { shown: false, on: false } });
  });

  it("Firefox, in an extension page: the consent is asked from the click itself, before the background", async () => {
    const order: string[] = [];
    const permissions: DataCollectionPermissions = {
      request: vi.fn(async (p) => {
        order.push(`request ${p.data_collection.join()}`);
        return true;
      }),
    };
    const send = async (m: unknown) => {
      order.push((m as { type: string }).type);
      return { view: { shown: true, on: true } };
    };
    const c = runtimeUsageSwitch(send, permissions);
    const pending = c.set(true);
    // Called synchronously, while the click's gesture holds.
    expect(permissions.request).toHaveBeenCalledTimes(1);
    expect(await pending).toEqual({ view: { shown: true, on: true } });
    expect(order).toEqual(["request technicalAndInteraction", "usage:switch"]);
    // Declined in Firefox's prompt: the switch stays off, the background is not asked.
    const declined = runtimeUsageSwitch(send, { request: async () => false });
    expect(await declined.set(true)).toEqual({ view: { shown: true, on: false } });
    // Turning it off asks nothing of the browser here: the background withdraws it.
    order.length = 0;
    await c.set(false);
    expect(order).toEqual(["usage:switch"]);
  });

  it("a refused prompt still lets the background say what is possible", async () => {
    const send = vi.fn(async () => ({ view: { shown: true, on: false }, fromToolbar: true }));
    const c = runtimeUsageSwitch(send, {
      request: async () => {
        throw new Error("not from a user input handler");
      },
    });
    expect(await c.set(true)).toEqual({ view: { shown: true, on: false }, fromToolbar: true });
  });

  it("the switch follows the counts' keys in chrome.storage.local", () => {
    const listeners: ((changes: Record<string, unknown>, area: string) => void)[] = [];
    vi.stubGlobal("chrome", {
      storage: {
        onChanged: {
          addListener: (l: (changes: Record<string, unknown>, area: string) => void) => listeners.push(l),
          removeListener: (l: (changes: Record<string, unknown>, area: string) => void) =>
            listeners.splice(listeners.indexOf(l), 1),
        },
      },
    });
    const onChange = vi.fn();
    const stop = runtimeUsageSwitch(async () => undefined, undefined).watch(onChange);
    listeners[0]({ [USAGE_COUNTS_KEY]: {} }, "local");
    listeners[0]({ [USAGE_COUNTS_OFF_KEY]: {} }, "local");
    listeners[0]({ other: {} }, "local");
    listeners[0]({ [USAGE_COUNTS_KEY]: {} }, "session");
    expect(onChange).toHaveBeenCalledTimes(2);
    if (stop) stop();
    expect(listeners).toHaveLength(0);
    vi.unstubAllGlobals();
    // A context with no storage hears nothing, and does not throw.
    vi.stubGlobal("chrome", undefined);
    expect(runtimeUsageSwitch(async () => undefined, undefined).watch(onChange)).toBeUndefined();
  });
});

describe("where each browser keeps the reader's choice", () => {
  it("Chromium and Safari: the switch's key, absent unanswered", async () => {
    const area = (store: Record<string, unknown>) => ({
      get: async () => store,
      set: async () => {},
    });
    expect(await switchConsent(area({}))).toEqual({ kind: "switch", off: undefined });
    expect(await switchConsent(area({ [USAGE_COUNTS_OFF_KEY]: true }))).toEqual({ kind: "switch", off: true });
    expect(await switchConsent(area({ [USAGE_COUNTS_OFF_KEY]: false }))).toEqual({ kind: "switch", off: false });
    expect(await switchConsent(area({ [USAGE_COUNTS_OFF_KEY]: "yes" }))).toEqual({ kind: "switch", off: undefined });
  });

  it("Firefox: its data-collection consent, which a Firefox before 140 does not have", async () => {
    expect(await grantConsent({ getAll: async () => ({ data_collection: ["technicalAndInteraction"] }) })).toEqual({
      kind: "grant",
      supported: true,
      granted: true,
    });
    expect(await grantConsent({ getAll: async () => ({ data_collection: [] }) })).toEqual({
      kind: "grant",
      supported: true,
      granted: false,
    });
    expect(await grantConsent({ getAll: async () => ({}) })).toEqual({
      kind: "grant",
      supported: false,
      granted: false,
    });
    expect(await grantConsent({ getAll: async () => Promise.reject(new Error("x")) })).toEqual({
      kind: "grant",
      supported: false,
      granted: false,
    });
  });

  it("Firefox: asked and withdrawn as `technicalAndInteraction`, and its changes recognised", async () => {
    const request = vi.fn(async () => true);
    const remove = vi.fn(async () => true);
    const d = dataCollectionOf({ getAll: async () => ({}), request, remove });
    await d.request();
    await d.remove();
    expect(request).toHaveBeenCalledWith({ data_collection: ["technicalAndInteraction"] });
    expect(remove).toHaveBeenCalledWith({ data_collection: ["technicalAndInteraction"] });
    expect(touchesDataCollection({ data_collection: ["technicalAndInteraction"] })).toBe(true);
    expect(touchesDataCollection({ origins: ["<all_urls>"] })).toBe(false);
    expect(touchesDataCollection(null)).toBe(false);
  });
});
