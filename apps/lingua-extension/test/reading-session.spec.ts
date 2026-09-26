import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { PageAnalysis } from "@/analyzer/types.ts";
import { HL_UNKNOWN } from "@/reading/highlight.ts";
import type { Gesture } from "@/reading/wordpopup.ts";
import { type ReadingHost, ReadingSession, type SessionOptions } from "@/reading/session.ts";
import { makeFakePort } from "./helpers.ts";

// The reading session reads the document it is given (add-lingua-reader D3). A book section
// is a document in its own iframe — its own window, its own realm, its own highlight
// registry — and the session attaches to one section after another. jsdom's iframes are
// separate windows, so these tests read a real second realm, the way the reader page does.

class FakeHighlight extends Set<Range> {
  constructor(...ranges: Range[]) {
    super(ranges);
  }
}

type RuntimeListener = (msg: unknown, sender: { tab?: { id: number } }, send: (r: unknown) => void) => boolean | void;

let runtimeListeners: RuntimeListener[];
let sent: unknown[];
let pageRegistry: Map<string, FakeHighlight>;

const CSS_TEXT = { tokens: "", popup: "", drawer: "", hud: "" };

/** An iframe: a section document with its own window and highlight registry. */
function section(html: string): { host: ReadingHost; registry: Map<string, FakeHighlight>; frame: HTMLIFrameElement } {
  const frame = document.createElement("iframe");
  document.body.append(frame);
  const doc = frame.contentDocument!;
  const win = frame.contentWindow! as Window & typeof globalThis;
  doc.body.innerHTML = html;
  const registry = new Map<string, FakeHighlight>();
  Object.assign(win, { CSS: { highlights: registry }, Highlight: FakeHighlight });
  return {
    frame,
    registry,
    host: {
      doc,
      win,
      paintWhole: true,
      toSurface: (box) => ({ left: box.left + 100, top: box.top + 50, bottom: box.bottom + 50 }),
      source: () => "The Hound of the Baskervilles · I: Mr. Sherlock Holmes",
      exposureSource: () => "reading:book",
    },
  };
}

/** "It was a dark night." with `dark` unknown (bytes 9–13 of block 0). */
function analysis(blocks: string[]): PageAnalysis {
  const at = blocks[0]?.indexOf("dark") ?? -1;
  return {
    analyzer_version: "1",
    analysable: at >= 0,
    tokens:
      at >= 0
        ? [{ block: 0, start: at, end: at + 4, surface: "dark", lemma: "dark", class: "Unknown", gloss: "sombre" }]
        : [],
    counted: at >= 0 ? 5 : 0,
    known: at >= 0 ? 4 : 0,
    percent: at >= 0 ? 80 : null,
  };
}

function session(options: Partial<SessionOptions> = {}) {
  const { port, calls } = makeFakePort();
  port.analyse = async (blocks) => analysis(blocks);
  const s = new ReadingSession(port, { css: CSS_TEXT, surface: "book", ...options });
  return { s, port, calls };
}

