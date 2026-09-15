import { describe, expect, it, vi } from "vitest";
import { AccountFlow, type AccountViewState, viewFromHash } from "@/account/flow.ts";
import type { AccountMessage, AccountReply } from "@/account/messages.ts";

type Replies = Partial<Record<AccountMessage["type"], AccountReply | null>>;

const PASSWORD = "correct horse battery";

function setup(replies: Replies = {}, pendingSeed: string | null = null) {
  const sent: AccountMessage[] = [];
  let pending = pendingSeed;
  const pendingWrites: (string | null)[] = [];
  const defaults: Replies = {
    "account:state": { ok: true, state: { signedIn: false } },
    "account:providers": { ok: true, providers: { google: true, apple: true } },
    "account:profile": { ok: true, handle: "alice" },
  };
  const deps = {
    send: vi.fn(async (message: AccountMessage): Promise<AccountReply | null> => {
      sent.push(message);
      if (message.type in replies) return replies[message.type] ?? null;
      return defaults[message.type] ?? { ok: true };
    }),
    pending: {
      get: async () => pending,
      set: async (email: string | null) => {
        pendingWrites.push(email);
        pending = email;
      },
    },
    locale: "fr-FR",
    clearPersistedError: vi.fn(async () => {}),
  };
  const changes: AccountViewState[] = [];
  const flow = new AccountFlow(deps, (s) => changes.push(s));
  return { flow, sent, pendingWrites, deps, changes };
}

const types = (sent: AccountMessage[]) => sent.map((m) => m.type);

describe("viewFromHash", () => {
  it("maps known hashes and defaults to sign-in", () => {
    expect(viewFromHash("#signup")).toBe("signup");
    expect(viewFromHash("#forgot")).toBe("forgot");
    expect(viewFromHash("#verify")).toBe("verify");
    expect(viewFromHash("#signedin")).toBe("signin");
    expect(viewFromHash("#handle")).toBe("signin");
    expect(viewFromHash("")).toBe("signin");
  });
});

describe("AccountFlow.init", () => {
  it("shows the signed-in view with the handle when a session exists", async () => {
    const { flow } = setup({ "account:state": { ok: true, state: { signedIn: true } } });
    const s = await flow.init("#signup");
    expect(s.view).toBe("signedin");
    expect(s.handle).toBe("alice");
  });

  it("opens the requested view with the available providers", async () => {
    const { flow } = setup({ "account:providers": { ok: true, providers: { google: false, apple: true } } });
    const s = await flow.init("#signup");
    expect(s.view).toBe("signup");
    expect(s.providers).toEqual({ google: false, apple: true });
  });

  it("resumes the code step from the pending email, and refuses it without one", async () => {
    expect((await setup({}, "me@example.com").flow.init("#verify")).view).toBe("verify");
    expect((await setup({}, "me@example.com").flow.init("#verify")).email).toBe("me@example.com");
    expect((await setup().flow.init("#verify")).view).toBe("signin");
  });

  it("hides the providers when the worker does not answer", async () => {
    const { flow } = setup({ "account:providers": null, "account:state": null });
    const s = await flow.init("");
    expect(s.providers).toEqual({ google: false, apple: false });
    expect(s.view).toBe("signin");
  });
});

