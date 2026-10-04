import { describe, expect, it, vi } from "vitest";
import {
  AccountFlow,
  type AccountViewState,
  deleteAccountUrl,
  providerName,
  viewFromHash,
  wantsConnected,
} from "@/account/flow.ts";
import type { AccountMessage, AccountReply } from "@/account/messages.ts";

type Replies = Partial<Record<AccountMessage["type"], AccountReply | null>>;

const PASSWORD = "correct horse battery";

function setup(replies: Replies = {}, pendingSeed: string | null = null, pendingPasswordSeed: string | null = null) {
  const sent: AccountMessage[] = [];
  let pending = pendingSeed;
  const pendingWrites: (string | null)[] = [];
  let pendingPassword = pendingPasswordSeed;
  const pendingPasswordWrites: (string | null)[] = [];
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
    pendingPassword: {
      get: async () => pendingPassword,
      set: async (email: string | null) => {
        pendingPasswordWrites.push(email);
        pendingPassword = email;
      },
    },
    locale: "fr-FR",
    clearPersistedError: vi.fn(async () => {}),
  };
  const changes: AccountViewState[] = [];
  const flow = new AccountFlow(deps, (s) => changes.push(s));
  return { flow, sent, pendingWrites, pendingPasswordWrites, deps, changes };
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

describe("AccountFlow « Tes données » (add-lingua-privacy-controls)", () => {
  const signedIn: Replies = { "account:state": { ok: true, state: { signedIn: true } } };

  it("links to the deletion page in the reader's language", () => {
    expect(deleteAccountUrl("fr-FR")).toBe("https://cymbra.app/suppression-compte/");
    expect(deleteAccountUrl("fr")).toBe("https://cymbra.app/suppression-compte/");
    expect(deleteAccountUrl("en-US")).toBe("https://cymbra.app/en/delete-account/");
    expect(deleteAccountUrl("de")).toBe("https://cymbra.app/en/delete-account/");
    expect(setup().flow.view().deleteAccountUrl).toBe("https://cymbra.app/suppression-compte/");
  });

  it("erases only after the confirmation, then says so and stays signed in", async () => {
    const { flow, sent } = setup(signedIn);
    await flow.init("");
    await flow.eraseLinguaData(); // not confirmed yet
    expect(types(sent)).not.toContain("account:eraseLinguaData");
    expect(flow.askErase().confirmingErase).toBe(true);
    const s = await flow.eraseLinguaData();
    expect(types(sent)).toContain("account:eraseLinguaData");
    expect(s.confirmingErase).toBe(false);
    expect(s.notice).toContain("autres appareils");
    expect(s.view).toBe("signedin");
  });

  it("cancelling sends nothing", async () => {
    const { flow, sent } = setup(signedIn);
    await flow.init("");
    flow.askErase();
    expect(flow.cancelErase().confirmingErase).toBe(false);
    expect(types(sent)).not.toContain("account:eraseLinguaData");
  });

  it("reports a failed erasure without claiming anything was erased", async () => {
    const { flow } = setup({ ...signedIn, "account:eraseLinguaData": { ok: false, error: "unavailable" } });
    await flow.init("");
    flow.askErase();
    const s = await flow.eraseLinguaData();
    expect(s.error).toBe("Impossible de joindre Cymbra. Vérifie ta connexion et réessaie.");
    expect(s.notice).toBeNull();
    expect(s.confirmingErase).toBe(false);
  });

  it("words a server failure as intact data", async () => {
    const { flow } = setup({ ...signedIn, "account:eraseLinguaData": { ok: false, error: "unknown" } });
    await flow.init("");
    flow.askErase();
    expect((await flow.eraseLinguaData()).error).toContain("Tes données sont intactes");
  });
});

