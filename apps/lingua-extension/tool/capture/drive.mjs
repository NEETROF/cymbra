// Driving the reading surfaces as a reader's pointer would (add-site-lingua-showcase D2). The word
// card, the pill and the in-page panel each live in a CLOSED shadow root, which Playwright's locators
// do not enter, and the highlights are painted with `CSS.highlights`, so a highlighted word is not an
// element. A word is found from the page's own text (a Range and its client rects); a control of a
// surface through the DevTools protocol — `DOM.getDocument` with `pierce: true` reaches closed roots
// as DevTools' own Elements panel does — and `DOM.getBoxModel`. Then `page.mouse` clicks at that
// point. Nothing is clicked by a fixed coordinate: a layout change fails loudly on the missing
// control rather than clicking the wrong one.

/** The surfaces' shadow hosts (src/reading/wordpopup.ts, hud.ts, drawer.ts). */
export const HOSTS = {
  card: "cymbra-lingua-host",
  pill: "cymbra-lingua-hud-host",
  panel: "cymbra-lingua-drawer-host",
};

/**
 * The text of a DOM.Node from `DOM.getDocument`, its descendants' text nodes concatenated — but
 * those under an element carrying the class `skip` (a card's part-of-speech headings, say).
 */
export function nodeText(node, skip) {
  if (node.nodeType === 3) return node.nodeValue ?? "";
  if (skip && node.nodeType === 1 && (attributesOf(node).class ?? "").split(/\s+/).includes(skip)) return "";
  let text = "";
  for (const child of node.children ?? []) text += nodeText(child, skip);
  for (const root of node.shadowRoots ?? []) text += nodeText(root, skip);
  return text;
}

/** A node's attributes as an object (`DOM.Node.attributes` is a flat name, value list). */
export function attributesOf(node) {
  const out = {};
  const list = node.attributes ?? [];
  for (let i = 0; i + 1 < list.length; i += 2) out[list[i]] = list[i + 1];
  return out;
}

/** Every node under `node`, shadow roots and template contents included, depth first. */
export function* walk(node) {
  yield node;
  for (const root of node.shadowRoots ?? []) yield* walk(root);
  for (const child of node.children ?? []) yield* walk(child);
  if (node.contentDocument) yield* walk(node.contentDocument);
}

/**
 * The first element under the host whose id is `host` that `matches` accepts — `{ tag, text,
 * className, attribute }`, each optional: the tag name (lowercase), the trimmed text exactly, a class
 * it carries, an attribute it holds with this value.
 */
export function findIn(documentNode, host, matches) {
  const hostNode = [...walk(documentNode)].find((n) => attributesOf(n).id === host);
  if (!hostNode) return null;
  for (const node of walk(hostNode)) {
    if (node.nodeType !== 1) continue;
    if (matches.tag && node.localName !== matches.tag) continue;
    const attrs = attributesOf(node);
    if (matches.className && !(attrs.class ?? "").split(/\s+/).includes(matches.className)) continue;
    if (matches.attribute && attrs[matches.attribute[0]] !== matches.attribute[1]) continue;
    if (matches.text !== undefined && nodeText(node).trim() !== matches.text) continue;
    return node;
  }
  return null;
}

/** The centre of a `DOM.getBoxModel` border quad. */
export function quadCentre(quad) {
  const xs = [quad[0], quad[2], quad[4], quad[6]];
  const ys = [quad[1], quad[3], quad[5], quad[7]];
  return { x: (Math.min(...xs) + Math.max(...xs)) / 2, y: (Math.min(...ys) + Math.max(...ys)) / 2 };
}

/** A DevTools session on `page`, opened once. */
const sessions = new WeakMap();
async function cdp(page) {
  let session = sessions.get(page);
  if (!session) {
    session = await page.context().newCDPSession(page);
    sessions.set(page, session);
  }
  return session;
}

/**
 * Where a control of a surface is, in the page's CSS pixels, or null while it is absent.
 * @param {import("playwright-core").Page} page
 */
export async function locateControl(page, host, matches) {
  const session = await cdp(page);
  const { root } = await session.send("DOM.getDocument", { depth: -1, pierce: true });
  const node = findIn(root, host, matches);
  if (!node) return null;
  try {
    const { model } = await session.send("DOM.getBoxModel", { nodeId: node.nodeId });
    if (model.width === 0 || model.height === 0) return null;
    return quadCentre(model.border);
  } catch {
    return null; // not rendered (hidden)
  }
}

/** Wait for a control, then click it; fails naming what it looked for. */
export async function clickControl(page, host, matches, { timeout = 10_000, label } = {}) {
  const point = await waitFor(() => locateControl(page, host, matches), timeout);
  if (!point) throw new Error(`no ${label ?? JSON.stringify(matches)} in #${host}`);
  await page.mouse.move(point.x, point.y, { steps: 12 });
  await page.mouse.click(point.x, point.y);
}