describe("AccountFlow sign-up → code → signed in", () => {
  it("signs up, verifies and signs straight in with the held password", async () => {
    const { flow, sent, pendingWrites } = setup();
    await flow.init("#signup");
    let s = await flow.signUp(" new@example.com ", PASSWORD);
    expect(sent.at(-1)).toEqual({
      type: "account:signUp",
      email: "new@example.com",
      password: PASSWORD,
      locale: "fr-FR",
    });
    expect(s.view).toBe("verify");
    expect(s.notice).toContain("new@example.com");

    s = await flow.verify(" 123456 ");
    expect(types(sent).slice(-3)).toEqual(["account:verifyEmail", "account:signInLocal", "account:profile"]);
    expect(sent.at(-3)).toEqual({ type: "account:verifyEmail", code: "123456" });
    expect(sent.at(-2)).toEqual({ type: "account:signInLocal", email: "new@example.com", password: PASSWORD });
    expect(s.view).toBe("signedin");
    expect(pendingWrites).toEqual(["new@example.com", null]);
  });

  it("never writes the password to storage", async () => {
    const { flow, pendingWrites } = setup({
      "account:signInLocal": { ok: false, error: "failedPrecondition" },
    });
    await flow.init("");
    await flow.signUp("new@example.com", PASSWORD);
    await flow.signInEmail("new@example.com", PASSWORD);
    expect(JSON.stringify(pendingWrites)).not.toContain(PASSWORD);
  });

  it("keeps the form on a taken email and flags it for the sign-in/forgot links", async () => {
    const { flow } = setup({ "account:signUp": { ok: false, error: "alreadyExists" } });
    await flow.init("#signup");
    const s = await flow.signUp("taken@example.com", PASSWORD);
    expect(s.view).toBe("signup");
    expect(s.errorKind).toBe("alreadyExists");
    expect(s.error).toContain("déjà");
    expect(s.email).toBe("taken@example.com");
  });

  it("returns to sign-in after a reload, since the password is gone", async () => {
    const { flow, sent } = setup({}, "me@example.com");
    await flow.init("#verify");
    const s = await flow.verify("123456");
    expect(types(sent)).not.toContain("account:signInLocal");
    expect(s.view).toBe("signin");
    expect(s.email).toBe("me@example.com");
    expect(s.notice).toContain("vérifiée");
  });

  it("reports an invalid code and stays on the code step", async () => {
    const { flow } = setup({ "account:verifyEmail": { ok: false, error: "invalidArgument" } }, "me@example.com");
    await flow.init("#verify");
    const s = await flow.verify("000000");
    expect(s.view).toBe("verify");
    expect(s.error).toContain("expiré");
  });

  it("lands on sign-in with the error when the automatic sign-in fails", async () => {
    const { flow } = setup({ "account:signInLocal": { ok: false, error: "rateLimited" } });
    await flow.init("#signup");
    await flow.signUp("new@example.com", PASSWORD);
    const s = await flow.verify("123456");
    expect(s.view).toBe("signin");
    expect(s.error).toContain("Trop de tentatives");
  });

  it("resends the code to the pending email", async () => {
    const { flow, sent } = setup({}, "me@example.com");
    await flow.init("#verify");
    const s = await flow.resend();
    expect(sent.at(-1)).toEqual({ type: "account:resendVerification", email: "me@example.com", locale: "fr-FR" });
    expect(s.notice).toContain("nouveau code");
  });

  it("marks the state busy while a call is in flight", async () => {
    const { flow, changes } = setup();
    await flow.init("#signup");
    await flow.signUp("new@example.com", PASSWORD);
    expect(changes.some((c) => c.busy)).toBe(true);
    expect(flow.view().busy).toBe(false);
  });
});

describe("AccountFlow sign-in", () => {
  it("routes an unverified email to the code step, then signs in after verification", async () => {
    let verified = false;
    const { flow, sent, pendingWrites, deps } = setup();
    deps.send.mockImplementation(async (m: AccountMessage) => {
      sent.push(m);
      if (m.type === "account:state") return { ok: true, state: { signedIn: false } };
      if (m.type === "account:providers") return { ok: true, providers: { google: false, apple: false } };
      if (m.type === "account:profile") return { ok: true, handle: "alice" };
      if (m.type === "account:verifyEmail") verified = true;
      if (m.type === "account:signInLocal" && !verified) return { ok: false, error: "failedPrecondition" };
      return { ok: true, state: { signedIn: true } };
    });
    await flow.init("");
    let s = await flow.signInEmail("me@example.com", PASSWORD);
    expect(s.view).toBe("verify");
    expect(s.error).toBeNull();
    expect(pendingWrites).toEqual(["me@example.com"]);
    s = await flow.verify("123456");
    expect(s.view).toBe("signedin");
    expect(sent.at(-2)).toEqual({ type: "account:signInLocal", email: "me@example.com", password: PASSWORD });
  });

  it("shows the credential error for a wrong password", async () => {
    const { flow } = setup({ "account:signInLocal": { ok: false, error: "unauthenticated" } });
    await flow.init("");
    const s = await flow.signInEmail("me@example.com", "wrong");
    expect(s.view).toBe("signin");
    expect(s.error).toBe("Email ou mot de passe incorrect.");
  });

  it("treats a silent worker as unreachable", async () => {
    const { flow } = setup({ "account:signInLocal": null });
    await flow.init("");
    const s = await flow.signInEmail("me@example.com", "pw");
    expect(s.errorKind).toBe("unavailable");
    expect(s.error).toContain("Impossible de joindre Cymbra");
  });

  it("ignores empty submissions", async () => {
    const { flow, sent } = setup();
    await flow.init("");
    await flow.signInEmail("  ", "pw");
    await flow.signUp("a@b.c", "");
    await flow.verify(" ");
    await flow.requestReset("");
    await flow.resetPassword("", "x");
    expect(types(sent)).toEqual(["account:state", "account:providers"]);
  });
});

