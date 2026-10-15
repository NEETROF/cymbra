import { afterEach, describe, expect, it } from "vitest";
import * as storeChoice from "../src/lib/store-choice";
import { runScript } from "./support/inline-script";

// The store buttons' script (change: add-site-lingua-showcase, D11), run in jsdom on the markup
// `StoreButtons.astro` (`choose`) and `StoreChoice.astro` write — as `test/astro/lingua-showcase.spec.ts`
// pins it — for a few browsers: which button turns primary, where Firefox's points, which hint shows.

const COMPONENT = "src/components/StoreChoice.astro";
const FIREFOX_DESKTOP = "https://addons.mozilla.org/firefox/addon/cymbra-lingua/";
const FIREFOX_ANDROID = "https://addons.mozilla.org/android/addon/cymbra-lingua/";

const FIREFOX_ON_ANDROID = "Mozilla/5.0 (Android 15; Mobile; rv:143.0) Gecko/143.0 Firefox/143.0";
const CHROME_ON_ANDROID =
  "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Mobile Safari/537.36";
const CHROME_ON_IPHONE =
  "Mozilla/5.0 (iPhone; CPU iPhone OS 18_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/141.0.7390.41 Mobile/15E148 Safari/604.1";
const CHROME_ON_WINDOWS =
  "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36";

const FACTS = ["userAgent", "platform", "maxTouchPoints", "userAgentData"] as const;

/** The Lingua page's store row and hints, and a browser saying `userAgent` of itself. */
function load(userAgent: string, { platform = "", maxTouchPoints = 0 } = {}) {
  document.body.innerHTML = `
    <div class="cta-row" data-store-choice>
      <a class="btn btn-ghost" href="https://chromewebstore.google.com/detail/cymbra-lingua/x" data-store="chrome">Chrome</a>
      <a class="btn btn-ghost" href="${FIREFOX_DESKTOP}" data-store="firefox" data-android-href="${FIREFOX_ANDROID}">Firefox</a>
      <a class="btn btn-ghost" href="https://apps.apple.com/app/id6813053825" data-store="safari">Safari</a>
    </div>
    <p class="muted small store-hint" data-store-hint="ios" hidden>iOS</p>
    <p class="muted small store-hint" data-store-hint="android" hidden>Android</p>`;
  const facts = { userAgent, platform, maxTouchPoints, userAgentData: undefined };
  for (const name of FACTS) Object.defineProperty(navigator, name, { value: facts[name], configurable: true });
  runScript(COMPONENT, { "../lib/store-choice": storeChoice });
  const button = (store: string) => document.querySelector<HTMLAnchorElement>(`a[data-store="${store}"]`)!;
  return {
    primary: [...document.querySelectorAll("a.btn-primary")].map((a) => a.getAttribute("data-store")),
    firefoxHref: button("firefox").getAttribute("href"),
    hints: [...document.querySelectorAll<HTMLElement>("[data-store-hint]")].filter((p) => !p.hidden).map((p) => p.dataset.storeHint),
  };
}

afterEach(() => {
  for (const name of FACTS) Reflect.deleteProperty(navigator, name);
  document.body.innerHTML = "";
});

describe("the store buttons' script (D11)", () => {
  it("Firefox on Android: the Firefox button, pointed at the Android listing, no hint", () => {
    expect(load(FIREFOX_ON_ANDROID, { platform: "Linux armv81", maxTouchPoints: 5 })).toEqual({
      primary: ["firefox"],
      firefoxHref: FIREFOX_ANDROID,
      hints: [],
    });
  });

  it("another Android browser: the Firefox button, pointed at the Android listing, and the Android hint", () => {
    expect(load(CHROME_ON_ANDROID, { platform: "Linux armv81", maxTouchPoints: 5 })).toEqual({
      primary: ["firefox"],
      firefoxHref: FIREFOX_ANDROID,
      hints: ["android"],
    });
  });

  it("Chrome on an iPhone: the App Store, and the hint that Lingua runs in Safari there", () => {
    expect(load(CHROME_ON_IPHONE, { platform: "iPhone", maxTouchPoints: 5 })).toEqual({
      primary: ["safari"],
      firefoxHref: FIREFOX_DESKTOP,
      hints: ["ios"],
    });
  });

  it("Chrome on a computer: the Chrome Web Store, Firefox's link and the hints untouched", () => {
    expect(load(CHROME_ON_WINDOWS, { platform: "Win32" })).toEqual({
      primary: ["chrome"],
      firefoxHref: FIREFOX_DESKTOP,
      hints: [],
    });
  });

  it("an unknown browser: the buttons stay alike", () => {
    expect(load("curl/8.7.1")).toEqual({ primary: [], firefoxHref: FIREFOX_DESKTOP, hints: [] });
  });
});
