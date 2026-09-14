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
  };
  const deps = {
    send: vi.fn(async (message: AccountMessage) => {
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
    expect(viewFromHash("")).toBe("signin");
  });
});

describe("AccountFlow.init", () => {
  it("shows the signed-in view when a session exists", async () => {
    const { flow } = setup({ "account:state": { ok: true, state: { signedIn: true } } });
    expect((await flow.init("#signup")).view).toBe("signedin");
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
    expect(types(sent).slice(-2)).toEqual(["account:verifyEmail", "account:signInLocal"]);
    expect(sent.at(-2)).toEqual({ type: "account:verifyEmail", code: "123456" });
    expect(sent.at(-1)).toEqual({ type: "account:signInLocal", email: "new@example.com", password: PASSWORD });
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
    expect(sent.at(-1)).toEqual({ type: "account:signInLocal", email: "me@example.com", password: PASSWORD });
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
    expect(sent.at(-1)).toEqual({ type: "account:signInApple" });
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

  it("signs out back to sign-in", async () => {
    const { flow, sent } = setup({ "account:state": { ok: true, state: { signedIn: true } } });
    await flow.init("");
    const s = await flow.signOut();
    expect(sent.at(-1)).toEqual({ type: "account:signOut" });
    expect(s.view).toBe("signin");
  });
});