describe("AccountFlow — Comptes connectés (add-lingua-connected-accounts)", () => {
  const GOOGLE = { provider: "google", subject: "g-1", linkedAt: 1_790_000_000 };
  const LOCAL = { provider: "local", subject: "ada@example.com", linkedAt: 1_791_000_000 };
  const signedIn: Replies = {
    "account:state": { ok: true, state: { signedIn: true } },
    "account:identities": { ok: true, identities: [GOOGLE], linkable: { google: true, apple: true } },
  };

  it("reads #connected, and names the methods", () => {
    expect(wantsConnected("#connected")).toBe(true);
    expect(wantsConnected("#connected?x")).toBe(true);
    expect(wantsConnected("#data")).toBe(false);
    expect(viewFromHash("#connected")).toBe("signin"); // never a signed-out view
    expect(providerName("google")).toBe("Google");
    expect(providerName("apple")).toBe("Apple");
    expect(providerName("local")).toBe("Email et mot de passe");
    expect(providerName("github")).toBe("github");
  });

  it("opens on the connected accounts from #connected when signed in", async () => {
    const { flow } = setup(signedIn);
    const s = await flow.init("#connected");
    expect(s.view).toBe("connected");
    expect(s.identities).toEqual([GOOGLE]);
    expect(s.linkable).toEqual({ google: true, apple: true });
    expect(s.passwordStep).toBe("closed");
  });

  it("stays on sign-in from #connected when signed out, and on the handle step without a handle", async () => {
    const out = setup();
    expect((await out.flow.init("#connected")).view).toBe("signin");
    expect(out.sent.map((m) => m.type)).not.toContain("account:identities");
    const noHandle = setup({ ...signedIn, "account:profile": { ok: true, handle: null } });
    expect((await noHandle.flow.init("#connected")).view).toBe("handle");
    expect((await noHandle.flow.openConnected()).view).toBe("handle");
  });

  it("goes there and back from the signed-in view", async () => {
    const { flow } = setup(signedIn);
    await flow.init("");
    expect((await flow.openConnected()).view).toBe("connected");
    expect(flow.leaveConnected().view).toBe("signedin");
    expect(flow.leaveConnected().view).toBe("signedin");
  });

  it("says when the list cannot be read, and reads it again", async () => {
    const { flow, deps } = setup({ ...signedIn, "account:identities": null });
    await flow.init("");
    const s = await flow.openConnected();
    expect(s.identities).toBeNull();
    expect(s.error).toBe("Impossible de joindre Cymbra. Vérifie ta connexion et réessaie.");
    deps.send.mockImplementation(async (m: AccountMessage) =>
      m.type === "account:identities"
        ? { ok: true, identities: [LOCAL], linkable: { google: false, apple: false } }
        : { ok: true },
    );
    expect((await flow.loadIdentities()).identities).toEqual([LOCAL]);
  });

  it("links a provider and reloads, and says nothing on a cancel", async () => {
    const { flow, deps, sent } = setup(signedIn);
    await flow.init("#connected");
    deps.send.mockImplementation(async (m: AccountMessage) => {
      sent.push(m);
      if (m.type === "account:linkProvider") return { ok: true };
      if (m.type === "account:identities")
        return {
          ok: true,
          identities: [GOOGLE, { ...GOOGLE, provider: "apple", subject: "a-1" }],
          linkable: { google: true, apple: true },
        };
      return { ok: true };
    });
    const s = await flow.link("apple");
    expect(sent).toContainEqual({ type: "account:linkProvider", provider: "apple" });
    expect(s.identities?.map((i) => i.provider)).toEqual(["google", "apple"]);
    expect(s.notice).toBe("Apple est lié à ton compte.");

    deps.send.mockImplementation(async () => ({ ok: false, cancelled: true }));
    const cancelled = await flow.link("google");
    expect(cancelled.error).toBeNull();
  });

  it("says a provider already linked to another account", async () => {
    const { flow } = setup({ ...signedIn, "account:linkProvider": { ok: false, error: "alreadyExists" } });
    await flow.init("#connected");
    const s = await flow.link("google");
    expect(s.error).toBe("Ce compte Google est déjà lié à un autre compte Cymbra.");
    expect(s.identities).toEqual([GOOGLE]);
  });

  it("removes a method only once confirmed", async () => {
    const { flow, sent, deps } = setup({
      ...signedIn,
      "account:identities": { ok: true, identities: [GOOGLE, LOCAL], linkable: { google: true, apple: false } },
    });
    await flow.init("#connected");
    expect(await flow.remove()).toMatchObject({ removing: null }); // nothing asked: nothing sent
    expect(sent.map((m) => m.type)).not.toContain("account:unlink");
    flow.askRemove(GOOGLE);
    expect(flow.cancelRemove().removing).toBeNull();
    expect(flow.askRemove(GOOGLE).removing).toEqual(GOOGLE);
    deps.send.mockImplementation(async (m: AccountMessage) => {
      sent.push(m);
      if (m.type === "account:identities")
        return { ok: true, identities: [LOCAL], linkable: { google: true, apple: false } };
      return { ok: true };
    });
    const s = await flow.remove();
    expect(sent).toContainEqual({ type: "account:unlink", provider: "google", subject: "g-1" });
    expect(s.identities).toEqual([LOCAL]);
    expect(s.removing).toBeNull();
    expect(s.notice).toBe("Méthode retirée : Google.");
  });

  it("words a refused removal as the last method", async () => {
    const { flow } = setup({ ...signedIn, "account:unlink": { ok: false, error: "failedPrecondition" } });
    await flow.init("#connected");
    flow.askRemove(GOOGLE);
    const s = await flow.remove();
    expect(s.error).toBe("Tu ne peux pas retirer ta seule méthode de connexion.");
    expect(s.removing).toBeNull();
  });

  it("sets a password: the code step, then the method, the reader still signed in", async () => {
    const { flow, sent, pendingPasswordWrites, deps } = setup(signedIn);
    await flow.init("#connected");
    expect(flow.showPasswordForm().passwordStep).toBe("form");
    expect((await flow.setPassword("  ", PASSWORD)).passwordStep).toBe("form"); // nothing typed
    const s = await flow.setPassword(" ada@example.com ", PASSWORD);
    expect(sent).toContainEqual({
      type: "account:setPassword",
      email: "ada@example.com",
      password: PASSWORD,
      locale: "fr-FR",
    });
    expect(s.passwordStep).toBe("code");
    expect(s.passwordEmail).toBe("ada@example.com");
    expect(pendingPasswordWrites).toEqual(["ada@example.com"]);

    deps.send.mockImplementation(async (m: AccountMessage) => {
      sent.push(m);
      if (m.type === "account:identities")
        return { ok: true, identities: [GOOGLE, LOCAL], linkable: { google: true, apple: true } };
      return { ok: true };
    });
    expect((await flow.confirmPassword(" ")).passwordStep).toBe("code"); // no code
    const done = await flow.confirmPassword(" 123456 ");
    expect(sent).toContainEqual({ type: "account:verifyEmail", code: "123456" });
    expect(sent.map((m) => m.type)).not.toContain("account:signInLocal");
    expect(done.view).toBe("connected");
    expect(done.passwordStep).toBe("closed");
    expect(done.identities).toEqual([GOOGLE, LOCAL]);
    expect(done.notice).toContain("ada@example.com");
    expect(pendingPasswordWrites).toEqual(["ada@example.com", null]);
    // The password went to the background once, and nowhere else.
    expect(JSON.stringify(pendingPasswordWrites)).not.toContain(PASSWORD);
  });

  it("says a refused submission and a wrong code in their own words", async () => {
    const { flow } = setup({ ...signedIn, "account:setPassword": { ok: false, error: "alreadyExists" } });
    await flow.init("#connected");
    flow.showPasswordForm();
    const refused = await flow.setPassword("ada@example.com", PASSWORD);
    expect(refused.passwordStep).toBe("form");
    expect(refused.error).toContain("déjà utilisée");
    const code = setup(
      { ...signedIn, "account:verifyEmail": { ok: false, error: "invalidArgument" } },
      null,
      "ada@example.com",
    );
    await code.flow.init("#connected");
    const wrong = await code.flow.confirmPassword("000000");
    expect(wrong.passwordStep).toBe("code");
    expect(wrong.error).toContain("expiré");
  });

  it("resumes on the code step after a reload, and starts over or closes", async () => {
    const { flow, pendingPasswordWrites } = setup(signedIn, null, "ada@example.com");
    const s = await flow.init("#connected");
    expect(s.passwordStep).toBe("code");
    expect(s.passwordEmail).toBe("ada@example.com");
    const again = await flow.restartPassword();
    expect(again.passwordStep).toBe("form");
    expect(again.passwordEmail).toBe("ada@example.com");
    expect((await flow.cancelPassword()).passwordStep).toBe("closed");
    expect(pendingPasswordWrites).toEqual([null, null]);
  });

  it("forgets everything about the connected accounts on sign-out", async () => {
    const { flow, pendingPasswordWrites } = setup(signedIn, null, "ada@example.com");
    await flow.init("#connected");
    const s = await flow.signOut();
    expect(s).toMatchObject({ view: "signin", identities: null, passwordStep: "closed", passwordEmail: "" });
    expect(pendingPasswordWrites).toContain(null);
  });
});
