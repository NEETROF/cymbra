import { describe, expect, it, vi } from "vitest";
import { Code, ConnectError, type Client } from "@connectrpc/connect";
import type { UserService } from "@/gen/user_pb";
import { errorCopy } from "@/account/copy.ts";
import { HANDLE_MAX_LENGTH, isValidHandle, localHandleStatus } from "@/account/handle.ts";
import { type AccountHostDeps, handleAccountMessage } from "@/account/host.ts";
import { type AccountProfile, userServicePort } from "@/account/profile.ts";
import { AccountError, authErrorFromCode } from "@/state/auth-errors.ts";

describe("handle policy", () => {
  it("accepts 1 to 15 letters or digits, Unicode letters included", () => {
    expect(HANDLE_MAX_LENGTH).toBe(15);
    expect(isValidHandle("alice")).toBe(true);
    expect(isValidHandle("Café99")).toBe(true);
    expect(isValidHandle("a".repeat(15))).toBe(true);
  });

  it("rejects empty, too long, spaces and symbols", () => {
    for (const bad of ["", "a".repeat(16), "a b", "a_b", "a-b", "@alice", "alice!"]) {
      expect(isValidHandle(bad), bad).toBe(false);
    }
  });

  it("gives the local status of a candidate", () => {
    expect(localHandleStatus("  ")).toBe("empty");
    expect(localHandleStatus("a b")).toBe("invalid");
    expect(localHandleStatus(" alice ")).toBe("checking");
  });
});

describe("handle failures", () => {
  it("maps an optimistic-concurrency abort to a conflict", () => {
    expect(authErrorFromCode(Code.Aborted)).toBe("conflict");
  });

  it("words a taken handle, the policy and an expired session", () => {
    expect(errorCopy("handle", "alreadyExists")).toContain("pris");
    expect(errorCopy("handle", "conflict")).toContain("pris");
    expect(errorCopy("handle", "invalidArgument")).toContain("lettres ou chiffres");
    expect(errorCopy("handle", "unauthenticated")).toContain("session");
    expect(errorCopy("handle", "unknown")).toContain("erreur");
  });
});

const profile = (over: Partial<AccountProfile> = {}): AccountProfile => ({
  handle: null,
  version: 3,
  displayName: null,
  preferences: '{"theme":"dark"}',
  ...over,
});

function hostSetup(current: AccountProfile | Error = profile(), signedInAtStart = true) {
  let signedIn = signedInAtStart;
  const session = {
    state: vi.fn(() => ({ signedIn })),
    signOut: vi.fn(async () => {
      signedIn = false;
    }),
  };
  const account = {
    profile: vi.fn(async () => {
      if (current instanceof Error) throw current;
      return current;
    }),
    checkHandle: vi.fn(async (handle: string) => handle !== "taken"),
    setHandle: vi.fn(async (handle: string, from: AccountProfile) => ({ ...from, handle, version: from.version + 1 })),
    deleteAccount: vi.fn(async () => {}),
  };
  const deps: AccountHostDeps = {
    session: session as unknown as AccountHostDeps["session"],
    account,
    providers: () => ({ google: false, apple: false }),
    eraseLinguaData: vi.fn(async () => {}),
    onSignedIn: vi.fn(),
  };
  return { session, account, deps };
}

describe("handleAccountMessage — handle", () => {
  it("answers the profile's handle while signed in", async () => {
    expect(await handleAccountMessage({ type: "account:profile" }, hostSetup().deps)).toEqual({
      ok: true,
      state: { signedIn: true },
      handle: null,
    });
    const named = hostSetup(profile({ handle: "alice" }));
    expect((await handleAccountMessage({ type: "account:profile" }, named.deps)).handle).toBe("alice");
  });

  it("does not read a profile while signed out", async () => {
    const { deps, account } = hostSetup(profile(), false);
    expect(await handleAccountMessage({ type: "account:profile" }, deps)).toEqual({
      ok: false,
      error: "unauthenticated",
    });
    expect(account.profile).not.toHaveBeenCalled();
  });

  it("checks availability", async () => {
    const { deps } = hostSetup();
    expect(await handleAccountMessage({ type: "account:checkHandle", handle: "taken" }, deps)).toEqual({
      ok: true,
      available: false,
    });
  });

  it("sets the handle on a freshly read profile", async () => {
    const { deps, account } = hostSetup();
    const reply = await handleAccountMessage({ type: "account:setHandle", handle: "Alice" }, deps);
    expect(account.setHandle).toHaveBeenCalledWith("Alice", profile());
    expect(reply).toEqual({ ok: true, state: { signedIn: true }, handle: "Alice" });
  });

  it("replies with a conflict when the handle was claimed meanwhile", async () => {
    const { deps, account } = hostSetup();
    account.setHandle.mockRejectedValueOnce(new AccountError("conflict"));
    expect(await handleAccountMessage({ type: "account:setHandle", handle: "alice" }, deps)).toEqual({
      ok: false,
      error: "conflict",
    });
  });

  it("abandon deletes a handle-less account, then signs out", async () => {
    const { deps, account, session } = hostSetup();
    const reply = await handleAccountMessage({ type: "account:abandon" }, deps);
    expect(account.deleteAccount).toHaveBeenCalledOnce();
    expect(session.signOut).toHaveBeenCalledOnce();
    expect(reply).toEqual({ ok: true, state: { signedIn: false } });
  });

  it("abandon only signs out an account that has a handle", async () => {
    const { deps, account, session } = hostSetup(profile({ handle: "alice" }));
    await handleAccountMessage({ type: "account:abandon" }, deps);
    expect(account.deleteAccount).not.toHaveBeenCalled();
    expect(session.signOut).toHaveBeenCalledOnce();
  });

  it("abandon never deletes on a guess when the profile cannot be read", async () => {
    const { deps, account, session } = hostSetup(new AccountError("unavailable"));
    await handleAccountMessage({ type: "account:abandon" }, deps);
    expect(account.deleteAccount).not.toHaveBeenCalled();
    expect(session.signOut).toHaveBeenCalledOnce();
  });

  it("abandon still signs out when the delete fails", async () => {
    const { deps, account, session } = hostSetup();
    account.deleteAccount.mockRejectedValueOnce(new AccountError("unavailable"));
    expect((await handleAccountMessage({ type: "account:abandon" }, deps)).ok).toBe(true);
    expect(session.signOut).toHaveBeenCalledOnce();
  });
});