describe("AccountFlow providers", () => {
  it("signs in with Apple", async () => {
    const { flow, sent } = setup();
    await flow.init("");
    const s = await flow.signInWith("apple");
    expect(sent.at(-2)).toEqual({ type: "account:signInApple" });
    expect(s.view).toBe("signedin");
  });

  it("shows nothing when the reader closes the window", async () => {
    const { flow, deps } = setup({ "account:signInGoogle": { ok: false, cancelled: true } });
    await flow.init("");
    const s = await flow.signInWith("google");
    expect(s.error).toBeNull();
    expect(s.view).toBe("signin");
    expect(deps.clearPersistedError).not.toHaveBeenCalled();
  });

  it("shows an Apple failure live and drops the persisted copy", async () => {
    const { flow, deps } = setup({ "account:signInApple": { ok: false, error: "unauthenticated" } });
    await flow.init("");
    const s = await flow.signInWith("apple");
    expect(s.error).toContain("Apple");
    expect(s.error).not.toBe("Email ou mot de passe incorrect.");
    expect(deps.clearPersistedError).toHaveBeenCalledOnce();
  });
});

describe("AccountFlow on Safari, through the host app", () => {
  it("sends the reader to the host app without reporting a failure", async () => {
    const { flow, deps } = setup({ "account:signInApple": { ok: false, handedOff: true } });
    await flow.init("");
    const s = await flow.signInWith("apple");
    expect(s.error).toBeNull();
    expect(s.notice).toContain("Cymbra Lingua");
    expect(s.view).toBe("signin");
    expect(deps.clearPersistedError).not.toHaveBeenCalled();
  });

  it("signs in once the host app handed a token back", async () => {
    const { flow } = setup({ "account:collectHandedToken": { ok: true, provider: "google" } });
    await flow.init("");
    expect((await flow.collectHandedToken()).view).toBe("signedin");
  });

  it("stays put when nothing was handed back or the worker is silent", async () => {
    for (const reply of [{ ok: true }, null]) {
      const { flow } = setup({ "account:collectHandedToken": reply });
      await flow.init("#signup");
      const s = await flow.collectHandedToken();
      expect(s.view).toBe("signup");
      expect(s.error).toBeNull();
    }
  });

  it("shows a rejected token live under its provider and drops the persisted copy", async () => {
    const { flow, deps } = setup({
      "account:collectHandedToken": { ok: false, error: "unauthenticated", provider: "apple" },
    });
    await flow.init("");
    const s = await flow.collectHandedToken();
    expect(s.error).toContain("Apple");
    expect(s.errorKind).toBe("unauthenticated");
    expect(deps.clearPersistedError).toHaveBeenCalledOnce();
  });
});

describe("AccountFlow password reset", () => {
  it("requests a code, then resets and returns to sign-in", async () => {
    const { flow, sent } = setup();
    await flow.init("#forgot");
    let s = await flow.requestReset(" me@example.com ");
    expect(sent.at(-1)).toEqual({ type: "account:requestPasswordReset", email: "me@example.com", locale: "fr-FR" });
    expect(s.view).toBe("reset");
    expect(s.notice).toContain("Si un compte existe");
    s = await flow.resetPassword("654321", "new-password");
    expect(sent.at(-1)).toEqual({ type: "account:resetPassword", code: "654321", newPassword: "new-password" });
    expect(s.view).toBe("signin");
    expect(s.email).toBe("me@example.com");
  });

  it("stays on the reset step with a bad code", async () => {
    const { flow } = setup({ "account:resetPassword": { ok: false, error: "invalidArgument" } });
    await flow.init("#forgot");
    await flow.requestReset("me@example.com");
    const s = await flow.resetPassword("bad", "pw");
    expect(s.view).toBe("reset");
    expect(s.error).toContain("expiré");
  });
});

