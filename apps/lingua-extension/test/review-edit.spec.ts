import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { LinguaPort } from "@/analyzer/port.ts";
import { checkEdit, editedForm, FORM_MAX_UNITS, GLOSS_MAX_UNITS, refusalOf } from "@/review/edit.ts";
import { mountReview } from "@/review/review-page.ts";
import type { ReviewView } from "@/review/session.ts";
import { type ReviewActions, type ReviewEditActions, renderReview } from "@/review/view.ts";
import { type AsyncStorageArea, ROOT_KEY } from "@/state/storage.ts";
import { type FakeCard, makeFakePort } from "./helpers.ts";

// Editing the card under review (add-lingua-card-edit): the page's checks (D3), the form as the
// render paints it (D1, D2), and the page around it — the reveal it implies, the save, a merge said
// once, a card leaving the session (D6, D9, D10).

const packs = vi.hoisted(() => ({ shipped: ["en-fr"] }));
vi.mock("@/analyzer/pairs.ts", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/analyzer/pairs.ts")>();
  return {
    ...actual,
    readingLanguage: (port: Parameters<typeof actual.readingLanguage>[0]) =>
      actual.readingLanguage(port, packs.shipped),
    acceptedLanguages: (port: Parameters<typeof actual.acceptedLanguages>[0]) =>
      actual.acceptedLanguages(port, packs.shipped),
  };
});

const SENTENCE = "The bed is colloquially referred to as a tester.";
const OLD = "colloquially referred to as";
const NEW = "referred to as";

describe("the page's checks, the engine's rule (D3)", () => {
  it("trims, collapses and strips a form as capture does, and bounds both fields", () => {
    const cases: [string, string, ReturnType<typeof checkEdit>][] = [
      ["four-poster bed,", "", null],
      ["  referred \t to as ", "dénommé", null],
      [" , — ", "", refusalOf("empty")],
      ["a".repeat(FORM_MAX_UNITS), "", null],
      ["a".repeat(FORM_MAX_UNITS + 1), "", refusalOf("form_too_long")],
      ["bed", "é".repeat(GLOSS_MAX_UNITS), null],
      ["bed", "é".repeat(GLOSS_MAX_UNITS + 1), refusalOf("gloss_too_long")],
    ];
    for (const [form, gloss, refusal] of cases) expect(checkEdit(form, gloss), form).toEqual(refusal);
    expect(editedForm("« four-poster  bed, »")).toBe("four-poster bed");
    expect(refusalOf("gloss_too_long").field).toBe("gloss");
    expect(refusalOf("empty").field).toBe("form");
  });
});

