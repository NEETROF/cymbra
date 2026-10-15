import { beforeEach, describe, expect, it, vi } from "vitest";
import { onboarding as enOnboarding } from "@/i18n/en/onboarding.ts";
import { onboarding as esOnboarding } from "@/i18n/es/onboarding.ts";
import { onboarding as frOnboarding } from "@/i18n/fr/onboarding.ts";
import { popup as enPopup } from "@/i18n/en/popup.ts";
import { popup as esPopup } from "@/i18n/es/popup.ts";
import { popup as frPopup } from "@/i18n/fr/popup.ts";
import type { NoticeKind } from "@/usage/counts.ts";
import type { NoticeControls } from "@/usage/messages.ts";
import { mountUsageNotice, noticeCopyOf, whenVisible } from "@/usage/notice.ts";

// The reader is told before any report leaves (add-lingua-usage-counts D7): one sentence on the
// welcome tab or in the popup — with two equal buttons on Chromium — never over a page.

const PRIVACY = "https://cymbra.app/confidentialite/";
const settle = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));

function controls(kind: NoticeKind) {
  const c = {
    status: vi.fn(async () => kind),
    shown: vi.fn(async () => {}),
    answer: vi.fn(async (yes: boolean) => void yes),
  } satisfies NoticeControls;
  return c;
}

function container(): HTMLElement {
  const el = document.createElement("section");
  el.hidden = true;
  document.body.replaceChildren(el);
  return el;
}

describe("The reader is told before any report leaves", () => {
  beforeEach(() => document.body.replaceChildren());

  it("Chrome, welcome tab: the sentence with its two buttons, and nothing said before one is pressed", async () => {
    const el = container();
    const c = controls("ask");
    expect(await mountUsageNotice(el, { copy: noticeCopyOf(frOnboarding), controls: c, privacyUrl: PRIVACY })).toBe(
      "ask",
    );
    expect(el.hidden).toBe(false);
    expect(el.querySelector(".usage-notice-text")?.textContent).toBe(frOnboarding.usageAsk);
    const buttons = [...el.querySelectorAll<HTMLButtonElement>("button")];
    expect(buttons.map((b) => b.textContent)).toEqual(["Oui, compter", "Non merci"]);
    // Equal in size and weight: the same class, neither marked the default.
    expect(new Set(buttons.map((b) => b.className)).size).toBe(1);
    expect(c.shown).not.toHaveBeenCalled();
    expect(c.answer).not.toHaveBeenCalled();
    const link = el.querySelector<HTMLAnchorElement>("a")!;
    expect(link.href).toBe(PRIVACY);
    expect(link.textContent).toBe("Politique de confidentialité");
  });

  it("« Oui, compter »: the answer is sent once, and the reader thanked", async () => {
    const el = container();
    const c = controls("ask");
    await mountUsageNotice(el, { copy: noticeCopyOf(frPopup), controls: c, privacyUrl: PRIVACY });
    const [yes, no] = el.querySelectorAll<HTMLButtonElement>("button");
    yes.click();
    no.click(); // disabled by then
    await settle();
    expect(c.answer.mock.calls).toEqual([[true]]);
    expect(el.querySelector(".usage-notice-done")?.textContent).toBe(frPopup.usageYesDone);
    expect(el.querySelector<HTMLElement>(".usage-notice-answers")!.hidden).toBe(true);
  });

  it("« Non merci »: the switch turned off, said plainly", async () => {
    const el = container();
    const c = controls("ask");
    await mountUsageNotice(el, { copy: noticeCopyOf(enPopup), controls: c, privacyUrl: PRIVACY });
    el.querySelectorAll<HTMLButtonElement>("button")[1].click();
    await settle();
    expect(c.answer.mock.calls).toEqual([[false]]);
    expect(el.querySelector(".usage-notice-done")?.textContent).toBe("Noted: nothing will be counted.");
  });

  it("Safari, popup first: the sentence is shown, and the background told it was", async () => {
    const el = container();
    const c = controls("inform");
    await mountUsageNotice(el, { copy: noticeCopyOf(esPopup), controls: c, privacyUrl: PRIVACY });
    expect(el.hidden).toBe(false);
    expect(el.querySelector(".usage-notice-text")?.textContent).toBe(esPopup.usageInform);
    expect(el.querySelectorAll("button")).toHaveLength(0);
    expect(c.shown).toHaveBeenCalledTimes(1);
  });

  it("the popup opened on Réglages › Données: the sentence is told as shown only once it is on screen", async () => {
    // Review of #958: the main view — and the sentence in it — is hidden when the popup opens on
    // a settings tab; Safari's gate must not open on a sentence nobody saw.
    const el = container();
    const c = controls("inform");
    let onScreen!: () => void;
    const visible = vi.fn(() => new Promise<void>((resolve) => (onScreen = resolve)));
    const mounted = mountUsageNotice(el, { copy: noticeCopyOf(frPopup), controls: c, privacyUrl: PRIVACY, visible });
    await settle();
    expect(visible).toHaveBeenCalledWith(el);
    expect(c.shown).not.toHaveBeenCalled();
    // The reader goes back to the main view: the sentence is on screen.
    onScreen();
    expect(await mounted).toBe("inform");
    expect(c.shown).toHaveBeenCalledTimes(1);
  });

  it("on screen means intersecting the viewport, observed once", async () => {
    const observed: Element[] = [];
    let report!: (entries: { isIntersecting: boolean }[]) => void;
    const disconnect = vi.fn();
    class FakeObserver {
      constructor(callback: (entries: { isIntersecting: boolean }[]) => void) {
        report = callback;
      }
      observe(el: Element): void {
        observed.push(el);
      }
      disconnect = disconnect;
    }
    vi.stubGlobal("IntersectionObserver", FakeObserver);
    try {
      const el = container();
      let resolved = false;
      const seen = whenVisible(el).then(() => (resolved = true));
      expect(observed).toEqual([el]);
      // Hidden behind the settings view: not yet.
      report([{ isIntersecting: false }]);
      await settle();
      expect(resolved).toBe(false);
      report([{ isIntersecting: true }]);
      await seen;
      expect(disconnect).toHaveBeenCalledTimes(1);
    } finally {
      vi.unstubAllGlobals();
    }
  });

  it("nothing due: nothing shown, nothing said", async () => {
    const el = container();
    const c = controls("none");
    await mountUsageNotice(el, { copy: noticeCopyOf(enOnboarding), controls: c, privacyUrl: PRIVACY });
    expect(el.hidden).toBe(true);
    expect(el.childElementCount).toBe(0);
    expect(c.shown).not.toHaveBeenCalled();
  });

  it("the sentence says what is counted, that nothing read leaves, and where to turn it off — in every language", () => {
    for (const m of [frOnboarding, frPopup]) {
      for (const text of [m.usageInform, m.usageAsk]) {
        expect(text).toContain("aucun identifiant");
        expect(text).toMatch(/rien de ce que tu lis ne quitte ton appareil/i);
        expect(text).toContain("Réglages → Données");
      }
    }
    for (const m of [enOnboarding, enPopup]) {
      expect(m.usageInform).toContain("no identifier");
      expect(m.usageAsk).toContain("Settings → Data");
      expect([m.usageYes, m.usageNo]).toEqual(["Yes, count", "No, thanks"]);
    }
    for (const m of [esOnboarding, esPopup]) {
      expect(m.usageInform).toContain("sin ningún identificador");
      expect(m.usageAsk).toContain("Ajustes → Datos");
      expect([m.usageYes, m.usageNo]).toEqual(["Sí, contar", "No, gracias"]);
    }
  });
});