/** Whether a surface shows a control matching `matches`. */
export async function hasControl(page, host, matches) {
  return (await locateControl(page, host, matches)) !== null;
}

/**
 * The trimmed text of a surface's element matching `matches`, or null while it is absent or not
 * rendered: what the card's gloss or the pill says, read through the closed root — without the
 * text under elements of the class `skip`, when given.
 * @param {import("playwright-core").Page} page
 */
export async function controlText(page, host, matches, { skip } = {}) {
  if (!(await hasControl(page, host, matches))) return null;
  const session = await cdp(page);
  const { root } = await session.send("DOM.getDocument", { depth: -1, pierce: true });
  const node = findIn(root, host, matches);
  return node ? nodeText(node, skip).replace(/\s+/g, " ").trim() : null;
}

/**
 * The words a frame's reader paints with `highlight`, with the centre of each word's first box in
 * the PAGE's CSS pixels (the frame's own offset added) and whether that box is inside the window:
 * the EPUB reader's book lives in frames of the reader page.
 * @param {import("playwright-core").Page} page
 */
export async function frameWords(page, highlight = "cymbra-lingua-unknown") {
  const viewport = page.viewportSize() ?? { width: 0, height: 0 };
  const words = [];
  for (const frame of page.frames()) {
    if (frame === page.mainFrame()) continue;
    const element = await frame.frameElement().catch(() => null);
    const box = element ? await element.boundingBox() : null;
    if (!box) continue;
    const found = await frame
      .evaluate((name) => {
        const set = CSS.highlights.get(name);
        if (!set) return [];
        const out = [];
        for (const range of set) {
          const r = document.createRange();
          r.setStart(range.startContainer, range.startOffset);
          r.setEnd(range.endContainer, range.endOffset);
          const rect = r.getClientRects()[0];
          if (rect) out.push({ text: r.toString(), x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 });
        }
        return out;
      }, highlight)
      .catch(() => []);
    for (const w of found) {
      const x = box.x + w.x;
      const y = box.y + w.y;
      words.push({ text: w.text, x, y, onScreen: x > 0 && x < viewport.width && y > 0 && y < viewport.height });
    }
  }
  return words;
}

/** Poll `probe` until it returns something truthy or `timeout` passes; returns the last answer. */
export async function waitFor(probe, timeout = 10_000, every = 150) {
  const end = Date.now() + timeout;
  for (;;) {
    const got = await probe();
    if (got || Date.now() > end) return got;
    await new Promise((r) => setTimeout(r, every));
  }
}

/**
 * The words the reader paints with `highlight` (`cymbra-lingua-unknown` or `-learning`), in the
 * document's order, with the centre of each word's first box and whether it is on screen.
 */
export async function highlightedWords(page, highlight = "cymbra-lingua-unknown") {
  return page.evaluate((name) => {
    const set = CSS.highlights.get(name);
    if (!set) return [];
    const words = [];
    for (const range of set) {
      // A live Range for its boxes, whether the registry holds Ranges or StaticRanges.
      const r = document.createRange();
      r.setStart(range.startContainer, range.startOffset);
      r.setEnd(range.endContainer, range.endOffset);
      const box = r.getClientRects()[0];
      if (!box) continue;
      words.push({
        text: r.toString(),
        x: box.left + box.width / 2,
        y: box.top + box.height / 2,
        top: box.top + scrollY,
        onScreen: box.top >= 0 && box.bottom <= innerHeight,
      });
    }
    return words.sort((a, b) => a.top - b.top || a.x - b.x);
  }, highlight);
}

/**
 * The centre of the `occurrence`-th whole-word `word` in the page's text, scrolled into view first
 * (`block`: where in the window). Null when the page has no such word.
 */
export async function wordPoint(page, word, { occurrence = 0, block = "center" } = {}) {
  return page.evaluate(
    ({ word, occurrence, block }) => {
      const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
      const pattern = new RegExp(
        `(^|[^\\p{L}\\p{N}])(${word.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")})(?![\\p{L}\\p{N}])`,
        "giu",
      );
      let seen = 0;
      for (let node = walker.nextNode(); node; node = walker.nextNode()) {
        for (const m of node.nodeValue.matchAll(pattern)) {
          if (seen++ !== occurrence) continue;
          const start = m.index + m[1].length;
          const range = document.createRange();
          range.setStart(node, start);
          range.setEnd(node, start + m[2].length);
          node.parentElement.scrollIntoView({ block, behavior: "instant" });
          const box = range.getClientRects()[0];
          return box ? { x: box.left + box.width / 2, y: box.top + box.height / 2 } : null;
        }
      }
      return null;
    },
    { word, occurrence, block },
  );
}

/** Click a word of the page as a reader would, moving the pointer there first. */
export async function clickWord(page, word, options = {}) {
  const point = await wordPoint(page, word, options);
  if (!point) throw new Error(`the page has no word « ${word} »`);
  await page.mouse.move(point.x, point.y, { steps: options.steps ?? 12 });
  await page.mouse.click(point.x, point.y);
  return point;
}
