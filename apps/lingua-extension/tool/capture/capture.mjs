// `yarn capture` (add-site-lingua-showcase): the site's images, share images and demo videos, and
// the Chrome Web Store / addons.mozilla.org screenshots, made from the real extension. Run on a
// developer's machine, never in CI (README.md, "Captures").
//
//   yarn capture                 every interface language (fr, en, es)
//   yarn capture --only=fr,en    some of them
//   yarn capture --headed        watch it (the window opens)
//   yarn capture --no-video      the stills alone
//   yarn capture --record-seed   re-record tool/capture/fixtures/state/<language>.json first
//
// It builds `dist-capture/` (build.mjs), loads it unpacked in Playwright's Chromium (Chrome for
// Testing) with Chrome's field trials left on, restores the committed reader state through
// Réglages › Données › Restaurer (its dates moved to the run's day, seed.mjs), writes thirty days of
// statistics, then drives the pages a reader would (drive.mjs) and encodes what it took with ffmpeg
// (images.mjs). Every run reads its clock in one time zone (plan.mjs, TIME_ZONE); the text is drawn
// with the fonts of the machine it runs on.
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright-core";
import { buildForCapture, captureExtensionId } from "./build.mjs";
import { DAILY_KEY, recentDays } from "./days.mjs";
import {
  clickControl,
  clickWord,
  controlText,
  findIn,
  frameWords,
  hasControl,
  highlightedWords,
  HOSTS,
  waitFor,
  wordPoint,
} from "./drive.mjs";
import { assembleEpub, parseBook } from "./epub.mjs";
import {
  assertStorePng,
  columnCrop,
  cropAround,
  ffmpeg,
  hasFfmpeg,
  imageArgs,
  padded,
  scaled,
  union,
  videoArgs,
} from "./images.mjs";
import {
  BOOK,
  glossProblems,
  INTERFACE_LANGUAGES,
  LEVEL,
  LOCALE,
  READER_LEVELS,
  SEED_REVIEW,
  STORE_SHOTS,
  STUDIED,
  TIME_ZONE,
  wordsFor,
} from "./plan.mjs";
import { seedForRun } from "./seed.mjs";
import { serveFixtures } from "./server.mjs";

// The tool's own clock, before any date is read: the days of statistics and the seed's dates are
// computed here, the browser reads the same zone (launch).
process.env.TZ = TIME_ZONE;

const HERE = dirname(fileURLToPath(import.meta.url));
const APP = join(HERE, "..", "..");
const REPO = join(APP, "..", "..");
const FIXTURES = join(HERE, "fixtures");
const SITE_ASSETS = join(REPO, "apps/site/src/assets/lingua");
const SITE_MEDIA = join(REPO, "apps/site/public/media/lingua");
const STORE_DIR = join(APP, "store-captures");
const VIEWPORT = { width: 1280, height: 800 };
/**
 * The fixture pages' host name, which the browser resolves to the local server: a card keeps the
 * host it was met on as its source, and `.example` is reserved, no one's site (RFC 2606).
 */
const PAGE_HOST = "reading.example";
/**
 * The captures' device pixels per CSS pixel. Three, so that the phone's close-up of the card holds
 * 1260 device pixels for a 420-pixel-wide phone at 3×; the site's whole-window sources are written
 * at twice the window's width (`SITE_WIDTH`), its crops at their full resolution.
 */
const SCALE = 3;
const SITE_WIDTH = 2 * VIEWPORT.width;
/** The site's hero: 16:9 from the window's foot, the card (opened 210 px down) and the pill kept. */
const HERO = {
  x: 0,
  y: VIEWPORT.height - (VIEWPORT.width * 9) / 16,
  width: VIEWPORT.width,
  height: (VIEWPORT.width * 9) / 16,
};

const args = process.argv.slice(2);
const flag = (name) => args.includes(`--${name}`);
const only = args
  .find((a) => a.startsWith("--only="))
  ?.slice("--only=".length)
  .split(",");
const languages = only ?? INTERFACE_LANGUAGES;
for (const l of languages)
  if (!INTERFACE_LANGUAGES.includes(l)) throw new Error(`--only: unknown interface language ${l}`);
const headed = flag("headed");
const withVideo = !flag("no-video");

/** The capture build's id, read from its manifest's key once it is built (main). */
let extensionId = "";

/** The interface catalogue's strings the tool looks for, in each language (src/i18n/<language>/). */
async function catalogue(language) {
  const load = async (module) => (await import(join(APP, "src/i18n", language, `${module}.ts`)))[module];
  return {
    card: await load("card"),
    review: await load("review"),
    settings: await load("settings"),
    colours: await load("colours"),
  };
}

function log(language, message) {
  console.log(`[capture ${language}] ${message}`);
}

/** A step's failure names the shot and the interface language. */
async function step(language, shot, run) {
  try {
    return await run();
  } catch (e) {
    throw new Error(`${shot} (${language}): ${e instanceof Error ? e.message : e}`, { cause: e });
  }
}

// — The browser —

/**
 * Playwright's Chromium on a fresh profile, the capture build loaded unpacked, field trials on
 * (D2), the clock in `TIME_ZONE`. The profile is deleted on close.
 */
