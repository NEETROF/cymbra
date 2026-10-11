import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it, vi } from "vitest";
import { accountCopy, errorCopy } from "@/account/copy.ts";
import { AccountFlow, type AccountViewState, providerName } from "@/account/flow.ts";
import type { AccountMessage, AccountReply } from "@/account/messages.ts";
import { mountAccountPage } from "@/account/page.ts";
import { type AccountActions, linkedOn, renderAccount } from "@/account/view.ts";
import { COPY_PENDING_ATTR, fillPageInLanguage, INTERFACE_LANGUAGE_KEY } from "@/i18n/index.ts";
import { NATIVE_CHOSEN_KEY } from "@/state/native-language.ts";
import { LAST_NATIVE_KEY } from "@/state/store.ts";
import { PENDING_RULE, pageArea, refusingArea, REVEAL_KEYFRAMES } from "./helpers.ts";

// The account page (localise-lingua-account-onboarding D1, D3): its skeleton holds no text; opened as
// account.ts opens it — `fillPageInLanguage` over the preferences area, its first read of its own,
// with the account's modules — every node holds byte for byte what the page held before this change
// in French (its line breaks collapsed as a browser shows them), the page says its language, and the
// mark that hid the body is gone, a storage that cannot be read included. The flows, the views and
// the errors in plain words take the language account.ts read; their French is asserted, unchanged,
// by the account-copy, account-flow and account-view specs, and their English here.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const HTML = readFileSync(join(root, "src/account/account.html"), "utf8");
const CSS = readFileSync(join(root, "src/account/account.css"), "utf8");

function page(): Document {
  return new DOMParser().parseFromString(HTML, "text/html");
}

/** Every `data-copy` node of the page, by selector, with the text the page held before. */
const TEXTS: Array<[string, string]> = [
  ["title", "Compte — Cymbra Lingua"],
  ["h1", "Cymbra Lingua"],
  [
    ".account-footnote",
    "Ton compte Cymbra est le même que dans Cymbra Music. Sans compte, l'extension fonctionne entièrement sur cet appareil.",
  ],
];

/** 4 October 2026 at noon UTC, in Unix seconds: the same day in every time zone the tests run in. */
const OCTOBER_4 = Date.UTC(2026, 9, 4, 12) / 1000;

afterEach(() => {
  vi.restoreAllMocks();
});

describe("the account page, filled from the catalogue", () => {
  it("A page before its script: holds no text, and hides its body until filled", () => {
    const doc = page();
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(true);
    expect(doc.documentElement.hasAttribute("lang")).toBe(false);
    for (const [selector] of TEXTS) expect(doc.querySelector(selector)?.textContent, selector).toBe("");
    expect(doc.getElementById("account-root")?.childElementCount).toBe(0);
    // Hidden while pending — and shown after a moment even if the script never fills it.
    expect(CSS).toMatch(PENDING_RULE);
    expect(CSS).toMatch(REVEAL_KEYFRAMES);
  });

  it("Every reader today: opened as account.ts opens it, every node holds the text the page held", async () => {
    const doc = page();
    const { language, copy } = await fillPageInLanguage(doc, pageArea(), accountCopy);
    expect(language).toBe("fr");
    expect(copy).toBe(accountCopy());
    for (const [selector, text] of TEXTS) expect(doc.querySelector(selector)?.textContent, selector).toBe(text);
    expect(doc.querySelectorAll("[data-copy]").length).toBe(TEXTS.length);
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
    expect(doc.documentElement.lang).toBe("fr");
  });

  it("A storage that cannot be read: the page still shows, in French, and says so", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const doc = page();
    await fillPageInLanguage(doc, refusingArea(), accountCopy);
    for (const [selector, text] of TEXTS) expect(doc.querySelector(selector)?.textContent, selector).toBe(text);
    expect(doc.documentElement.lang).toBe("fr");
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
    expect(warn).toHaveBeenCalledWith(expect.stringContaining("[Cymbra Lingua]"), expect.any(Error));
  });

  it("dates a link in French as before: `fr-FR`, the day, the month and the year", () => {
    expect(linkedOn(OCTOBER_4)).toBe("Lié le 4 octobre 2026");
    expect(linkedOn(OCTOBER_4, "fr")).toBe("Lié le 4 octobre 2026");
  });
});