describe("AccountFlow handle step", () => {
  const handleLess: Replies = { "account:profile": { ok: true, handle: null } };
  const signedInHandleLess: Replies = { ...handleLess, "account:state": { ok: true, state: { signedIn: true } } };

  it("asks for a handle after signing in to an account without one", async () => {
    const { flow } = setup(handleLess);
    await flow.init("");
    const s = await flow.signInEmail("me@example.com", PASSWORD);
    expect(s.view).toBe("handle");
    expect(s.handleStatus).toBe("empty");
  });

  it("asks for a handle after a provider sign-in and when opening signed in", async () => {
    const provider = setup(handleLess);
    await provider.flow.init("");
    expect((await provider.flow.signInWith("google")).view).toBe("handle");
    expect((await setup(signedInHandleLess).flow.init("")).view).toBe("handle");
  });

  it("keeps the session when the profile cannot be read", async () => {
    const { flow } = setup({ "account:state": { ok: true, state: { signedIn: true } }, "account:profile": null });
    const s = await flow.init("");
    expect(s.view).toBe("signedin");
    expect(s.errorKind).toBe("unavailable");
  });

  it("checks the policy locally before asking the server", async () => {
    const { flow, sent } = setup(signedInHandleLess);
    await flow.init("");
    expect(flow.editHandle("a b").handleStatus).toBe("invalid");
    expect(flow.editHandle("  ").handleStatus).toBe("empty");
    await flow.checkHandle();
    expect(types(sent)).not.toContain("account:checkHandle");
    expect(flow.editHandle("alice").handleStatus).toBe("checking");
  });

  it("reports available, taken and an unreachable check", async () => {
    const taken = setup({ ...signedInHandleLess, "account:checkHandle": { ok: true, available: false } });
    await taken.flow.init("");
    taken.flow.editHandle(" alice ");
    expect((await taken.flow.checkHandle()).handleStatus).toBe("taken");
    expect(taken.sent.at(-1)).toEqual({ type: "account:checkHandle", handle: "alice" });

    const free = setup({ ...signedInHandleLess, "account:checkHandle": { ok: true, available: true } });
    await free.flow.init("");
    free.flow.editHandle("alice");
    expect((await free.flow.checkHandle()).handleStatus).toBe("available");

    const offline = setup({ ...signedInHandleLess, "account:checkHandle": null });
    await offline.flow.init("");
    offline.flow.editHandle("alice");
    expect((await offline.flow.checkHandle()).handleStatus).toBe("error");
  });

  it("drops an availability answer overtaken by newer typing", async () => {
    const { flow, deps } = setup(signedInHandleLess);
    await flow.init("");
    const base = deps.send.getMockImplementation()!;
    let answer!: (reply: AccountReply) => void;
    deps.send.mockImplementation((m: AccountMessage) =>
      m.type === "account:checkHandle"
        ? new Promise<AccountReply>((resolve) => {
            answer = resolve;
          })
        : base(m),
    );
    flow.editHandle("ali");
    const stale = flow.checkHandle();
    flow.editHandle("alic");
    answer({ ok: true, available: false });
    expect((await stale).handleStatus).toBe("checking");
    expect(flow.view().candidate).toBe("alic");
  });

  it("saves the handle and lands signed in", async () => {
    const { flow, sent } = setup({ ...signedInHandleLess, "account:setHandle": { ok: true, handle: "Alice" } });
    await flow.init("");
    flow.editHandle(" Alice ");
    const s = await flow.commitHandle();
    expect(sent.at(-1)).toEqual({ type: "account:setHandle", handle: "Alice" });
    expect(s.view).toBe("signedin");
    expect(s.handle).toBe("Alice");
    expect(s.notice).toContain("@Alice");
  });

  it("flags a handle claimed in the meantime", async () => {
    const { flow } = setup({ ...signedInHandleLess, "account:setHandle": { ok: false, error: "conflict" } });
    await flow.init("");
    flow.editHandle("alice");
    const s = await flow.commitHandle();
    expect(s.view).toBe("handle");
    expect(s.handleStatus).toBe("taken");
    expect(s.error).toContain("pris");
  });

  it("refuses to save an invalid handle without asking the server", async () => {
    const { flow, sent } = setup(signedInHandleLess);
    await flow.init("");
    flow.editHandle("a-b");
    expect((await flow.commitHandle()).handleStatus).toBe("invalid");
    flow.editHandle("");
    expect((await flow.commitHandle()).handleStatus).toBe("empty");
    expect(types(sent)).not.toContain("account:setHandle");
  });

  it("leaves the step through abandon, back to sign-in", async () => {
    const { flow, sent } = setup(signedInHandleLess);
    await flow.init("");
    const s = await flow.abandonHandle();
    expect(sent.at(-1)).toEqual({ type: "account:abandon" });
    expect(s.view).toBe("signin");
    expect(s.notice).toContain("déconnecté");
  });

  it("cannot be navigated away from", async () => {
    const { flow } = setup(signedInHandleLess);
    await flow.init("");
    expect(flow.go("signup").view).toBe("handle");
  });
});

describe("AccountFlow navigation", () => {
  it("clears messages on navigation and needs an email for the code step", async () => {
    const { flow } = setup({ "account:signInLocal": { ok: false, error: "unauthenticated" } });
    await flow.init("");
    await flow.signInEmail("me@example.com", "wrong");
    let s = flow.go("forgot");
    expect(s.view).toBe("forgot");
    expect(s.error).toBeNull();
    expect(flow.go("forgot").view).toBe("forgot");
    expect(flow.go("signedin").view).toBe("forgot");
    s = setup().flow.go("verify");
    expect(s.view).toBe("signin");
  });

  it("stays on the signed-in view when a link or hash asks for another", async () => {
    const { flow } = setup({ "account:state": { ok: true, state: { signedIn: true } } });
    await flow.init("");
    expect(flow.go("signin").view).toBe("signedin");
  });

  it("signs out back to sign-in", async () => {
    const { flow, sent } = setup({ "account:state": { ok: true, state: { signedIn: true } } });
    await flow.init("");
    const s = await flow.signOut();
    expect(sent.at(-1)).toEqual({ type: "account:signOut" });
    expect(s.view).toBe("signin");
    expect(s.handle).toBeNull();
  });
});