async function launch({ dist, locale, recordVideoDir }) {
  const profile = mkdtempSync(join(tmpdir(), "lingua-capture-profile-"));
  const context = await chromium.launchPersistentContext(profile, {
    // The full Chromium build: the default headless binary (chrome-headless-shell) loads no extension.
    channel: "chromium",
    headless: !headed,
    locale,
    // The pages' clock, and the browser's own for the extension's worker and pages.
    timezoneId: TIME_ZONE,
    env: { ...process.env, TZ: TIME_ZONE },
    viewport: VIEWPORT,
    deviceScaleFactor: SCALE,
    args: [
      `--disable-extensions-except=${dist}`,
      `--load-extension=${dist}`,
      `--host-resolver-rules=MAP ${PAGE_HOST} 127.0.0.1`,
    ],
    // Playwright turns Chrome's field trials off, which once hid a blank EPUB reader from it.
    ignoreDefaultArgs: ["--disable-field-trial-config", "--disable-extensions"],
    ...(recordVideoDir ? { recordVideo: { dir: recordVideoDir, size: VIEWPORT } } : {}),
  });
  if (!context.serviceWorkers().length) await context.waitForEvent("serviceworker", { timeout: 20_000 });
  return {
    context,
    async close() {
      await context.close();
      rmSync(profile, { recursive: true, force: true });
    },
  };
}

const extensionUrl = (page) => `chrome-extension://${extensionId}/${page}`;

/** The install's onboarding tab, once it has opened. */
async function onboardingPage(context) {
  const found = await waitFor(() => context.pages().find((p) => p.url().endsWith("/onboarding.html")), 10_000);
  if (!found) throw new Error("the install opened no onboarding tab");
  return found;
}

/** Close the onboarding tab the install opens: a capture restores its state instead. */
async function closeOnboarding(context) {
  const page = await onboardingPage(context);
  await page.close();
}

/** Print the flags the captures run with (chrome://version), once. */
let commandLineShown = false;
async function showCommandLine(context) {
  if (commandLineShown) return;
  commandLineShown = true;
  const page = await context.newPage();
  await page.goto("chrome://version");
  console.log(`[capture] chrome://version command line:\n${(await page.textContent("#command_line"))?.trim()}`);
  await page.close();
}

// — The reader's state —

/** The extension's panel page, opened as a tab: Réglages holds Restaurer, Sauvegarder and Couleurs. */
async function openPanel(context) {
  const panel = await context.newPage();
  await panel.goto(extensionUrl("sidepanel.html"));
  await panel.locator("html:not([data-copy-pending])").waitFor();
  await showSettingsTab(panel, "data");
  return panel;
}

/**
 * Réglages, on one of its sub-tabs (« Données » holds the file row, « Apparence » the colours). On a
 * fresh profile the panel page can come up before the extension has finished starting, its Réglages
 * left empty: the page is then reloaded, up to three times.
 */
async function showSettingsTab(panel, tab) {
  const tabButton = panel.locator(`#view-settings .set-tab[data-tab="${tab}"]`);
  for (let attempt = 1; ; attempt++) {
    await panel.click('#views [data-view="settings"]');
    try {
      await tabButton.waitFor({ state: "visible", timeout: 8_000 });
      break;
    } catch (e) {
      if (attempt >= 3) throw new Error(`Réglages showed no « ${tab} » tab after ${attempt} loads`, { cause: e });
      await panel.reload();
      await panel.locator("html:not([data-copy-pending])").waitFor();
    }
  }
  await tabButton.click();
  await panel.locator(`#view-settings [role="tabpanel"][data-tab="${tab}"]`).waitFor();
}

/**
 * Restore `statePath` through Réglages › Données › Restaurer, as a reader does (D4) — its dates
 * first moved to the run's day (seed.mjs), written beside the run's raw captures.
 */
async function restore(panel, statePath, work) {
  const backup = JSON.parse(readFileSync(statePath, "utf8"));
  const forRun = join(work, `seed-${relative(FIXTURES, statePath).replace(/[/\\]/g, "-")}`);
  writeFileSync(forRun, JSON.stringify(seedForRun(backup, Date.now())));
  const note = panel.locator("#view-settings .set-file-row + .set-note");
  await panel.locator('#view-settings .set-file-row input[type="file"]').setInputFiles(forRun);
  const restored = new Set(
    await Promise.all(INTERFACE_LANGUAGES.map(async (l) => (await catalogue(l)).settings.restored)),
  );
  const said = await waitFor(async () => {
    const text = (await note.textContent())?.trim();
    return text || null;
  }, 60_000);
  if (!said || !restored.has(said)) throw new Error(`the restore said ${JSON.stringify(said)}`);
}

/** Write the thirty days ending today through the store's own message (D4). */
async function writeDays(panel, studied, native) {
  const items = { [DAILY_KEY]: recentDays({ nowMs: Date.now(), languages: studied, native }) };
  const reply = await panel.evaluate((message) => chrome.runtime.sendMessage(message), { type: "store:set", items });
  if (!reply?.ok) throw new Error(`the store refused the statistics: ${JSON.stringify(reply)}`);
}

