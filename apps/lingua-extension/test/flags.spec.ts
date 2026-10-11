import { describe, expect, it, vi } from "vitest";
import {
  FLAG_REFRESH_MS,
  FLAGS_KEY,
  FlagStore,
  accountOfToken,
  flagOn,
  toValues,
  type WireFlags,
} from "@/flags/store.ts";
import { isFlagsMessage, readFlag, readFlags } from "@/flags/messages.ts";

// change: add-lingua-feature-betas — the extension reads its effective flags.

function memoryArea(items: Record<string, unknown> = {}) {
  const data: Record<string, unknown> = { ...items };
  return {
    data,
    get: vi.fn(async (key: string | string[] | null) => (typeof key === "string" ? { [key]: data[key] } : { ...data })),
    set: vi.fn(async (patch: Record<string, unknown>) => {
      Object.assign(data, patch);
    }),
  };
}

const on = (key: string): WireFlags["flags"][number] => ({ key, value: { kind: { case: "boolValue", value: true } } });

function setup(opts: { account?: string | null; fetch?: (v: string) => Promise<WireFlags>; now?: () => number } = {}) {
  const area = memoryArea();
  let account: string | null = opts.account === undefined ? "acc-1" : opts.account;
  const fetch = vi.fn(opts.fetch ?? (async () => ({ version: "v1", unchanged: false, flags: [on("x.enabled")] })));
  const store = new FlagStore({ fetch, area, account: () => account, now: opts.now });
  return { area, fetch, store, setAccount: (a: string | null) => (account = a) };
}

describe("flag values", () => {
  it("reads every wire kind, and only a true boolean is on", () => {
    const values = toValues([
      on("a"),
      { key: "b", value: { kind: { case: "boolValue", value: false } } },
      { key: "c", value: { kind: { case: "intValue", value: 100n } } },
      { key: "d", value: { kind: { case: "numberValue", value: 0.5 } } },
      { key: "e", value: { kind: { case: "stringValue", value: "s" } } },
      { key: "f", value: { kind: { case: "jsonValue", value: "{}" } } },
      { key: "g" },
    ]);
    expect(values).toEqual({ a: true, b: false, c: 100, d: 0.5, e: "s", f: "{}" });
    expect(flagOn(values, "a")).toBe(true);
    expect(flagOn(values, "b")).toBe(false);
    expect(flagOn(values, "c")).toBe(false);
    expect(flagOn(values, "missing")).toBe(false);
  });

  it("reads the account from the access token's subject", () => {
    const tok = (p: object) => `h.${btoa(JSON.stringify(p))}.s`;
    expect(accountOfToken(tok({ sub: "acc-1" }))).toBe("acc-1");
    expect(accountOfToken(tok({}))).toBeNull();
    expect(accountOfToken("garbage")).toBeNull();
    expect(accountOfToken(null)).toBeNull();
  });
});

describe("FlagStore", () => {
  it("fetches, persists the snapshot for the account, and reads it back", async () => {
    const { area, fetch, store } = setup();
    await store.refresh({ force: true });
    expect(fetch).toHaveBeenCalledWith("");
    expect(await store.read()).toEqual({ "x.enabled": true });
    expect((area.data[FLAGS_KEY] as { account: string }).account).toBe("acc-1");
  });

  it("is throttled outside sign-in, and sends its known version once the throttle has passed", async () => {
    let t = 0;
    const { fetch, store } = setup({ now: () => t });
    await store.refresh({ force: true });
    await store.refresh();
    expect(fetch).toHaveBeenCalledTimes(1);
    t = FLAG_REFRESH_MS + 1;
    await store.refresh();
    expect(fetch).toHaveBeenCalledTimes(2);
    expect(fetch).toHaveBeenLastCalledWith("v1");
  });

  it("keeps its values when the server says unchanged", async () => {
    const answers: WireFlags[] = [
      { version: "v1", unchanged: false, flags: [on("x.enabled")] },
      { version: "v1", unchanged: true, flags: [] },
    ];
    const { store } = setup({ fetch: async () => answers.shift()! });
    await store.refresh({ force: true });
    await store.refresh({ force: true });
    expect(await store.read()).toEqual({ "x.enabled": true });
  });

  it("keeps the last-good snapshot when a fetch fails", async () => {
    let fail = false;
    const { store } = setup({
      fetch: async () => {
        if (fail) throw new Error("offline");
        return { version: "v1", unchanged: false, flags: [on("x.enabled")] };
      },
    });
    await store.refresh({ force: true });
    fail = true;
    vi.spyOn(console, "warn").mockImplementation(() => {});
    await store.refresh({ force: true });
    expect(await store.read()).toEqual({ "x.enabled": true });
  });

  it("never reads one account's snapshot for another, and refetches from scratch for it", async () => {
    const { fetch, setAccount, store } = setup();
    await store.refresh({ force: true });
    setAccount("acc-2");
    expect(await store.read()).toEqual({});
    await store.refresh();
    expect(fetch).toHaveBeenLastCalledWith("");
  });

  it("reads every flag off when signed out, and asks the server nothing", async () => {
    const { fetch, setAccount, store } = setup();
    await store.refresh({ force: true });
    setAccount(null);
    expect(await store.read()).toEqual({});
    await store.refresh({ force: true });
    expect(fetch).toHaveBeenCalledTimes(1);
  });

  it("clears on sign-out", async () => {
    const { area, store } = setup();
    await store.refresh({ force: true });
    await store.clear();
    expect(area.data[FLAGS_KEY]).toBeNull();
  });

  it("drops an answer that lands after the account changed", async () => {
    let release: (w: WireFlags) => void = () => {};
    const { area, fetch, setAccount, store } = setup({ fetch: () => new Promise<WireFlags>((r) => (release = r)) });
    const pending = store.refresh({ force: true });
    await vi.waitFor(() => expect(fetch).toHaveBeenCalled());
    setAccount("acc-2");
    release({ version: "v1", unchanged: false, flags: [on("x.enabled")] });
    await pending;
    expect(area.data[FLAGS_KEY]).toBeUndefined();
  });

  it("runs one fetch at a time", async () => {
    const { fetch, store } = setup();
    await Promise.all([store.refresh({ force: true }), store.refresh({ force: true })]);
    expect(fetch).toHaveBeenCalledTimes(1);
  });
});

describe("flags messages", () => {
  it("recognises its message and reads through the background, off by default", async () => {
    expect(isFlagsMessage({ type: "flags:get" })).toBe(true);
    expect(isFlagsMessage({ type: "sync:request" })).toBe(false);
    expect(await readFlag("x.enabled", async () => ({ values: { "x.enabled": true } }))).toBe(true);
    expect(await readFlag("x.enabled", async () => undefined)).toBe(false);
    expect(
      await readFlags(async () => {
        throw new Error("no background");
      }),
    ).toEqual({});
  });
});
