import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { type BrowserFacts, pickStore, type StoreChoice } from "../src/lib/store-choice";

// The store of the visitor's browser (change: add-site-lingua-showcase, D11), on a table of user
// agents and client hints, and the scripts that read it: they store and send nothing.

const CHROME = [{ brand: "Chromium" }, { brand: "Google Chrome" }, { brand: "Not=A?Brand" }];
const SAFARI_MAC = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.6 Safari/605.1.15";

const TABLE: [string, BrowserFacts, StoreChoice][] = [
  ["Safari on macOS", { userAgent: SAFARI_MAC, platform: "MacIntel", maxTouchPoints: 0 }, { store: "safari", hint: null }],
  [
    "Safari on iPhone",
    {
      userAgent:
        "Mozilla/5.0 (iPhone; CPU iPhone OS 18_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.6 Mobile/15E148 Safari/604.1",
      platform: "iPhone",
      maxTouchPoints: 5,
    },
    { store: "safari", hint: null },
  ],
  [
    "Chrome on iPhone",
    {
      userAgent:
        "Mozilla/5.0 (iPhone; CPU iPhone OS 18_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/141.0.7390.41 Mobile/15E148 Safari/604.1",
      platform: "iPhone",
      maxTouchPoints: 5,
    },
    { store: "safari", hint: "ios" },
  ],
  [
    "Firefox on iPhone",
    {
      userAgent:
        "Mozilla/5.0 (iPhone; CPU iPhone OS 18_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) FxiOS/143.0 Mobile/15E148 Safari/605.1.15",
      platform: "iPhone",
      maxTouchPoints: 5,
    },
    { store: "safari", hint: "ios" },
  ],
  [
    "Chrome on an iPad, which reports a Mac with touch points",
    {
      userAgent:
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/141.0.7390.41 Mobile/15E148 Safari/604.1",
      platform: "MacIntel",
      maxTouchPoints: 5,
    },
    { store: "safari", hint: "ios" },
  ],
  ["Safari on an iPad, which reports a Mac", { userAgent: SAFARI_MAC, platform: "MacIntel", maxTouchPoints: 5 }, { store: "safari", hint: null }],
  [
    "Firefox on macOS",
    { userAgent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:143.0) Gecko/20100101 Firefox/143.0", platform: "MacIntel" },
    { store: "firefox", hint: null },
  ],
  [
    "Firefox on Windows",
    { userAgent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:143.0) Gecko/20100101 Firefox/143.0", platform: "Win32" },
    { store: "firefox", hint: null },
  ],
  [
    "Firefox on Linux",
    { userAgent: "Mozilla/5.0 (X11; Linux x86_64; rv:143.0) Gecko/20100101 Firefox/143.0", platform: "Linux x86_64" },
    { store: "firefox", hint: null },
  ],
  [
    "Firefox on Android",
    { userAgent: "Mozilla/5.0 (Android 15; Mobile; rv:143.0) Gecko/143.0 Firefox/143.0", platform: "Linux armv81", maxTouchPoints: 5 },
    { store: "firefox-android", hint: null },
  ],
  [
    "Chrome on Android",
    {
      userAgent: "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Mobile Safari/537.36",
      brands: CHROME,
      platform: "Linux armv81",
      maxTouchPoints: 5,
    },
    { store: "firefox-android", hint: "android" },
  ],
  [
    "Samsung Internet on Android",
    {
      userAgent:
        "Mozilla/5.0 (Linux; Android 15; SM-S921B) AppleWebKit/537.36 (KHTML, like Gecko) SamsungBrowser/28.0 Chrome/130.0.0.0 Mobile Safari/537.36",
      platform: "Linux armv81",
      maxTouchPoints: 5,
    },
    { store: "firefox-android", hint: "android" },
  ],
  [
    "Chrome on Windows",
    {
      userAgent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36",
      brands: CHROME,
      platform: "Win32",
    },
    { store: "chrome", hint: null },
  ],
  [
    "Edge on Windows",
    {
      userAgent:
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36 Edg/141.0.0.0",
      brands: [{ brand: "Microsoft Edge" }, { brand: "Chromium" }],
      platform: "Win32",
    },
    { store: "chrome", hint: null },
  ],
  [
    "Brave on Windows",
    {
      userAgent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36",
      brands: [{ brand: "Brave" }, { brand: "Chromium" }],
      platform: "Win32",
    },
    { store: "chrome", hint: null },
  ],
  [
    "Opera on Windows, no client hints",
    {
      userAgent:
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36 OPR/123.0.0.0",
      platform: "Win32",
    },
    { store: "chrome", hint: null },
  ],
  [
    "Chrome on macOS",
    {
      userAgent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36",
      brands: CHROME,
      platform: "MacIntel",
      maxTouchPoints: 0,
    },
    { store: "chrome", hint: null },
  ],
  ["an unknown agent", { userAgent: "curl/8.7.1" }, { store: null, hint: null }],
];

describe("pickStore", () => {
  it.each(TABLE)("%s", (_name, facts, expected) => {
    expect(pickStore(facts)).toEqual(expected);
  });
});

describe("the scripts read the browser and keep nothing", () => {
  // Read from their source: no storage, no cookie, no request of any kind (D11, D9).
  const FORBIDDEN = /localStorage|sessionStorage|indexedDB|document\.cookie|cookieStore|fetch\(|XMLHttpRequest|sendBeacon|WebSocket|EventSource|navigator\.share/;
  const source = (path: string) => readFileSync(resolve(__dirname, "..", path), "utf8");

  it.each(["src/lib/store-choice.ts", "src/components/StoreChoice.astro", "src/components/DemoVideo.astro"])("%s", (path) => {
    expect(source(path)).not.toMatch(FORBIDDEN);
  });
});