/** A fresh browser holding the committed state of `language`'s reader. */
async function preparedBrowser({ dist, language, work, recordVideoDir }) {
  const browser = await launch({ dist, locale: LOCALE[language], recordVideoDir });
  await closeOnboarding(browser.context);
  await showCommandLine(browser.context);
  const panel = await openPanel(browser.context);
  await restore(panel, join(FIXTURES, "state", `${language}.json`), work);
  await writeDays(panel, STUDIED[language], language);
  // Drawn again in the interface language the restored state names.
  await panel.reload();
  await panel.locator(`html[lang="${language}"]:not([data-copy-pending])`).waitFor();
  await showSettingsTab(panel, "data");
  return { ...browser, panel };
}

// — Pages —

/** Open a fixture page and wait until the reader has painted it and its pill shows a share. */
async function openArticle(context, origin, studied, { query = "" } = {}) {
  const page = await context.newPage();
  await page.goto(`${origin}/${studied}.html${query}`);
  await waitForReader(page);
  return page;
}

async function waitForReader(page) {
  const painted = await waitFor(async () => (await highlightedWords(page)).length > 0, 20_000);
  if (!painted) throw new Error("the reader painted no word on the page");
  // The pill's share, then a moment for the visible-first windows to settle.
  const pill = await waitFor(() => pillShare(page), 10_000);
  if (pill === null) throw new Error("the page's pill never showed a share");
  await page.waitForTimeout(800);
}

/** The share the pill shows, as a number (`93 %` → 93), or null while it shows none. */
async function pillShare(page) {
  const text = await controlText(page, HOSTS.pill, { tag: "button" });
  const digits = text?.match(/\d+/);
  return digits ? Number(digits[0]) : null;
}

/** The first of `candidates` the reader highlights as unknown on this page. */
async function firstHighlighted(page, candidates, language, what) {
  const unknown = new Set((await highlightedWords(page)).map((w) => w.text.toLowerCase()));
  for (const word of candidates) {
    if (unknown.has(word.toLowerCase())) return word;
    log(language, `${what}: « ${word} » is not highlighted, trying the next word`);
  }
  throw new Error(`${what}: none of ${candidates.join(", ")} is highlighted`);
}

/** Scroll so that `word` sits `top` CSS pixels below the window's top edge. */
async function bringWordTo(page, word, top) {
  const point = await wordPoint(page, word, { block: "center" });
  if (!point) throw new Error(`the page has no word « ${word} »`);
  await page.evaluate((dy) => window.scrollBy({ top: dy, behavior: "instant" }), point.y - top);
  await page.waitForTimeout(400);
}

/** The box of a surface's element (CSS pixels), from its border quad. */
async function surfaceBox(page, host, matches) {
  const session = await page.context().newCDPSession(page);
  try {
    const { root } = await session.send("DOM.getDocument", { depth: -1, pierce: true });
    const node = findIn(root, host, matches);
    if (!node) return null;
    const { model } = await session.send("DOM.getBoxModel", { nodeId: node.nodeId });
    const xs = [0, 2, 4, 6].map((i) => model.border[i]);
    const ys = [1, 3, 5, 7].map((i) => model.border[i]);
    return {
      x: Math.min(...xs),
      y: Math.min(...ys),
      width: Math.max(...xs) - Math.min(...xs),
      height: Math.max(...ys) - Math.min(...ys),
    };
  } catch {
    return null; // not rendered yet
  } finally {
    await session.detach();
  }
}

/** Open the word card on `word`; returns the word's and the card's boxes. */
async function openCard(page, word) {
  await bringWordTo(page, word, 210);
  const point = await clickWord(page, word, { block: "nearest" });
  const card = await waitFor(() => surfaceBox(page, HOSTS.card, { className: "card" }), 10_000);
  if (!card || card.width === 0) throw new Error(`no word card opened on « ${word} »`);
  await page.waitForTimeout(600);
  const wordBox = await page.evaluate(({ x, y }) => {
    const range = document.caretRangeFromPoint(x, y);
    if (!range) return null;
    range.expand("word");
    const box = range.getBoundingClientRect();
    return { x: box.left, y: box.top, width: box.width, height: box.height };
  }, point);
  return { word: wordBox ?? { x: point.x - 30, y: point.y - 12, width: 60, height: 24 }, card };
}

/**
 * The open card's gloss, once the pack has answered, without its part-of-speech headings; a card
 * without a translation fails, naming the word: a capture never shows « Pas de traduction dans le
 * pack ».
 */
async function cardGloss(page, word, copy) {
  const gloss = await waitFor(async () => {
    const text = await controlText(page, HOSTS.card, { className: "gloss" }, { skip: "pos" });
    return text && text !== copy.card.waiting && text !== copy.card.translating ? text : null;
  }, 10_000);
  if (!gloss || gloss === copy.card.noGloss || gloss === copy.card.noGlossExpression) {
    throw new Error(
      `« ${word} »'s card has no translation (${JSON.stringify(gloss)}): choose another word in plan.mjs`,
    );
  }
  return gloss;
}

/** The tab id of `page`, asked from an extension page while `page` is the active tab. */
async function tabIdOf(page, panel) {
  await page.bringToFront();
  const id = await panel.evaluate(async () => (await chrome.tabs.query({ active: true }))[0]?.id ?? null);
  if (id == null) throw new Error("no active tab to open the panel on");
  return id;
}