beforeEach(() => {
  runtimeListeners = [];
  sent = [];
  pageRegistry = new Map();
  // The page's own registry, which a book section must never paint into.
  vi.stubGlobal("CSS", { highlights: pageRegistry });
  vi.stubGlobal("Highlight", FakeHighlight);
  vi.stubGlobal("chrome", {
    runtime: {
      sendMessage: vi.fn(async (msg: { type?: string }) => {
        sent.push(msg);
        return msg?.type === "store:get" ? { items: {} } : undefined;
      }),
      onMessage: { addListener: (fn: RuntimeListener) => void runtimeListeners.push(fn) },
    },
    storage: {
      local: { get: async () => ({}), set: async () => {} },
      onChanged: { addListener: () => {} },
    },
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
  document.body.innerHTML = "";
  document.documentElement.querySelectorAll("[data-cymbra-lingua-skip]").forEach((el) => el.remove());
});

function indicator() {
  const states: { analysable: boolean; percent: number | null }[] = [];
  return {
    states,
    factory: () => ({
      mount: () => {},
      update: (s: { analysable: boolean; percent: number | null }) => void states.push(s),
      setHidden: () => {},
    }),
  };
}

describe("a session attached to a book section", () => {
  it("paints the section in the section's own registry, whole, and says it painted", async () => {
    const painted = vi.fn();
    const ind = indicator();
    const { s } = session({ onPainted: painted, indicator: ind.factory });
    await s.start(null);
    const { host, registry } = section("<p>It was a dark night.</p>");
    await s.attach(host);
    expect(painted).toHaveBeenCalled();
    const unknown = [...(registry.get(HL_UNKNOWN) ?? [])];
    expect(unknown.map((r) => r.toString())).toEqual(["dark"]);
    expect(unknown[0].startContainer.ownerDocument).toBe(host.doc);
    expect(pageRegistry.size).toBe(0);
    expect(ind.states.at(-1)).toMatchObject({ analysable: true, percent: 80 });
    expect(sent).toContainEqual({ type: "stats", pct: 80 });
  });

  it("detaches from one section before reading the next", async () => {
    const { s } = session();
    await s.start(null);
    const first = section("<p>It was a dark night.</p>");
    await s.attach(first.host);
    const second = section("<p>Then the dark came back.</p>");
    await s.attach(second.host);
    expect(first.registry.has(HL_UNKNOWN)).toBe(false);
    expect([...(second.registry.get(HL_UNKNOWN) ?? [])].map((r) => r.toString())).toEqual(["dark"]);
    s.detach();
    expect(second.registry.has(HL_UNKNOWN)).toBe(false);
  });

  it("answers the popup with the section's figures, as a book", async () => {
    const { s } = session();
    await s.start(null);
    const reply = async (sender = {}) => {
      let answer: unknown = "none";
      const kept = runtimeListeners[0]({ type: "getStats" }, sender, (r) => (answer = r));
      await vi.waitFor(() => expect(answer).not.toBe("none"));
      return { kept, answer };
    };
    expect((await reply()).answer).toMatchObject({ surface: "book", analysable: false });
    await s.attach(section("<p>It was a dark night.</p>").host);
    expect((await reply()).answer).toMatchObject({ surface: "book", analysable: true, percent: 80 });
  });

  it("does not answer what a tab's content script broadcasts", async () => {
    const { s } = session();
    await s.start(null);
    const answer = vi.fn();
    expect(runtimeListeners[0]({ type: "getStats" }, { tab: { id: 3 } }, answer)).toBe(false);
    expect(answer).not.toHaveBeenCalled();
  });

  it("keeps the book and chapter as the source of a card captured in it", async () => {
    const { s, calls } = session();
    await s.start(null);
    const { host } = section("<p>It was a dark night.</p>");
    await s.attach(host);
    const gesture: Gesture = {
      lemma: "dark",
      surface: "dark",
      sentence: "It was a dark night.",
      status: "learning",
      expression: false,
      gloss: "sombre",
    };
    await (s as unknown as { onGesture(g: Gesture): Promise<void> }).onGesture(gesture);
    expect(calls.addCard[0]).toMatchObject({
      lemma: "dark",
      sentence: "It was a dark night.",
      url: "The Hound of the Baskervilles · I: Mr. Sherlock Holmes",
    });
  });

  it("hands a click on nothing to the host, and stops hearing a detached section", async () => {
    const blank = vi.fn();
    const { s } = session({ onBlankClick: blank });
    await s.start(null);
    const { host } = section("<p>It was a dark night.</p>");
    await s.attach(host);
    const click = () => host.doc.body.dispatchEvent(new host.win.MouseEvent("click", { bubbles: true, clientX: 5 }));
    click();
    expect(blank).toHaveBeenCalledOnce();
    s.detach();
    click();
    expect(blank).toHaveBeenCalledOnce();
  });

  it("does not hand over a click on a link", async () => {
    const blank = vi.fn();
    const { s } = session({ onBlankClick: blank });
    await s.start(null);
    const { host } = section(`<p><a href="#n1">1</a></p>`);
    await s.attach(host);
    host.doc.querySelector("a")!.dispatchEvent(new host.win.MouseEvent("click", { bubbles: true }));
    expect(blank).not.toHaveBeenCalled();
  });

  it("says it painted, without painting, when the reader is switched off", async () => {
    vi.stubGlobal("chrome", {
      ...(globalThis as unknown as { chrome: object }).chrome,
      storage: {
        local: { get: async () => ({ "cymbra-lingua-enabled": false }), set: async () => {} },
        onChanged: { addListener: () => {} },
      },
    });
    const painted = vi.fn();
    const { s } = session({ onPainted: painted });
    await s.start(null);
    const { host, registry } = section("<p>It was a dark night.</p>");
    await s.attach(host);
    expect(painted).toHaveBeenCalled();
    expect(registry.has(HL_UNKNOWN)).toBe(false);
    expect(sent).toContainEqual({ type: "stats", pct: null, disabled: true });
  });
});

// Content that changes after the first paint (fix-lingua-dynamic-rescan): what reaches the page is
// read, and a change that leaves every block's text as it was costs no analysis. The section's own
// MutationObserver runs for real; jsdom has no IntersectionObserver, so every change counts as on
// screen.
describe("a page that changes after its first paint", () => {
  const DEBOUNCE = 250;
  const pause = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

  async function painted(html: string) {
    const { s, port } = session();
    const analysed: string[][] = [];
    port.analyse = async (blocks) => {
      analysed.push(blocks);
      return analysis(blocks);
    };
    await s.start(null);
    const sec = section(html);
    await s.attach(sec.host);
    return { s, analysed, ...sec };
  }

  it("does not analyse the page again when only a clock of digits ticks", async () => {
    const { s, analysed, host } = await painted(`<p>It was a dark night.</p><div id="clock"><span>0:14</span></div>`);
    expect(analysed).toHaveLength(1);

    (host.doc.querySelector("#clock span")!.firstChild as Text).data = "0:15";
    await pause(DEBOUNCE + 150);

    expect(analysed).toHaveLength(1);
    s.detach();
  });

  it("reads a sentence appended to a paragraph already painted", async () => {
    const { s, analysed, host, registry } = await painted(`<p id="p">It was a night.</p>`);
    expect([...(registry.get(HL_UNKNOWN) ?? [])]).toEqual([]); // nothing unknown yet

    host.doc.getElementById("p")!.append(" Then a dark wind rose.");
    await vi.waitFor(() => expect(analysed).toHaveLength(2), { timeout: 2000 });

    expect(analysed[1]).toEqual(["It was a night. Then a dark wind rose."]);
    await vi.waitFor(() => expect([...(registry.get(HL_UNKNOWN) ?? [])].map((r) => r.toString())).toEqual(["dark"]));
    s.detach();
  });

  it("never analyses a video player's caption line", async () => {
    const { s, analysed, host } = await painted(
      `<div class="ytp-caption-window-container"><span id="cue">a dark line</span></div><p>It was a night.</p>`,
    );
    expect(analysed[0]).toEqual(["It was a night."]);

    (host.doc.getElementById("cue")!.firstChild as Text).data = "another dark line";
    await pause(DEBOUNCE + 150);

    expect(analysed).toHaveLength(1);
    s.detach();
  });
});

// Safari removes a phrase's selection once a finger lifts from it, so its callout stops covering
// the expression card — only where that card is shown: the reader on, the page analysed.
describe("a phrase a finger lifts from", () => {
  /** Select `text` in the section, then lift a finger from it. */
  function liftFrom(host: ReadingHost, text: string): Selection {
    const node = host.doc.querySelector("p")!.firstChild!;
    const at = node.textContent!.indexOf(text);
    const range = host.doc.createRange();
    range.setStart(node, at);
    range.setEnd(node, at + text.length);
    const sel = host.win.getSelection()!;
    sel.removeAllRanges();
    sel.addRange(range);
    host.doc.body.dispatchEvent(new host.win.Event("touchend", { bubbles: true }));
    return sel;
  }

  it("loses its selection where the reader is at work: switched on, the page analysed", async () => {
    const { s } = session({ dropPhraseOnLift: true });
    await s.start(null);
    const { host } = section("<p>It was a dark night.</p>");
    await s.attach(host);
    expect(liftFrom(host, "a dark").isCollapsed).toBe(true);
  });

  it("keeps it on a page the reader could not analyse", async () => {
    const { s } = session({ dropPhraseOnLift: true });
    await s.start(null);
    const { host } = section("<p>Il faisait nuit noire.</p>");
    await s.attach(host);
    expect(liftFrom(host, "nuit noire").isCollapsed).toBe(false);
  });

  it("keeps it while the reader is switched off", async () => {
    vi.stubGlobal("chrome", {
      ...(globalThis as unknown as { chrome: object }).chrome,
      storage: {
        local: { get: async () => ({ "cymbra-lingua-enabled": false }), set: async () => {} },
        onChanged: { addListener: () => {} },
      },
    });
    const { s } = session({ dropPhraseOnLift: true });
    await s.start(null);
    const { host } = section("<p>It was a dark night.</p>");
    await s.attach(host);
    expect(liftFrom(host, "a dark").isCollapsed).toBe(false);
  });

  it("keeps it outside Safari's build", async () => {
    const { s } = session();
    await s.start(null);
    const { host } = section("<p>It was a dark night.</p>");
    await s.attach(host);
    expect(liftFrom(host, "a dark").isCollapsed).toBe(false);
  });
});

// A selection that begins asks for the engine (add-lingua-translation-android D2): the cold start
// runs while the handles move. Only with a translator — no model ready, nothing is sent.
describe("a selection that begins", () => {
  function select(host: ReadingHost, text: string): void {
    const node = host.doc.querySelector("p")!.firstChild!;
    const at = node.textContent!.indexOf(text);
    const range = host.doc.createRange();
    range.setStart(node, at);
    range.setEnd(node, at + text.length);
    const sel = host.win.getSelection()!;
    sel.removeAllRanges();
    sel.addRange(range);
    host.doc.dispatchEvent(new host.win.Event("selectionchange"));
  }
  function collapse(host: ReadingHost): void {
    host.win.getSelection()!.removeAllRanges();
    host.doc.dispatchEvent(new host.win.Event("selectionchange"));
  }

  it("warms the translator once per selection, before the card", async () => {
    const warm = vi.fn();
    const translate = vi.fn(async () => ({ kind: "unavailable" as const }));
    const { s } = session({ translator: () => ({ translate, warm }) });
    await s.start(null);
    const { host } = section("<p>It was a dark night.</p>");
    await s.attach(host);
    select(host, "a dark");
    select(host, "a dark night"); // a handle moved
    expect(warm).toHaveBeenCalledOnce();
    expect(translate).not.toHaveBeenCalled(); // nothing settled yet
    collapse(host);
    select(host, "night");
    expect(warm).toHaveBeenCalledTimes(2);
    s.detach();
  });

  it("sends nothing without a translator: extended translation off, or its model not ready", async () => {
    let ready = false;
    const warm = vi.fn();
    const { s } = session({ translator: () => (ready ? { translate: vi.fn(), warm } : null) });
    await s.start(null);
    const { host } = section("<p>It was a dark night.</p>");
    await s.attach(host);
    select(host, "a dark");
    expect(warm).not.toHaveBeenCalled();
    collapse(host);
    ready = true;
    select(host, "a dark");
    expect(warm).toHaveBeenCalledOnce();
    s.detach();
  });
});
