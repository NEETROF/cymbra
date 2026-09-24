import { beforeEach, describe, expect, it, vi } from "vitest";
import { createHud, type HudActions, type HudState, LinguaHud, restingPosition } from "@/reading/hud.ts";

beforeEach(() => {
  document.body.innerHTML = "";
  document.documentElement.querySelector("#cymbra-lingua-hud-host")?.remove();
});

function actions(over: Partial<HudActions> = {}): HudActions {
  return {
    onReview: vi.fn(),
    onStats: vi.fn(),
    onSettings: vi.fn(),
    onMoved: vi.fn(),
    ...over,
  };
}

function state(over: Partial<HudState> = {}): HudState {
  return { analysable: true, percent: 42, ...over };
}

function q(el: HTMLElement, sel: string): HTMLElement {
  const found = el.querySelector<HTMLElement>(sel);
  if (!found) throw new Error(`no element "${sel}"`);
  return found;
}

function act(el: HTMLElement, label: string): HTMLButtonElement {
  const b = [...el.querySelectorAll<HTMLButtonElement>(".hud-act")].find((x) => x.textContent === label);
  if (!b) throw new Error(`no action "${label}"`);
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
    act(hud.el, "Réviser").click();
    expect(a.onReview).toHaveBeenCalledOnce();
    expect(row.hidden).toBe(true); // collapsed after acting

    q(hud.el, ".hud-pct").click();
    act(hud.el, "Stats").click();
    expect(a.onStats).toHaveBeenCalledOnce();
  });

  it("opens the settings from the gear and collapses", () => {
    const a = actions();
    const hud = createHud(a);
    hud.update(state());
    q(hud.el, ".hud-pct").click();
    q(hud.el, ".hud-gear").click();
    expect(a.onSettings).toHaveBeenCalledOnce();
    expect(q(hud.el, ".hud-actions").hidden).toBe(true);
  });

  it("offers the level choice until a level is declared, opening the settings", () => {
    const a = actions();
    const hud = createHud(a);
    hud.update(state({ needsLevel: true }));
    const level = q(hud.el, ".hud-level");
    expect(level.hidden).toBe(false);
    expect(level.textContent).toBe("Choisis ton niveau");

    level.click();
    expect(a.onSettings).toHaveBeenCalledOnce();

    hud.update(state({ needsLevel: false }));
    expect(level.hidden).toBe(true);
  });

  it("hides the level choice by default", () => {
    const hud = createHud(actions());
    hud.update(state());
    expect(q(hud.el, ".hud-level").hidden).toBe(true);
  });

  it("collapses via the chevron", () => {
    const hud = createHud(actions());
    hud.update(state());
    q(hud.el, ".hud-pct").click();
    expect(q(hud.el, ".hud-actions").hidden).toBe(false);
    q(hud.el, ".hud-collapse").click();
    expect(q(hud.el, ".hud-actions").hidden).toBe(true);
  });
});

function box(left: number, top: number, width: number, height: number): DOMRect {
  return { left, top, width, height, right: left + width, bottom: top + height, x: left, y: top } as DOMRect;
}

/** jsdom has no PointerEvent: a MouseEvent carries every field the HUD reads (bar pointerId). */
function pointer(target: EventTarget, type: string, x: number, y: number, button = 0): void {
  target.dispatchEvent(new MouseEvent(type, { bubbles: true, composed: true, clientX: x, clientY: y, button }));
}

/** A shown HUD whose layout is stubbed: the pill's box is `at.current`, the band a right-hand column. */
function laidOut(a: HudActions = actions()) {
  const hud = createHud(a);
  document.body.append(hud.el);
  hud.update(state());
  const pill = q(hud.el, ".hud-pill");
  const at = { current: box(950, 722, 60, 34) }; // jsdom's 1024×768 viewport, bottom-right
  vi.spyOn(pill, "getBoundingClientRect").mockImplementation(() => at.current);
  vi.spyOn(hud.el, "getBoundingClientRect").mockReturnValue(box(1012, 12, 0, 744));
  return { hud, pill, pct: q(hud.el, ".hud-pct"), row: q(hud.el, ".hud-actions"), at };
}