/** Open the in-page panel on `view`, the way the Firefox background does (D6). */
async function openDrawer(page, panel, view) {
  const tabId = await tabIdOf(page, panel);
  await panel.evaluate(({ tabId, view }) => chrome.tabs.sendMessage(tabId, { type: "openDrawer", view }), {
    tabId,
    view,
  });
  await page.bringToFront();
  const opened = await waitFor(() => hasControl(page, HOSTS.panel, { className: "drawer" }), 10_000);
  if (!opened) throw new Error(`the in-page panel did not open on ${view}`);
}

async function shoot(page, path) {
  mkdirSync(dirname(path), { recursive: true });
  await page.screenshot({ path, animations: "disabled", caret: "hide" });
}

/** The box (CSS pixels) holding every element `selectors` name on the page, each required. */
async function boxOf(page, selectors) {
  const box = await page.evaluate((selectors) => {
    const rects = [];
    for (const selector of selectors) {
      const element = document.querySelector(selector);
      if (!element) return { missing: selector };
      const r = element.getBoundingClientRect();
      if (r.width > 0 && r.height > 0) rects.push(r);
    }
    const left = Math.min(...rects.map((r) => r.left));
    const top = Math.min(...rects.map((r) => r.top));
    const right = Math.max(...rects.map((r) => r.right));
    const bottom = Math.max(...rects.map((r) => r.bottom));
    return { x: left, y: top, width: right - left, height: bottom - top };
  }, selectors);
  if ("missing" in box) throw new Error(`the page has no ${box.missing}`);
  return box;
}

// — The shots (D5) —

async function shots({ dist, origin, language, work, epubs }) {
  const copy = await catalogue(language);
  const [first, second] = STUDIED[language];
  const browser = await preparedBrowser({ dist, language, work });
  const { context, panel } = browser;
  const boxes = {};
  const raw = (key) => join(work, language, `${key}.png`);
  try {
    // card — the first studied language's article, its pill, a verb's card open.
    await step(language, "card", async () => {
      const page = await openArticle(context, origin, first);
      const word = await firstHighlighted(page, wordsFor(language, first).card, language, "card");
      boxes.card = await openCard(page, word);
      await cardGloss(page, word, copy);
      await shoot(page, raw("card"));
      await page.close();
      log(language, `card: « ${word} »`);
    });

    // second — the second studied language's article, a card open, with its translation.
    await step(language, "second", async () => {
      const page = await openArticle(context, origin, second);
      const word = await firstHighlighted(page, wordsFor(language, second).second, language, "second");
      boxes.second = await openCard(page, word);
      const gloss = await cardGloss(page, word, copy);
      const problems = glossProblems(gloss);
      if (problems.length > 0) {
        throw new Error(
          `« ${word} »'s card reads « ${gloss} » (${problems.join(", ")}): choose another word in plan.mjs`,
        );
      }
      boxes.second.column = await boxOf(page, ["article"]);
      await shoot(page, raw("second"));
      await page.close();
      log(language, `second: « ${word} » — ${gloss}`);
    });

    // review — the first article with the in-page panel open on a revealed card (D6), whose gloss
    // reads as a translation.
    await step(language, "review", async () => {
      const page = await openArticle(context, origin, first);
      await openDrawer(page, panel, "review");
      await clickControl(page, HOSTS.panel, { className: "review-start" }, { label: "the review's start" });
      await clickControl(page, HOSTS.panel, { className: "review-reveal" }, { label: "« Afficher la réponse »" });
      const answered = await waitFor(
        () => hasControl(page, HOSTS.panel, { tag: "button", text: copy.review.recalled }),
        10_000,
      );
      if (!answered) throw new Error(`no « ${copy.review.recalled} » after the reveal`);
      const gloss = await waitFor(async () => {
        if (await hasControl(page, HOSTS.panel, { className: "review-no-gloss" })) return { none: true };
        const text = await controlText(page, HOSTS.panel, { className: "review-gloss" });
        return text ? { text } : null;
      }, 10_000);
      if (!gloss || gloss.none) {
        throw new Error("the first card to review has no translation: re-record the seed (yarn capture --record-seed)");
      }
      const problems = glossProblems(gloss.text);
      if (problems.length > 0) {
        throw new Error(
          `the first card to review reads « ${gloss.text} » (${problems.join(", ")}): put a word with a short ` +
            `translation fifth in plan.mjs's seedDeck and re-record the seed (yarn capture --record-seed)`,
        );
      }
      boxes.review = await surfaceBox(page, HOSTS.panel, { className: "drawer" });
      if (!boxes.review) throw new Error("the in-page panel has no box");
      await page.mouse.move(VIEWPORT.width / 3, VIEWPORT.height - 20);
      await page.waitForTimeout(800);
      await shoot(page, raw("review"));
      await page.close();
      log(language, `review: « ${gloss.text} »`);
    });

    // stats — the statistics page on the first studied language: its picker must offer it.
    await step(language, "stats", async () => {
      const page = await context.newPage();
      await page.goto(extensionUrl("stats.html"));
      await page.locator("html:not([data-copy-pending])").waitFor();
      const picker = page.locator(`.stats-languages button[data-language="${first}"]`);
      await picker.waitFor({ timeout: 10_000 });
      if (!(await picker.getAttribute("class"))?.includes("active")) await picker.click();
      await page.locator(`.stats-languages button.active[data-language="${first}"]`).waitFor({ timeout: 10_000 });
      await page.locator("#stats-root .ladder-slot > *").first().waitFor({ timeout: 10_000 });
      await page.waitForTimeout(1500);
      // The site's crop: the heading, the picker, the vocabulary and the ladder with its notes —
      // not « Renforcer un niveau » nor the marked words, which the window would cut.
      boxes.stats = await boxOf(page, [
        "main h1",
        ".stats-languages",
        "#stats-root .vocab-slot",
        "#stats-root .ladder-slot",
      ]);
      await shoot(page, raw("stats"));
      await page.close();
    });

    // reader — the EPUB reader on the first studied language's book; then the same page in
    // « E-ink contrasté ». The seed has the book's names ignored, as a reader does; where plan.mjs's
    // READER_LEVELS says so, the level is declared in Réglages first.
    await step(language, "reader", async () => {
      const level = READER_LEVELS[language];
      if (level) {
        await panel.bringToFront();
        await showSettingsTab(panel, "language");
        const chip = panel.locator(
          `#view-settings .set-levels > :first-child .level-chips button[data-lvl="${level}"]`,
        );
        await chip.click();
        await panel
          .locator(`#view-settings .set-levels > :first-child .level-chips button.active[data-lvl="${level}"]`)
          .waitFor();
        log(language, `reader: ${first} declared at ${level}`);
      }
      const page = await context.newPage();
      await openBook(page, epubs.get(BOOK[first]), BOOK[first]);
      await assertNamesIgnored(page, language, first);
      await shoot(page, raw("reader"));
      log(language, `reader: ${(await page.locator(".reading-pct").textContent())?.trim()} known`);
      await step(language, "reader-eink", async () => {
        await panel.bringToFront();
        await showSettingsTab(panel, "look");
        await panel.locator(".set-colour-presets button", { hasText: copy.colours.presetEinkMono }).click();
        await page.bringToFront();
        await page.reload();
        await waitForBook(page);
        await shoot(page, raw("reader-eink"));
        await panel.bringToFront();
        await panel.locator(".set-colour-presets button", { hasText: copy.colours.presetCymbra }).click();
      });
      await page.close();
    });
  } finally {
    await browser.close();
  }
  return boxes;
}