/** A fake UserService client recording its requests. */
function fakeUser(fail: Partial<Record<"getAccount" | "updateAccount" | "checkHandleAvailability", Code>> = {}) {
  const calls = {
    getAccount: vi.fn(),
    updateAccount: vi.fn(),
    checkHandleAvailability: vi.fn(),
    deleteAccount: vi.fn(),
  };
  const account = (handle: string | undefined, displayName: string | undefined) => ({
    userId: "u1",
    handle,
    displayName,
    preferences: '{"theme":"dark"}',
    version: BigInt(7),
    updatedAt: BigInt(0),
  });
  const maybeFail = (method: keyof typeof fail) => {
    const code = fail[method];
    if (code != null) throw new ConnectError(`${method} failed`, code);
  };
  const client = {
    async getAccount(req: unknown) {
      calls.getAccount(req);
      maybeFail("getAccount");
      return account(undefined, "Ada");
    },
    async updateAccount(req: { handle?: string }) {
      calls.updateAccount(req);
      maybeFail("updateAccount");
      return { ...account(req.handle, "Ada"), version: BigInt(8) };
    },
    async checkHandleAvailability(req: { handle: string }) {
      calls.checkHandleAvailability(req);
      maybeFail("checkHandleAvailability");
      return { available: req.handle !== "taken" };
    },
    async deleteAccount(req: unknown) {
      calls.deleteAccount(req);
      return {};
    },
  };
  return { client: client as unknown as Client<typeof UserService>, calls };
}

describe("userServicePort", () => {
  it("reads the profile: no handle is null, the version a number", async () => {
    const { client } = fakeUser();
    expect(await userServicePort(() => client).profile()).toEqual({
      handle: null,
      version: 7,
      displayName: "Ada",
      preferences: '{"theme":"dark"}',
    });
  });

  it("sets the handle with the current version and sends the other fields back", async () => {
    const { client, calls } = fakeUser();
    const port = userServicePort(() => client);
    const updated = await port.setHandle("Alice", profile({ version: 7, displayName: "Ada" }));
    expect(calls.updateAccount).toHaveBeenCalledWith({
      handle: "Alice",
      expectedVersion: BigInt(7),
      preferences: '{"theme":"dark"}',
      displayName: "Ada",
    });
    expect(updated).toMatchObject({ handle: "Alice", version: 8 });

    await port.setHandle("Bob", profile({ displayName: null }));
    expect(calls.updateAccount).toHaveBeenLastCalledWith({
      handle: "Bob",
      expectedVersion: BigInt(3),
      preferences: '{"theme":"dark"}',
    });
  });

  it("checks availability and deletes the account", async () => {
    const { client, calls } = fakeUser();
    const port = userServicePort(() => client);
    expect(await port.checkHandle("taken")).toBe(false);
    expect(await port.checkHandle("alice")).toBe(true);
    await port.deleteAccount();
    expect(calls.deleteAccount).toHaveBeenCalledOnce();
  });

  it("categorizes failures", async () => {
    const { client } = fakeUser({ getAccount: Code.Unavailable, updateAccount: Code.AlreadyExists });
    const port = userServicePort(() => client);
    await expect(port.profile()).rejects.toMatchObject({ kind: "unavailable" });
    await expect(port.setHandle("alice", profile())).rejects.toMatchObject({ kind: "alreadyExists" });
  });
});