describe("renderReview — the edit link and the form (D1, D2)", () => {
  let root: HTMLElement;
  let edit: { [K in keyof ReviewEditActions]: ReturnType<typeof vi.fn> };
  let actions: ReviewActions;

  beforeEach(() => {
    document.body.innerHTML = "<div id='r'></div>";
    root = document.getElementById("r")!;
    edit = { open: vi.fn(), save: vi.fn(), cancel: vi.fn() };
    actions = {
      start: vi.fn(),
      reveal: vi.fn(),
      grade: vi.fn(),
      markKnown: vi.fn(),
      ignore: vi.fn(),
      edit: edit as unknown as ReviewEditActions,
    };
  });

  const view = (over: Partial<NonNullable<ReviewView["card"]>> = {}): ReviewView => ({
    phase: "reviewing",
    card: {
      headword: OLD,
      surface: OLD,
      sentence: SENTENCE,
      gloss: "familièrement appelé",
      revealed: true,
      remaining: 2,
      language: "en",
      ...over,
    },
    summary: null,
    moreDue: false,
  });
  const button = (label: string): HTMLButtonElement => {
    const found = [...root.querySelectorAll("button")].find((b) => b.textContent === label);
    if (!found) throw new Error(`no button "${label}"`);
    return found;
  };
  const input = (name: "form" | "gloss"): HTMLInputElement =>
    root.querySelector<HTMLInputElement>(`.review-edit-${name}`)!;
  const key = (target: EventTarget, k: string): KeyboardEvent => {
    const e = new KeyboardEvent("keydown", { key: k, bubbles: true, cancelable: true });
    target.dispatchEvent(e);
    return e;
  };

  it("offers « Modifier la carte » beside the two links, before and after the reveal, and only with a host's edit", () => {
    for (const revealed of [false, true]) {
      renderReview(root, view({ revealed }), actions);
      const links = [...root.querySelectorAll(".review-links button")].map((b) => b.textContent);
      expect(links).toEqual(["Je connais", "Ne plus me le montrer", "Modifier la carte"]);
    }
    button("Modifier la carte").click();
    expect(edit.open).toHaveBeenCalledOnce();
    renderReview(root, view(), { ...actions, edit: undefined });
    expect(root.querySelector(".review-edit-card")).toBeNull();
  });

  it("opens the form in place of the answer and the answers, the sentence in view, each field labelled", async () => {
    renderReview(root, view(), actions, { editing: {}, nativeLanguage: "fr" });
    expect(root.querySelector(".review-sentence")?.textContent).toBe(SENTENCE);
    expect(root.querySelector(".review-answer")).toBeNull();
    expect(root.querySelector(".review-bar")).toBeNull();
    const labels = [...root.querySelectorAll("label")].map((l) => [l.textContent, l.htmlFor]);
    expect(labels).toEqual([
      ["Mot ou expression", input("form").id],
      ["Traduction", input("gloss").id],
    ]);
    const word = input("form");
    expect([word.value, word.lang, word.maxLength, word.spellcheck]).toEqual([OLD, "en", 120, false]);
    expect([word.getAttribute("autocapitalize"), word.getAttribute("autocorrect"), word.enterKeyHint]).toEqual([
      "none",
      "off",
      "done",
    ]);
    expect([input("gloss").value, input("gloss").lang, input("gloss").maxLength]).toEqual([
      "familièrement appelé",
      "fr",
      200,
    ]);
    expect([button("Enregistrer").type, button("Annuler").type]).toEqual(["submit", "button"]);
    await Promise.resolve();
    expect(document.activeElement).toBe(word);
  });

  it("never fills the translation with a machine translation", () => {
    renderReview(root, view({ gloss: null }), actions, {
      editing: {},
      translation: { state: "translated", translation: { sentence: "Le lit…", marks: [] }, alone: false },
    });
    expect(input("gloss").value).toBe("");
  });

  it("saves the fields as typed, and refuses a form the engine would refuse, tied to its field", () => {
    renderReview(root, view(), actions, { editing: {} });
    const form = root.querySelector("form")!;
    input("form").value = NEW;
    form.requestSubmit();
    expect(edit.save).toHaveBeenCalledWith(NEW, "familièrement appelé");

    edit.save.mockClear();
    const cases: [string, string, "form" | "gloss", string][] = [
      [" , ", "", "form", "Écrivez un mot ou une expression."],
      ["a".repeat(121), "", "form", "120 caractères au plus."],
      [NEW, "é".repeat(201), "gloss", "200 caractères au plus."],
    ];
    for (const [typedForm, typedGloss, field, message] of cases) {
      input("form").value = typedForm;
      input("gloss").value = typedGloss;
      form.requestSubmit();
      const refused = input(field);
      const error = root.querySelector<HTMLElement>(`#${refused.getAttribute("aria-describedby")}`)!;
      expect([refused.getAttribute("aria-invalid"), error.textContent, error.hidden]).toEqual(["true", message, false]);
      const other = input(field === "form" ? "gloss" : "form");
      expect(other.hasAttribute("aria-invalid")).toBe(false);
      expect(document.activeElement).toBe(refused);
    }
    expect(edit.save).not.toHaveBeenCalled();
  });

  it("shows again what was typed and the engine's refusal", () => {
    renderReview(root, view(), actions, {
      editing: { form: "x".repeat(130), gloss: "dit", refusal: refusalOf("form_too_long") },
    });
    expect([input("form").value.length, input("gloss").value]).toEqual([130, "dit"]);
    expect(input("form").getAttribute("aria-invalid")).toBe("true");
    expect(root.querySelector(".review-edit-error:not([hidden])")?.textContent).toBe("120 caractères au plus.");
  });

  it("cancels on Escape and on « Annuler », and the review's keys never act from the form", () => {
    renderReview(root, view(), actions, { editing: {} });
    for (const k of [" ", "Enter", "ArrowLeft", "ArrowRight"]) {
      expect(key(input("form"), k).defaultPrevented).toBe(false);
      key(button("Enregistrer"), k);
    }
    expect(actions.reveal).not.toHaveBeenCalled();
    expect(actions.grade).not.toHaveBeenCalled();
    expect(key(input("gloss"), "Escape").defaultPrevented).toBe(true);
    button("Annuler").click();
    expect(edit.cancel).toHaveBeenCalledTimes(2);
  });

  it("says the merge once, at the top of the answer", () => {
    renderReview(root, view({ headword: NEW }), actions, { merged: NEW });
    expect(root.querySelector(".review-answer .review-merged")?.textContent).toBe(
      "Réunie avec la carte « referred to as » déjà dans vos révisions.",
    );
    renderReview(root, view({ headword: NEW }), actions, { merged: NEW, interfaceLanguage: "en" });
    expect(root.querySelector(".review-merged")?.textContent).toBe(
      "Merged with the card “referred to as” already in your reviews.",
    );
  });
});