/** Import `bytes` in the reader's library and open it. */
async function openBook(page, bytes, name) {
  await page.goto(extensionUrl("reader.html"));
  await page.locator("html:not([data-copy-pending])").waitFor();
  await page.locator(".lib-import input[type=file]").setInputFiles({
    name: `${name}.epub`,
    mimeType: "application/epub+zip",
    buffer: Buffer.from(bytes),
  });
  await page.locator(".lib-open").first().click();
  await waitForBook(page);
}

/** Wait until the open book's section is painted: its frame holds highlighted words. */
async function waitForBook(page) {
  const painted = await waitFor(async () => (await frameWords(page)).length > 0, 30_000);
  if (!painted) throw new Error("the book's page was never painted");
  await page.waitForTimeout(1200);
}

/** The reader's capture shows no name of the plan highlighted: the seed ignored them. */
async function assertNamesIgnored(page, language, studied) {
  const names = new Set(wordsFor(language, studied).names);
  const shown = (await frameWords(page)).filter((w) => w.onScreen && names.has(w.text)).map((w) => w.text);
  if (shown.length > 0) {
    throw new Error(
      `the book highlights ${[...new Set(shown)].join(", ")}: re-record the seed (yarn capture --record-seed)`,
    );
  }
}

// — The demo video (D8) —