describe("An English-native reader signs in (localise-lingua-account-onboarding)", () => {
  /** What account.ts builds for an English interface: the flow and the view in the language read. */
  function englishPage() {
    const sent: AccountMessage[] = [];
    const replies: Partial<Record<AccountMessage["type"], AccountReply>> = {
      "account:state": { ok: true, state: { signedIn: false } },
      "account:providers": { ok: true, providers: { google: true, apple: true } },
      "account:signInLocal": { ok: false, error: "unauthenticated" },
    };
    const root = document.createElement("div");
    document.body.replaceChildren(root);
    const actions = new Proxy({} as AccountActions, { get: () => () => {} });
    const states: AccountViewState[] = [];
    const flow = new AccountFlow(
      {
        send: async (message) => {
          sent.push(message);
          return replies[message.type] ?? { ok: true };
        },
        pending: { get: async () => null, set: async () => {} },
        pendingPassword: { get: async () => null, set: async () => {} },
        locale: "en",
        language: "en",
        clearPersistedError: async () => {},
      },
      (state) => {
        states.push(state);
        renderAccount(root, state, actions, "en");
      },
    );
    const labels = (): string[] => [...root.querySelectorAll("button")].map((b) => b.textContent ?? "");
    return { flow, root, sent, states, labels };
  }

  it("the page, the flows, the buttons and the errors in plain words are the English catalogue's", async () => {
    const doc = page();
    await fillPageInLanguage(doc, pageArea({ [INTERFACE_LANGUAGE_KEY]: "en" }), accountCopy);
    expect(doc.title).toBe("Account — Cymbra Lingua");
    expect(doc.querySelector(".account-footnote")?.textContent).toBe(
      "Your Cymbra account is the same as in Cymbra Music. Without an account, the extension works entirely on this device.",
    );
    expect(doc.documentElement.lang).toBe("en");
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);

    const { flow, root, labels } = englishPage();
    await flow.init("");
    expect(root.querySelector("h2")?.textContent).toBe("Sign in");
    expect(labels()).toEqual([
      "Continue with Google",
      "Continue with Apple",
      "Sign in",
      "Forgot your password?",
      "Create an account",
    ]);
    expect(root.querySelector(".account-sep")?.textContent).toBe("or with your email");
    expect([...root.querySelectorAll(".account-label")].map((l) => l.textContent)).toEqual(["Email", "Password"]);

    const failed = await flow.signInEmail("me@example.com", "wrong");
    expect(failed.error).toBe("Incorrect email or password.");
    expect(root.querySelector("[role=alert]")?.textContent).toBe("Incorrect email or password.");
  });

  it("the code step renders its message around the address in bold, and the notices are English", async () => {
    const { flow, root } = englishPage();
    await flow.init("#signup");
    const s = await flow.signUp("new@example.com", "a long passphrase");
    expect(s.notice).toBe("A verification code was sent to new@example.com.");
    const lead = root.querySelector(".account-lead")!;
    expect(lead.textContent).toBe("Enter the code sent to new@example.com");
    expect(lead.querySelector("b")?.textContent).toBe("new@example.com");
    expect(lead.firstChild?.textContent).toBe("Enter the code sent to ");
  });

  it("errors, providers and dates in English, the deletion page in English", () => {
    const flow = englishPage().flow;
    expect(flow.view().deleteAccountUrl).toBe("https://cymbra.app/en/delete-account/");
    expect(errorCopy("signUp", "alreadyExists", "en")).toBe("An account already uses this email.");
    expect(errorCopy("linkApple", "alreadyExists", "en")).toBe(
      "This Apple account is already linked to another Cymbra account.",
    );
    expect(errorCopy("handle", "invalidArgument", "en")).toBe("1 to 15 letters or digits only (no spaces or symbols).");
    expect(errorCopy("forgot", "unavailable", "en")).toBe(
      "Cymbra can't be reached. Check your connection and try again.",
    );
    expect(providerName("local", "en")).toBe("Email and password");
    expect(linkedOn(OCTOBER_4, "en")).toBe("Linked on October 4, 2026");
    expect(linkedOn(OCTOBER_4, "es")).toBe("Vinculado el 4 de octubre de 2026");
  });

  it("the handle step's help and the connected accounts in English", () => {
    const { flow, root, labels } = englishPage();
    const base = flow.view();
    const actions = new Proxy({} as AccountActions, { get: () => () => {} });
    renderAccount(root, { ...base, view: "handle", handleStatus: "empty" }, actions, "en");
    expect(root.querySelector(".account-help")?.textContent).toBe("1 to 15 letters or digits.");
    renderAccount(
      root,
      {
        ...base,
        view: "connected",
        identities: [{ provider: "google", subject: "g-1", linkedAt: OCTOBER_4 }],
        removing: null,
      },
      actions,
      "en",
    );
    expect(root.querySelector("h2")?.textContent).toBe("Connected accounts");
    expect(root.querySelector(".account-identity")?.textContent).toContain("Linked on October 4, 2026");
    expect(root.textContent).toContain("You can't remove your only sign-in method.");
    expect(labels()).toContain("Set a password");
    expect(labels()).toContain("Back");
  });
});