// — The page —

type Area = AsyncStorageArea & { raw: Record<string, unknown> };

function fakeArea(): Area {
  const raw: Record<string, unknown> = {};
  return {
    raw,
    async get(keys) {
      const list = keys == null ? Object.keys(raw) : Array.isArray(keys) ? keys : [keys];
      return Object.fromEntries(list.filter((k) => k in raw).map((k) => [k, raw[k]]));
    },
    async set(items) {
      Object.assign(raw, items);
    },
  };
}

beforeEach(() => {
  packs.shipped = ["en-fr"];
  vi.stubGlobal("chrome", { storage: { onChanged: { addListener: () => {} } } });
});

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const settle = async (): Promise<void> => {
  for (let i = 0; i < 3; i++) await new Promise((resolve) => setTimeout(resolve, 0));
};

/** The page, its first card under review — revealed when asked. */
async function reviewing(deck: FakeCard[], revealed = true, over: Partial<LinguaPort> = {}) {
  const fake = makeFakePort(deck);
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const area = fakeArea();
  const page = mountReview(container, { ...fake.port, ...over }, area, { now: () => 1_000, prefs: fakeArea() });
  await page.refresh();
  const click = async (label: string): Promise<void> => {
    const found = [...container.querySelectorAll("button")].find((b) => b.textContent === label);
    if (!found) throw new Error(`no button "${label}"`);
    found.click();
    await settle();
  };
  await click("Réviser");
  if (revealed) await click("Afficher la réponse");
  const save = async (form: string, gloss?: string): Promise<void> => {
    container.querySelector<HTMLInputElement>(".review-edit-form")!.value = form;
    if (gloss !== undefined) container.querySelector<HTMLInputElement>(".review-edit-gloss")!.value = gloss;
    container.querySelector("form")!.requestSubmit();
    await settle();
  };
  const text = (selector: string): string => container.querySelector(selector)?.textContent ?? "";
  return { ...fake, container, area, click, save, text };
}

const card = (headword: string, gloss: string | null = null): FakeCard => ({
  headword,
  surface: headword,
  sentence: SENTENCE,
  gloss,
});