async function demo({ dist, origin, language, work }) {
  const copy = await catalogue(language);
  const [first] = STUDIED[language];
  const words = wordsFor(language, first);
  const videoDir = join(work, language, "video");
  const browser = await preparedBrowser({ dist, language, work, recordVideoDir: videoDir });
  const { context } = browser;
  let video;
  const marks = {};
  try {
    await step(language, "video", async () => {
      const page = await context.newPage();
      const created = Date.now();
      await page.goto(`${origin}/${first}.html?pointer`);
      marks.start = (Date.now() - created) / 1000;
      // 0–4 s: the unknown words light up, the pill shows the share known.
      await waitForReader(page);
      await page.mouse.move(VIEWPORT.width - 200, 120);
      await page.waitForTimeout(3000);
      // 4–11 s: the pointer goes to a verb; its card names the tense and gives the gloss.
      const verb = await firstHighlighted(page, words.card, language, "video verb");
      await clickWord(page, verb, { steps: 60, block: "nearest" });
      await waitFor(() => hasControl(page, HOSTS.card, { className: "card" }), 10_000);
      await page.waitForTimeout(900);
      await shoot(page, join(work, language, "poster.png"));
      await page.waitForTimeout(5000);
      // 11–15 s: « + Deck »: the word joins the deck, its highlight turns to « learning ».
      await clickControl(
        page,
        HOSTS.card,
        { tag: "button", text: copy.card.addToDeck },
        { label: copy.card.addToDeck },
      );
      await page.waitForTimeout(3000);
      // 15–21 s: « Je connais » on another word: its highlight goes, the pill's share rises — on
      // further words of the first screen until it does.
      const before = await pillShare(page);
      let after = before;
      const marked = [];
      for (const known of words.known) {
        const unknown = new Set((await highlightedWords(page)).map((w) => w.text.toLowerCase()));
        if (!unknown.has(known.toLowerCase())) continue;
        const point = await wordPoint(page, known, { block: "nearest" });
        if (!point) continue;
        await page.mouse.move(point.x, point.y, { steps: 40 });
        await page.mouse.click(point.x, point.y);
        await waitFor(() => hasControl(page, HOSTS.card, { className: "card" }), 10_000);
        await page.waitForTimeout(marked.length === 0 ? 2500 : 1200);
        await clickControl(page, HOSTS.card, { tag: "button", text: copy.card.known }, { label: copy.card.known });
        marked.push(known);
        after = await waitFor(async () => {
          const share = await pillShare(page);
          return share !== null && before !== null && share > before ? share : null;
        }, 2500);
        if (after) break;
      }
      if (!after)
        throw new Error(`the pill stayed at ${before} % after « ${copy.card.known} » on ${marked.join(", ")}`);
      log(language, `video: ${verb} to the deck; ${marked.join(", ")} known; the pill ${before} % → ${after} %`);
      await page.waitForTimeout(3000);
      // 21–30 s: the statistics — the ladder, then the days read, stopping above the closing note
      // on agent sessions.
      await page.goto(extensionUrl("stats.html"));
      await page.locator("html:not([data-copy-pending])").waitFor();
      await page.locator("#stats-root .cards .chart").first().waitFor({ timeout: 10_000 });
      await page.waitForTimeout(3500);
      await scrollToCharts(page);
      await page.waitForTimeout(3500);
      marks.end = (Date.now() - created) / 1000;
      video = page.video();
      await page.close();
    });
  } finally {
    await browser.close();
  }
  return { path: await video.path(), ...marks };
}

/**
 * Scroll the statistics page with the wheel until the charts' last row sits at the window's foot,
 * the page's closing note (agent sessions are not counted here) still below it.
 */
async function scrollToCharts(page) {
  const target = await page.evaluate(() => {
    const cards = document.querySelector("#stats-root .cards");
    const note = document.querySelector("#stats-root > p.note:last-child");
    if (!cards || !note) return null;
    const cardsBottom = cards.getBoundingClientRect().bottom + scrollY;
    const noteTop = note.getBoundingClientRect().top + scrollY;
    return Math.max(0, Math.min(cardsBottom + 16 - innerHeight, noteTop - 4 - innerHeight));
  });
  if (target === null) throw new Error("the statistics page has no charts or no closing note");
  for (let guard = 0; guard < 200; guard++) {
    const left = target - (await page.evaluate(() => scrollY));
    if (left <= 1) break;
    await page.mouse.wheel(0, Math.min(40, left));
    await page.waitForTimeout(40);
  }
  const noteShown = await page.evaluate(
    () => (document.querySelector("#stats-root > p.note:last-child")?.getBoundingClientRect().top ?? 0) < innerHeight,
  );
  if (noteShown) throw new Error("the video's last frame shows the note on agent sessions");
}

// — The seed (D4): the product records it —

