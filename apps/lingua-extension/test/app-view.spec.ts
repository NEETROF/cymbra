import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AppModelView } from "@/app/translation.ts";
import { type AccountLineState, renderAccountLine, renderModelOffer } from "@/app/view.ts";

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
    [
      { relevant: true, signedIn: false, signedOut: false },
      "Dans l'app, la traduction en ligne demande de te connecter avec Apple ou Google.",
    ],
    [{ relevant: false, signedIn: false, signedOut: false }, ""],
    [{ relevant: false, signedIn: false, signedOut: true }, ""],
  ];

  it("says the session, offers to sign in again after a sign-out, or notes that the app needs Apple or Google", () => {
    for (const [state, text] of CASES) {
      renderAccountLine(el, state, "fr", { signIn: vi.fn(), signOut: vi.fn() });
      expect(el.textContent, JSON.stringify(state)).toBe(text);
      expect(el.hidden).toBe(text === "");
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
