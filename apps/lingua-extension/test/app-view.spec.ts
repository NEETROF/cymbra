import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AppModelView } from "@/app/translation.ts";
import { type AccountLineState, accountLabel, renderAccountLine, renderModelOffer } from "@/app/view.ts";

// The Cymbra Lingua app's own lines beside the extension's Révision (add-lingua-review-translation-fallback
// D7, D8): the models' offer and the app's session line, text nodes and small buttons only.

let el: HTMLElement;
beforeEach(() => {
  document.body.innerHTML = "<p id='l'></p>";
  el = document.getElementById("l")!;
});

const buttons = (): string[] => [...el.querySelectorAll("button")].map((b) => b.textContent ?? "");

describe("the models' line", () => {
  // (view, language, the line's text, its buttons)
  const CASES: [AppModelView, "fr" | "en" | "es", string, string[]][] = [
    [{ phase: "none" }, "fr", "", []],
    [
      { phase: "offer", pairs: ["en-fr"], bytes: 25_800_000 },
      "fr",
      "Traduire aussi dans l'app ? Téléchargement de 25,8 Mo",
      ["Télécharger", "Plus tard"],
    ],
    [
      { phase: "offer", pairs: ["en-fr"], bytes: 25_800_000 },
      "en",
      "Translate in the app too? A 25.8 MB download",
      ["Download", "Not now"],
    ],
    [{ phase: "downloading", received: 13, total: 26 }, "fr", "Téléchargement des modèles… 50 %", []],
    [{ phase: "downloading", received: 0, total: 0 }, "es", "Descargando los modelos… 0 %", []],
    [{ phase: "failed", reason: "network" }, "fr", "Le téléchargement n'a pas abouti.", ["Réessayer", "Plus tard"]],
  ];

  it("shows the offer with its size, the progress, or the failure — hidden with nothing to say", () => {
    for (const [view, language, text, labels] of CASES) {
      renderModelOffer(el, view, language, { accept: vi.fn(), decline: vi.fn() });
      expect(el.hidden, view.phase).toBe(view.phase === "none");
      expect(el.querySelector("span")?.textContent ?? "").toBe(text);
      expect(buttons()).toEqual(labels);
    }
  });

  it("accepts and declines on the reader's tap only", () => {
    const actions = { accept: vi.fn(), decline: vi.fn() };
    renderModelOffer(el, { phase: "offer", pairs: ["en-fr"], bytes: 1 }, "fr", actions);
    expect(actions.accept).not.toHaveBeenCalled();
    el.querySelector<HTMLButtonElement>(".app-model-download")!.click();
    el.querySelector<HTMLButtonElement>(".app-model-later")!.click();
    renderModelOffer(el, { phase: "failed", reason: "network" }, "fr", actions);
    el.querySelector<HTMLButtonElement>(".app-model-retry")!.click();
    expect(actions.accept).toHaveBeenCalledTimes(2);
    expect(actions.decline).toHaveBeenCalledOnce();
  });
});

describe("the app's session line", () => {
  // (state, the line's text)
  const CASES: [AccountLineState, string][] = [
    [
      { relevant: true, signedIn: true, signedOut: false },
      "Connecté à Cymbra pour la traduction en ligne — Se déconnecter",
    ],
    [{ relevant: true, signedIn: false, signedOut: true }, "Se connecter pour la traduction en ligne"],
    // Never signed in (add-lingua-app-email-signin D8): the link, no note restricting the methods.
    [{ relevant: true, signedIn: false, signedOut: false }, "Se connecter pour la traduction en ligne"],
    [{ relevant: false, signedIn: false, signedOut: false }, ""],
    [{ relevant: false, signedIn: false, signedOut: true }, ""],
  ];

  it("says the session, or offers to sign in — never a note restricting sign-in to Apple or Google", () => {
    for (const [state, text] of CASES) {
      renderAccountLine(el, state, "fr", { signIn: vi.fn(), signOut: vi.fn() });
      expect(el.textContent, JSON.stringify(state)).toBe(text);
      expect(el.hidden).toBe(text === "");
      expect(el.textContent).not.toMatch(/Apple|Google/);
    }
  });

  it("Before any sign-in: the sign-in link in every language", () => {
    const LINKS: ["fr" | "en" | "es", string][] = [
      ["fr", "Se connecter pour la traduction en ligne"],
      ["en", "Sign in for online translation"],
      ["es", "Iniciar sesión para la traducción en línea"],
    ];
    for (const [language, text] of LINKS) {
      const actions = { signIn: vi.fn(), signOut: vi.fn() };
      renderAccountLine(el, { relevant: true, signedIn: false, signedOut: false }, language, actions);
      expect(buttons()).toEqual([text]);
      el.querySelector<HTMLButtonElement>(".app-sign-in")!.click();
      expect(actions.signIn).toHaveBeenCalledOnce();
    }
  });

  it("signs in and out on the reader's tap", () => {
    const actions = { signIn: vi.fn(), signOut: vi.fn() };
    renderAccountLine(el, { relevant: true, signedIn: true, signedOut: false }, "en", actions);
    el.querySelector<HTMLButtonElement>(".app-sign-out")!.click();
    renderAccountLine(el, { relevant: true, signedIn: false, signedOut: true }, "en", actions);
    el.querySelector<HTMLButtonElement>(".app-sign-in")!.click();
    expect(actions.signOut).toHaveBeenCalledOnce();
    expect(actions.signIn).toHaveBeenCalledOnce();
  });
});

describe("the account named on the session line (add-lingua-app-email-signin D9)", () => {
  const OUT = "Connecté à Cymbra pour la traduction en ligne — Se déconnecter";
  // (name, language, the line's text)
  const CASES: [AccountLineState["name"], "fr" | "en" | "es", string][] = [
    [
      { handle: "alice", email: "me@example.com" },
      "fr",
      "Connecté en tant que @alice pour la traduction en ligne — Se déconnecter",
    ],
    [
      { handle: null, email: "me@example.com" },
      "fr",
      "Connecté en tant que me@example.com pour la traduction en ligne — Se déconnecter",
    ],
    [null, "fr", OUT],
    [{ handle: null, email: null }, "fr", OUT],
    [{ handle: "alice", email: null }, "en", "Signed in as @alice for online translation — Sign out"],
    [{ handle: "alice", email: null }, "es", "Sesión iniciada como @alice para la traducción en línea — Cerrar sesión"],
  ];

  it("The account signed in is named: the handle, else the email, else the plain line", () => {
    for (const [name, language, text] of CASES) {
      renderAccountLine(el, { relevant: true, signedIn: true, signedOut: false, name }, language, {
        signIn: vi.fn(),
        signOut: vi.fn(),
      });
      expect(el.textContent, JSON.stringify(name)).toBe(text);
    }
  });

  it("renders the name as text, never as markup", () => {
    const name = { handle: "<img src=x onerror=alert(1)>", email: null };
    renderAccountLine(el, { relevant: true, signedIn: true, signedOut: false, name }, "fr", {
      signIn: vi.fn(),
      signOut: vi.fn(),
    });
    expect(el.querySelector("img")).toBeNull();
    expect(el.textContent).toContain("@<img src=x onerror=alert(1)>");
  });

  it("labels the account", () => {
    expect(accountLabel({ handle: "alice", email: "me@example.com" })).toBe("@alice");
    expect(accountLabel({ handle: null, email: "me@example.com" })).toBe("me@example.com");
    expect(accountLabel({ handle: null, email: null })).toBeNull();
    expect(accountLabel(undefined)).toBeNull();
  });
});