describe("Révision — editing the card under review (D1, D10)", () => {
  it("reveals a hidden card and opens the form in the same repaint", async () => {
    const m = await reviewing([card(OLD, "familièrement appelé")], false);
    await m.click("Modifier la carte");
    expect(m.calls.reveals).toBe(1);
    expect(m.container.querySelector<HTMLInputElement>(".review-edit-gloss")?.value).toBe("familièrement appelé");
  });

  it("A free selection trimmed: renamed, marked in the same sentence, revealed, saved, no answer counted", async () => {
    const m = await reviewing([card(OLD, "familièrement appelé"), card("tester")]);
    await m.click("Modifier la carte");
    await m.save(NEW);
    expect(m.calls.edits).toEqual([{ form: NEW, gloss: "familièrement appelé" }]);
    expect(m.text(".review-word")).toBe(NEW);
    expect(m.text(".review-sentence")).toBe(SENTENCE);
    expect(m.container.querySelector(".review-edit")).toBeNull();
    expect(m.container.querySelector(".review-hit")).not.toBeNull(); // revealed: the reader answers as usual
    expect(m.calls.grades).toEqual([]);
    expect(m.area.raw).toHaveProperty(ROOT_KEY); // the backup was saved
    expect(m.text(".remaining")).toBe("2 carte(s) à revoir");
  });

  it("A form the sentence does not hold: the form above the unchanged sentence", async () => {
    const m = await reviewing([card(OLD)]);
    await m.click("Modifier la carte");
    await m.save("termed", "dit");
    expect(m.text(".review-headword")).toBe("termed");
    expect(m.text(".review-sentence")).toBe(SENTENCE);
    expect(m.text(".review-gloss")).toBe("dit");
  });

  it("Cancelling: the card as it was, nothing saved", async () => {
    const m = await reviewing([card(OLD, "familièrement appelé")]);
    await m.click("Modifier la carte");
    m.container.querySelector<HTMLInputElement>(".review-edit-form")!.value = NEW;
    await m.click("Annuler");
    expect(m.calls.edits).toEqual([]);
    expect(m.area.raw).not.toHaveProperty(ROOT_KEY);
    expect(m.text(".review-word")).toBe("colloquially referred to as");
  });

  it("A save that changes nothing writes nothing", async () => {
    const m = await reviewing([card(OLD, "familièrement appelé")]);
    await m.click("Modifier la carte");
    await m.save(OLD);
    expect(m.calls.edits).toHaveLength(1);
    expect(m.area.raw).not.toHaveProperty(ROOT_KEY);
    expect(m.container.querySelector(".review-edit")).toBeNull();
  });

  it("Two cards for one expression: the merge said once, gone with the next answer", async () => {
    const m = await reviewing([card(OLD), card("tester"), card(NEW)]);
    await m.click("Modifier la carte");
    await m.save("Referred to as");
    expect(m.text(".review-merged")).toBe("Réunie avec la carte « referred to as » déjà dans vos révisions.");
    expect(m.text(".remaining")).toBe("2 carte(s) à revoir"); // the other card does not come twice
    await m.click("Su");
    expect(m.container.querySelector(".review-merged")).toBeNull();
    expect(m.text(".review-word")).toBe("tester");
  });

  it("A form already known: the card leaves and the review moves to the next", async () => {
    const m = await reviewing([card(OLD), card("tester")]);
    m.calls.knownForms.push(NEW);
    await m.click("Modifier la carte");
    await m.save(NEW);
    expect(m.text(".review-word")).toBe("tester");
    expect(m.container.querySelector(".review-reveal")).not.toBeNull();
  });

  it("A refusal of the engine's own reopens the form with what was typed and the reason", async () => {
    const m = await reviewing([card(OLD)], true, {
      reviewEdit: async () => ({ ok: false, error: "form_too_long" }),
    });
    await m.click("Modifier la carte");
    await m.save(NEW, "dit");
    expect(m.container.querySelector<HTMLInputElement>(".review-edit-form")?.value).toBe(NEW);
    expect(m.container.querySelector<HTMLInputElement>(".review-edit-gloss")?.value).toBe("dit");
    expect(m.text(".review-edit-error:not([hidden])")).toBe("120 caractères au plus.");
    expect(m.area.raw).not.toHaveProperty(ROOT_KEY);
  });
});
