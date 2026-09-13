import { beforeEach, describe, expect, it, vi } from "vitest";
import { createHud, type HudActions, type HudState, LinguaHud } from "@/reading/hud.ts";

beforeEach(() => {
  document.body.innerHTML = "";
  document.documentElement.querySelector("#cymbra-lingua-hud-host")?.remove();
});

function actions(over: Partial<HudActions> = {}): HudActions {
  return {
    onReview: vi.fn(),
    onCapture: vi.fn(),
    onStats: vi.fn(),
    onSetLevel: vi.fn(),
    onHide: vi.fn(),
    ...over,
  };
}

function state(over: Partial<HudState> = {}): HudState {
  return { analysable: true, percent: 42, hasLevels: true, declaredLevel: "B1", ...over };
}

function q(el: HTMLElement, sel: string): HTMLElement {
  const found = el.querySelector<HTMLElement>(sel);
  if (!found) throw new Error(`no element "${sel}"`);
  return found;
}

function levelChip(el: HTMLElement, value: string): HTMLButtonElement {
  const b = [...el.querySelectorAll<HTMLButtonElement>(".hud-lvl")].find((x) => x.dataset.lvl === value);
  if (!b) throw new Error(`no level chip "${value}"`);
  return b;
}

describe("in-page HUD pill", () => {
  it("hides itself on a non-analysable page and shows the percentage otherwise", () => {
    const hud = createHud(actions());
    document.body.append(hud.el);

    hud.update(state({ analysable: false }));
    expect(hud.el.hidden).toBe(true);
    expect(hud.visible()).toBe(false);

    hud.update(state({ analysable: true, percent: 73 }));
    expect(hud.el.hidden).toBe(false);
    expect(q(hud.el, ".hud-pct").textContent).toBe("73%");
  });

  it("shows an em dash when the percentage is unknown", () => {
    const hud = createHud(actions());
    hud.update(state({ percent: null }));
    expect(q(hud.el, ".hud-pct").textContent).toBe("—");
  });

  it("stays collapsed until the pill is tapped, then reveals the actions", () => {
    const hud = createHud(actions());
    hud.update(state());
    const row = q(hud.el, ".hud-actions");
    expect(row.hidden).toBe(true);
    q(hud.el, ".hud-pct").click();
    expect(row.hidden).toBe(false);
    q(hud.el, ".hud-pct").click();
    expect(row.hidden).toBe(true);
  });

  it("dispatches the reader actions and collapses afterwards", () => {
    const a = actions();
    const hud = createHud(a);
    hud.update(state());
    const row = q(hud.el, ".hud-actions");

    q(hud.el, ".hud-pct").click(); // expand
    const btn = (label: string): HTMLButtonElement =>
      [...hud.el.querySelectorAll<HTMLButtonElement>(".hud-act")].find((b) => b.textContent === label)!;

    btn("Réviser").click();
    expect(a.onReview).toHaveBeenCalledOnce();
    expect(row.hidden).toBe(true); // collapsed after acting

    q(hud.el, ".hud-pct").click();
    btn("Capturer").click();
    expect(a.onCapture).toHaveBeenCalledOnce();

    q(hud.el, ".hud-pct").click();
    btn("Stats").click();
    expect(a.onStats).toHaveBeenCalledOnce();
  });

  it("toggles the settings popover from the gear and marks the current level", () => {
    const hud = createHud(actions());
    hud.update(state({ declaredLevel: "B1" }));
    const settings = q(hud.el, ".hud-settings");
    expect(settings.hidden).toBe(true);

    q(hud.el, ".hud-pct").click();
    q(hud.el, ".hud-gear").click();
    expect(settings.hidden).toBe(false);
    expect(levelChip(hud.el, "B1").classList.contains("active")).toBe(true);
    expect(levelChip(hud.el, "A1").classList.contains("active")).toBe(false);
  });

  it("declares a level (and 'Débutant' clears it), then closes the popover", () => {
    const a = actions();
    const hud = createHud(a);
    hud.update(state());
    const settings = q(hud.el, ".hud-settings");

    q(hud.el, ".hud-pct").click();
    q(hud.el, ".hud-gear").click();
    levelChip(hud.el, "C1").click();
    expect(a.onSetLevel).toHaveBeenLastCalledWith("C1");
    expect(settings.hidden).toBe(true);

    q(hud.el, ".hud-gear").click();
    levelChip(hud.el, "").click(); // "Débutant"
    expect(a.onSetLevel).toHaveBeenLastCalledWith(null);
  });

  it("hides the level picker when the pack has no CEFR levels", () => {
    const hud = createHud(actions());
    hud.update(state({ hasLevels: false }));
    expect(q(hud.el, ".hud-levels").hidden).toBe(true);
    expect(q(hud.el, ".hud-settings-label").hidden).toBe(true);
  });

  it("invokes onHide from 'Masquer la barre'", () => {
    const a = actions();
    const hud = createHud(a);
    hud.update(state());
    q(hud.el, ".hud-hide").click();
    expect(a.onHide).toHaveBeenCalledOnce();
  });
});

describe("LinguaHud shadow wrapper", () => {
  it("mounts a single host on the page and toggles visibility", () => {
    const hud = new LinguaHud({ css: ".hud{}", actions: actions() });
    hud.mount();
    hud.mount(); // idempotent
    const host = document.documentElement.querySelector("#cymbra-lingua-hud-host") as HTMLElement;
    expect(host).not.toBeNull();
    expect(document.querySelectorAll("#cymbra-lingua-hud-host").length).toBe(1);
    expect(host.getAttribute("data-cymbra-lingua-skip")).toBe("");

    hud.update(state());
    hud.setHidden(true);
    expect(host.style.display).toBe("none");
    hud.setHidden(false);
    expect(host.style.display).toBe("");

    hud.destroy();
    expect(document.documentElement.querySelector("#cymbra-lingua-hud-host")).toBeNull();
  });
});