describe("the entry's wiring: mountAccountPage (localise-lingua-account-onboarding D1, D2)", () => {
  interface Mounted {
    doc: Document;
    sent: AccountMessage[];
    hashes: string[];
  }

  async function mount(options: {
    area: { get(key: string): Promise<Record<string, unknown>> };
    /** The background's store; nothing recorded in it unless given. */
    store?: { get(key: string): Promise<Record<string, unknown>> };
    browserLanguage: string;
    hash?: string;
    signedIn?: boolean;
    pendingEmail?: string | null;
  }): Promise<Mounted & { mounted: Awaited<ReturnType<typeof mountAccountPage>> }> {
    const doc = page();
    const sent: AccountMessage[] = [];
    const hashes: string[] = [];
    let hash = options.hash ?? "";
    const replies: Partial<Record<AccountMessage["type"], AccountReply>> = {
      "account:state": { ok: true, state: { signedIn: options.signedIn ?? false } },
      "account:providers": { ok: true, providers: { google: true, apple: true } },
      "account:profile": { ok: true, handle: "ana" },
    };
    const mounted = await mountAccountPage(doc, {
      area: options.area,
      store: options.store ?? pageArea(),
      browserLanguage: options.browserLanguage,
      send: async (message) => {
        sent.push(message);
        return replies[message.type] ?? { ok: true };
      },
      pending: { get: async () => options.pendingEmail ?? null, set: async () => {} },
      pendingPassword: { get: async () => null, set: async () => {} },
      clearPersistedError: async () => {},
      hash: () => hash,
      replaceHash: (next) => {
        hash = next;
        hashes.push(next);
      },
    });
    return { doc, sent, hashes, mounted };
  }

  it("English chosen in an Italian browser: the page in English, `it` sent to the server", async () => {
    const { doc, sent, mounted } = await mount({
      area: pageArea({ [INTERFACE_LANGUAGE_KEY]: "en", [NATIVE_CHOSEN_KEY]: true }),
      store: pageArea({ [LAST_NATIVE_KEY]: "en" }),
      browserLanguage: "it-IT",
      hash: "#signup",
    });
    expect(mounted?.language).toBe("en");
    expect(doc.documentElement.lang).toBe("en");
    expect(doc.title).toBe("Account — Cymbra Lingua");
    expect(doc.querySelector("#account-root h2")?.textContent).toBe("Create a Cymbra account");
    await mounted!.flow.signUp("new@example.com", "a long passphrase");
    expect(sent.find((m) => m.type === "account:signUp")).toEqual({
      type: "account:signUp",
      email: "new@example.com",
      password: "a long passphrase",
      locale: "it",
    });
    expect(doc.querySelector("#account-root .account-lead")?.textContent).toBe(
      "Enter the code sent to new@example.com",
    );
  });

  it("an English interface in an Italian browser, nothing chosen on this device: the page in English, the browser's tag on the requests", async () => {
    const { doc, sent, mounted } = await mount({
      // Marked as chosen by an update, with no choice recorded: no choice (D2).
      area: pageArea({ [INTERFACE_LANGUAGE_KEY]: "en", [NATIVE_CHOSEN_KEY]: true }),
      browserLanguage: "it-IT",
      hash: "#signup",
    });
    expect(doc.documentElement.lang).toBe("en");
    await mounted!.flow.signUp("new@example.com", "a long passphrase");
    await mounted!.flow.resend();
    expect(sent.filter((m) => "locale" in m)).toEqual([
      { type: "account:signUp", email: "new@example.com", password: "a long passphrase", locale: "it-IT" },
      // The sign-up's locale: Cymbra ID records it only on an account that has none, so the account
      // keeps its own, whichever device gave it.
      { type: "account:resendVerification", email: "new@example.com", locale: "it-IT" },
    ]);
    expect(mounted!.flow.view().deleteAccountUrl).toBe("https://cymbra.app/en/delete-account/");
  });

  it("reads the interface language first, then whether the reader chose", async () => {
    const keys: string[] = [];
    const area = {
      get: async (key: string): Promise<Record<string, unknown>> => {
        keys.push(key);
        return {};
      },
    };
    await mount({ area, browserLanguage: "fr-FR" });
    expect(keys).toEqual([INTERFACE_LANGUAGE_KEY, NATIVE_CHOSEN_KEY]);
  });

  it("a storage that refuses, signed in: French, the signed-in step resumed, the deletion page by the browser as before", async () => {
    vi.spyOn(console, "warn").mockImplementation(() => {});
    const { doc, hashes } = await mount({ area: refusingArea(), browserLanguage: "en-GB", signedIn: true });
    expect(doc.documentElement.lang).toBe("fr");
    expect(doc.title).toBe("Compte — Cymbra Lingua");
    expect(doc.querySelector("#account-root h2")?.textContent).toBe("Tu es connecté");
    expect(hashes.at(-1)).toBe("#signedin");
    const deletion = [...doc.querySelectorAll<HTMLAnchorElement>("#account-root a")].find(
      (a) => a.textContent === "Supprimer mon compte Cymbra",
    );
    // A read that fails is no choice: the deletion page by the browser's tag, English for `en-GB`.
    expect(deletion?.getAttribute("href")).toBe("https://cymbra.app/en/delete-account/");
  });

  it("French chosen in an English browser, signed in: the French deletion page", async () => {
    const { doc } = await mount({
      area: pageArea({ [INTERFACE_LANGUAGE_KEY]: "fr", [NATIVE_CHOSEN_KEY]: true }),
      store: pageArea({ [LAST_NATIVE_KEY]: "fr" }),
      browserLanguage: "en-GB",
      signedIn: true,
    });
    const deletion = [...doc.querySelectorAll<HTMLAnchorElement>("#account-root a")].find(
      (a) => a.textContent === "Supprimer mon compte Cymbra",
    );
    expect(deletion?.getAttribute("href")).toBe("https://cymbra.app/suppression-compte/");
  });

  it("a storage that refuses, a code pending: French, the code step resumed, a hostile address as text", async () => {
    vi.spyOn(console, "warn").mockImplementation(() => {});
    const hostile = "<img src=x onerror=alert(1)>@x.dev";
    const { doc } = await mount({
      area: refusingArea(),
      browserLanguage: "en-GB",
      hash: "#verify",
      pendingEmail: hostile,
    });
    expect(doc.documentElement.lang).toBe("fr");
    const lead = doc.querySelector("#account-root .account-lead")!;
    expect(lead.textContent).toBe(`Saisis le code envoyé à ${hostile}`);
    expect(lead.querySelector("b")?.textContent).toBe(hostile);
    expect(doc.querySelector("img")).toBeNull();
    expect(doc.querySelector("[onerror]")).toBeNull();
  });
});