async function recordSeed({ dist, origin, language, epubs }) {
  const copy = await catalogue(language);
  const [first, second] = STUDIED[language];
  const browser = await launch({ dist, locale: LOCALE[language] });
  const { context } = browser;
  try {
    await step(language, "seed", async () => {
      // The install's welcome: the second studied language, then B1 for each.
      const onboarding = await onboardingPage(context);
      await onboarding.locator("html:not([data-copy-pending])").waitFor();
      const box = onboarding.locator(`#languages-section input[data-language="${second}"]`);
      await box.waitFor();
      if (!(await box.isChecked())) await box.check();
      await waitFor(async () => (await onboarding.locator("#level-rows > div").count()) === 2, 10_000);
      const rows = onboarding.locator("#level-rows > div");
      for (let i = 0; i < (await rows.count()); i++) {
        await rows.nth(i).locator(`button[data-lvl="${LEVEL}"]`).click();
        await rows.nth(i).locator(".confirm:not([hidden])").waitFor();
      }
      await onboarding.close();

      // Words marked known and added to the deck, on each studied language's page.
      for (const [studied, share] of [
        [first, 1],
        [second, 0.5],
      ]) {
        const page = await openArticle(context, origin, studied);
        const plan = wordsFor(language, studied);
        const before = [...new Set((await highlightedWords(page)).map((w) => w.text.toLowerCase()))];
        log(language, `seed: ${studied} at ${LEVEL}, ${before.length} words highlighted: ${before.join(" ")}`);
        const take = (list) => list.slice(0, Math.ceil(list.length * share));
        for (const [list, button] of [
          [take(plan.seedKnown), copy.card.known],
          [take(plan.seedDeck), copy.card.addToDeck],
        ]) {
          for (const word of list) {
            const unknown = new Set((await highlightedWords(page)).map((w) => w.text.toLowerCase()));
            if (!unknown.has(word.toLowerCase())) {
              log(language, `seed: « ${word} » (${studied}) is not highlighted, skipped`);
              continue;
            }
            await bringWordTo(page, word, 260);
            await clickWord(page, word, { block: "nearest", steps: 4 });
            await waitFor(() => hasControl(page, HOSTS.card, { className: "card" }), 10_000);
            // A card goes to the deck only with a translation: the review capture shows its answer.
            if (button === copy.card.addToDeck) {
              await page.waitForTimeout(300);
              if (await hasControl(page, HOSTS.card, { className: "gloss", text: copy.card.noGloss })) {
                log(language, `seed: « ${word} » (${studied}) has no translation in the pack, not added`);
                await clickControl(page, HOSTS.card, { className: "close" }, { label: "the card's close" });
                await waitFor(async () => !(await hasControl(page, HOSTS.card, { className: "card" })), 5_000);
                continue;
              }
            }
            await clickControl(
              page,
              HOSTS.card,
              { tag: "button", text: button },
              { label: `${button} on « ${word} »` },
            );
            await waitFor(async () => !(await hasControl(page, HOSTS.card, { className: "card" })), 5_000);
            // A card's capture time is kept to the second, and the review takes new cards in that
            // order: one second apart, they are reviewed in the plan's order, not the alphabet's.
            if (button === copy.card.addToDeck) await page.waitForTimeout(1100);
          }
        }
        await page.close();
      }

      // A short review on the first studied language's page.
      const panel = await openPanel(context);
      const page = await openArticle(context, origin, first);
      await openDrawer(page, panel, "review");
      await clickControl(page, HOSTS.panel, { className: "review-start" }, { label: "the review's start" });
      for (const answer of SEED_REVIEW) {
        await clickControl(page, HOSTS.panel, { className: "review-reveal" }, { label: "the reveal" });
        await clickControl(
          page,
          HOSTS.panel,
          { tag: "button", text: copy.review[answer] },
          { label: copy.review[answer] },
        );
        await page.waitForTimeout(400);
      }
      await page.close();

      // The book's names, ignored in the EPUB reader as a reader does: the reader's capture then
      // highlights words, not characters.
      const reader = await context.newPage();
      await openBook(reader, epubs.get(BOOK[first]), BOOK[first]);
      await ignoreNames(reader, first, copy, language);
      await reader.close();

      // Réglages › Données › Sauvegarder: the backup file, the product's own format.
      await panel.reload();
      await panel.locator("html:not([data-copy-pending])").waitFor();
      await showSettingsTab(panel, "data");
      const download = panel.waitForEvent("download");
      await panel.locator("#view-settings .set-file-row button", { hasText: copy.settings.backup }).click();
      const file = await download;
      const out = join(FIXTURES, "state", `${language}.json`);
      await file.saveAs(out);
      // Pretty-printed, so that a re-recording reads as a diff.
      writeFileSync(out, `${JSON.stringify(JSON.parse(readFileSync(out, "utf8")), null, 2)}\n`);
      log(language, `seed recorded → ${relative(REPO, out)}`);
    });
  } finally {
    await browser.close();
  }
}

/** « Ignorer » on each of the plan's names the open book highlights on screen. */
async function ignoreNames(page, studied, copy, language) {
  const names = new Set(wordsFor(language, studied).names);
  const ignored = [];
  for (let guard = 0; guard < 20; guard++) {
    const word = (await frameWords(page)).find((w) => w.onScreen && names.has(w.text));
    if (!word) break;
    await page.mouse.move(word.x, word.y, { steps: 4 });
    await page.mouse.click(word.x, word.y);
    await clickControl(
      page,
      HOSTS.card,
      { tag: "button", text: copy.card.ignore },
      { label: `${copy.card.ignore} on « ${word.text} »` },
    );
    await waitFor(async () => !(await hasControl(page, HOSTS.card, { className: "card" })), 5_000);
    await page.waitForTimeout(600);
    ignored.push(word.text);
  }
  const capitalised = [
    ...new Set((await frameWords(page)).filter((w) => w.onScreen && /^\p{Lu}/u.test(w.text)).map((w) => w.text)),
  ];
  log(
    language,
    `seed: ignored in the book ${ignored.join(", ") || "nothing"}; capitalised words still highlighted: ${capitalised.join(", ") || "none"}`,
  );
}

// — The outputs (D7) —