describe("restingPosition", () => {
  const band = box(1012, 12, 0, 744);

  it("rests against the nearer side edge", () => {
    expect(restingPosition(box(100, 300, 60, 34), band, 1024).side).toBe("left");
    expect(restingPosition(box(700, 300, 60, 34), band, 1024).side).toBe("right");
  });

  it("keeps the height it was dropped at, as a fraction of the band", () => {
    expect(restingPosition(box(0, 12, 60, 34), band, 1024).y).toBe(0);
    expect(restingPosition(box(0, 722, 60, 34), band, 1024).y).toBe(1);
    expect(restingPosition(box(0, 367, 60, 34), band, 1024).y).toBe(0.5);
  });

  it("never leaves the band, even dropped past its ends", () => {
    expect(restingPosition(box(0, -40, 60, 34), band, 1024).y).toBe(0);
    expect(restingPosition(box(0, 900, 60, 34), band, 1024).y).toBe(1);
  });

  it("sits at the bottom when the band is no taller than the pill", () => {
    expect(restingPosition(box(0, 0, 60, 34), box(0, 0, 0, 20), 1024).y).toBe(1);
  });
});

describe("dragging the pill", () => {
  it("starts bottom-right and takes a remembered position without a glide", () => {
    const { hud, pill } = laidOut();
    expect(hud.el.dataset.side).toBe("right");
    expect(hud.el.style.getPropertyValue("--hud-y")).toBe("1");

    hud.setPosition({ side: "left", y: 0.5 });
    expect(hud.el.dataset.side).toBe("left");
    expect(hud.el.style.getPropertyValue("--hud-y")).toBe("0.5");
    expect(pill.style.translate).toBeFalsy();
  });

  it("rests against the side it was dropped nearer, and reports where", () => {
    const a = actions();
    const { hud, pill, pct, at } = laidOut(a);

    pointer(pct, "pointerdown", 980, 739);
    pointer(window, "pointermove", 180, 339);
    expect(pill.classList.contains("dragging")).toBe(true);
    at.current = box(150, 322, 60, 34); // where the drag left it
    pointer(window, "pointerup", 180, 339);

    const expected = { side: "left", y: Math.round(((322 - 12) / (744 - 34)) * 1000) / 1000 };
    expect(a.onMoved).toHaveBeenCalledExactlyOnceWith(expected);
    expect(hud.el.dataset.side).toBe("left");
    expect(hud.el.style.getPropertyValue("--hud-y")).toBe(String(expected.y));
    expect(pill.classList.contains("dragging")).toBe(false);
    expect(pill.style.translate).toBeFalsy(); // glided home
  });

  it("measures the resting place with transitions off, so the glide starts from the drop point", () => {
    // Measuring forces a style pass: with the transition live, dropping the drag offset would
    // start the glide there and the "resting" box would be the drop point again (seen in Chromium).
    const { pill, pct, at } = laidOut();
    const transitions: string[] = [];
    vi.spyOn(pill, "getBoundingClientRect").mockImplementation(() => {
      transitions.push(`${pill.style.transition}|${pill.style.translate || "0"}`);
      return at.current;
    });
    pointer(pct, "pointerdown", 980, 739);
    pointer(window, "pointermove", 980, 400);
    pointer(window, "pointerup", 980, 400);
    expect(transitions.at(-1)).toBe("none|0"); // the resting box: offset dropped, glide off
    expect(pill.style.transition).toBe(""); // and the glide back on afterwards
  });

  it("does not let the click that ends a drag toggle the actions, but the next tap does", () => {
    const { pct, row } = laidOut();
    pointer(pct, "pointerdown", 980, 739);
    pointer(window, "pointermove", 980, 400);
    pointer(window, "pointerup", 980, 400);
    pct.click(); // the mouse's click after the drag
    expect(row.hidden).toBe(true);

    pointer(pct, "pointerdown", 980, 400);
    pointer(window, "pointerup", 980, 400);
    pct.click();
    expect(row.hidden).toBe(false);
  });

  it("forgets a finger's drag at the next press, which produces no click to swallow", () => {
    const { pct, row } = laidOut();
    pointer(pct, "pointerdown", 980, 739);
    pointer(window, "pointermove", 980, 400);
    pointer(window, "pointerup", 980, 400);
    // No click here (a moved finger is not a tap). The next tap must still open the actions.
    pointer(pct, "pointerdown", 980, 400);
    pct.click();
    expect(row.hidden).toBe(false);
  });

  it("treats a press that barely moves as a tap", () => {
    const a = actions();
    const { pill, pct, row } = laidOut(a);
    pointer(pct, "pointerdown", 980, 739);
    pointer(window, "pointermove", 983, 743); // 5 px
    expect(pill.classList.contains("dragging")).toBe(false);
    pointer(window, "pointerup", 983, 743);
    pct.click();
    expect(row.hidden).toBe(false);
    expect(a.onMoved).not.toHaveBeenCalled();
  });

  it("follows the pointer without leaving the viewport", () => {
    const { pill, pct } = laidOut();
    pointer(pct, "pointerdown", 980, 739);
    pointer(window, "pointermove", 900, 700);
    expect(pill.style.translate).toBe("-80px -39px");
    pointer(window, "pointermove", 2000, 2000);
    expect(pill.style.translate).toBe("14px 12px"); // 1024 − (950 + 60), 768 − (722 + 34)
    pointer(window, "pointermove", -2000, -2000);
    expect(pill.style.translate).toBe("-950px -722px");
    pointer(window, "pointerup", -2000, -2000);
  });

  it("settles a cancelled drag where it was left", () => {
    const a = actions();
    const { pct, at } = laidOut(a);
    pointer(pct, "pointerdown", 980, 739);
    pointer(window, "pointermove", 980, 100);
    at.current = box(950, 12, 60, 34);
    pointer(window, "pointercancel", 980, 100);
    expect(a.onMoved).toHaveBeenCalledExactlyOnceWith({ side: "right", y: 0 });
  });

  it("stops following once released", () => {
    const { pill, pct } = laidOut();
    pointer(pct, "pointerdown", 980, 739);
    pointer(window, "pointermove", 980, 400);
    pointer(window, "pointerup", 980, 400);
    pointer(window, "pointermove", 100, 100);
    expect(pill.style.translate).toBeFalsy();
  });

  it("ignores a secondary button", () => {
    const a = actions();
    const { pill, pct } = laidOut(a);
    pointer(pct, "pointerdown", 980, 739, 2);
    pointer(window, "pointermove", 100, 100);
    pointer(window, "pointerup", 100, 100);
    expect(pill.classList.contains("dragging")).toBe(false);
    expect(a.onMoved).not.toHaveBeenCalled();
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

  it("places the pill it wraps", () => {
    // The root is closed in production; opened here only to look inside.
    const attach = HTMLElement.prototype.attachShadow;
    const spy = vi.spyOn(HTMLElement.prototype, "attachShadow").mockImplementation(function (this: HTMLElement) {
      return attach.call(this, { mode: "open" });
    });
    const hud = new LinguaHud({ css: "", actions: actions() });
    spy.mockRestore();
    hud.mount();
    hud.setPosition({ side: "left", y: 0.25 });
    const root = document.getElementById("cymbra-lingua-hud-host")?.shadowRoot;
    const el = root?.querySelector<HTMLElement>(".hud");
    expect(el?.dataset.side).toBe("left");
    expect(el?.style.getPropertyValue("--hud-y")).toBe("0.25");
    hud.destroy();
  });
});
