import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Clients } from "@/net/transport.ts";

// The singleton holds module state, so each test starts from a module that has never been
// initialised — otherwise "used before init" could only ever be the first test in the file.
let api: typeof import("@/net/api.ts");

beforeEach(async () => {
  vi.resetModules();
  api = await import("@/net/api.ts");
});

describe("api()", () => {
  it("refuses to hand out clients before anything wired them", () => {
    // A caller that reaches the network before the background has a token getter is a bug
    // to fix at the call site, not a silently-unauthenticated request.
    expect(() => api.api()).toThrow(/before initApi/);
  });

  it("hands back the clients initApi built", () => {
    api.initApi(() => "tok");
    const clients = api.api();
    expect(Object.keys(clients).sort()).toEqual(["auth", "data", "deck", "knownWords", "stats", "user"]);
    expect(api.api()).toBe(clients); // the same instance: it is a singleton, not a factory
  });

  it("lets a test swap the whole set of clients in", () => {
    const fake = { auth: "fake" } as unknown as Clients;
    api.setClientsForTest(fake);
    expect(api.api()).toBe(fake);
  });
});