function encodeStills({ language, work, boxes }) {
  const raw = (key) => join(work, language, `${key}.png`);
  const site = join(SITE_ASSETS, language);
  const store = join(STORE_DIR, language);
  mkdirSync(site, { recursive: true });
  mkdirSync(store, { recursive: true });
  const bounds = { width: VIEWPORT.width, height: VIEWPORT.height };
  const webp = (key, crop, { output = key, width } = {}) =>
    ffmpeg(
      imageArgs({
        input: raw(key),
        output: join(site, `${output}.webp`),
        format: "webp",
        ...(crop ? { crop: scaled(crop, SCALE), ...(width ? { width } : {}) } : { width: SITE_WIDTH }),
      }),
    );
  // The site's sources (the site's build makes the served sizes): the readers' whole window at 2×;
  // the hero at 2×, 16:9 from the window's foot — its top lines go, the card and the pill stay — so
  // that a 1280×800 screen shows most of it under the store buttons; tighter crops, at full
  // resolution, where a figure is shown in a column — the second page's card with the start of its
  // lines and the review panel, portrait like the hero's close-up, and the statistics down to their
  // ladder (not « Renforcer un niveau »).
  webp("card", HERO, { width: SITE_WIDTH });
  webp("reader");
  webp("reader-eink");
  const secondFocus = union(boxes.second.word, boxes.second.card);
  webp(
    "second",
    columnCrop({ column: boxes.second.column, focus: secondFocus, width: 600, aspect: 4 / 5, padding: 28, bounds }),
  );
  webp("review", cropAround({ box: boxes.review, aspect: 4 / 5, minWidth: 640, bounds }));
  webp("stats", padded(boxes.stats, 28, bounds, { bottom: 12 }));
  const focus = union(boxes.card.word, boxes.card.card);
  // The phone's close-up of the card and the words around it (D10), portrait, at full resolution:
  // 1260 device pixels and more for a phone 420 pixels wide at 3×.
  const closeUp = cropAround({ box: focus, aspect: 4 / 5, padding: 28, minWidth: 420, bounds });
  webp("card", closeUp, { output: "card-closeup" });
  // The share image, 1200×630, from the card capture (D12): a page title close above is shown whole.
  const share = cropAround({ box: focus, aspect: 1200 / 630, padding: 40, minWidth: 960, snapTop: 120, bounds });
  ffmpeg(
    imageArgs({
      input: raw("card"),
      output: join(site, "share.jpg"),
      format: "jpeg",
      crop: scaled(share, SCALE),
      width: 1200,
      height: 630,
    }),
  );
  // The stores' five, 1280×800, RGB.
  STORE_SHOTS.forEach((key, i) => {
    const output = join(store, `${i + 1}-${key}.png`);
    ffmpeg(imageArgs({ input: raw(key), output, format: "png", width: 1280, height: 800 }));
    assertStorePng(relative(APP, output), readFileSync(output), { width: 1280, height: 800 });
  });
  // The Chrome Web Store's small promo tile, 440×280.
  const tile = cropAround({ box: focus, aspect: 440 / 280, padding: 24, minWidth: 560, bounds });
  const tileOut = join(store, "promo-tile-440x280.png");
  ffmpeg(
    imageArgs({
      input: raw("card"),
      output: tileOut,
      format: "png",
      crop: scaled(tile, SCALE),
      width: 440,
      height: 280,
    }),
  );
  assertStorePng(relative(APP, tileOut), readFileSync(tileOut), { width: 440, height: 280 });
}

function encodeDemo({ language, work, video }) {
  const site = join(SITE_ASSETS, language);
  const media = join(SITE_MEDIA, language);
  mkdirSync(media, { recursive: true });
  ffmpeg(
    videoArgs({
      input: video.path,
      output: join(media, "demo.mp4"),
      start: video.start,
      duration: video.end - video.start,
    }),
  );
  ffmpeg(
    imageArgs({
      input: join(work, language, "poster.png"),
      output: join(site, "demo-poster.webp"),
      format: "webp",
      width: SITE_WIDTH,
    }),
  );
}

// — Main —

async function main() {
  if (!hasFfmpeg()) {
    console.error("ffmpeg is needed to encode the captures: brew install ffmpeg");
    process.exit(1);
  }
  console.log("[capture] building dist-capture/ and checking it against a plain Chromium build…");
  const dist = buildForCapture(APP);
  extensionId = captureExtensionId(JSON.parse(readFileSync(join(dist, "manifest.json"), "utf8")));
  const epubs = new Map();
  for (const language of new Set(Object.values(BOOK))) {
    const book = parseBook(readFileSync(join(FIXTURES, "books", `${language}.txt`), "utf8"));
    epubs.set(language, await assembleEpub(book, `urn:cymbra-lingua-capture:${language}`));
  }
  const server = await serveFixtures({ dir: join(FIXTURES, "pages") });
  const origin = `http://${PAGE_HOST}:${new URL(server.origin).port}`;
  const work = mkdtempSync(join(tmpdir(), "lingua-capture-work-"));
  try {
    for (const language of languages) {
      if (flag("record-seed")) await recordSeed({ dist, origin, language, epubs });
      const boxes = await shots({ dist, origin, language, work, epubs });
      encodeStills({ language, work, boxes });
      log(language, "stills written");
      if (withVideo) {
        const video = await demo({ dist, origin, language, work });
        encodeDemo({ language, work, video });
        log(language, "video written");
      }
    }
  } finally {
    await server.close();
    if (!flag("keep-work")) rmSync(work, { recursive: true, force: true });
    else console.log(`[capture] raw captures kept in ${work}`);
  }
  console.log(
    `[capture] done: site images in ${relative(REPO, SITE_ASSETS)}/, videos in ${relative(REPO, SITE_MEDIA)}/, ` +
      `store captures in ${relative(REPO, STORE_DIR)}/ (not committed: upload them by hand).`,
  );
}

await main();
