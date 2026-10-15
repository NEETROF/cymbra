import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { runScript } from "./support/inline-script";

// The demo video's script (change: add-site-lingua-showcase, D9), run in jsdom on the markup it
// drives: it plays the video, muted, once half of it is on screen, and pauses it when it leaves —
// only when motion is welcome and data is not saved; the visitor's own play or pause, or the end,
// hands the video back to the visitor.

const COMPONENT = "src/components/DemoVideo.astro";

/** The IntersectionObserver the script makes, driven by the test. */
class FakeObserver {
  static made: FakeObserver[] = [];
  observed: Element[] = [];
  constructor(readonly callback: (entries: { intersectionRatio: number }[]) => void) {
    FakeObserver.made.push(this);
  }
  observe(element: Element) {
    this.observed.push(element);
  }
  disconnect() {}
  /** The video now shows `ratio` of itself. */
  show(ratio: number) {
    this.callback([{ intersectionRatio: ratio }]);
  }
}

let video: HTMLVideoElement;
let played: number;
let paused: number;

/** A page holding the component's video, a browser whose motion and data settings are given. */
function setUp({ reducedMotion = false, saveData = false } = {}) {
  document.body.innerHTML = '<video class="demo-video" data-demo-video muted playsinline controls preload="none"></video>';
  video = document.querySelector("video")!;
  let isPaused = true;
  played = 0;
  paused = 0;
  Object.defineProperty(video, "paused", { get: () => isPaused, configurable: true });
  video.play = vi.fn(() => {
    played += 1;
    isPaused = false;
    video.dispatchEvent(new Event("play"));
    return Promise.resolve();
  });
  video.pause = vi.fn(() => {
    paused += 1;
    isPaused = true;
    video.dispatchEvent(new Event("pause"));
  });
  vi.stubGlobal("matchMedia", (query: string) => ({
    matches: query === "(prefers-reduced-motion: no-preference)" ? !reducedMotion : false,
  }));
  Object.defineProperty(navigator, "connection", { value: { saveData }, configurable: true });
  FakeObserver.made = [];
  vi.stubGlobal("IntersectionObserver", FakeObserver);
  runScript(COMPONENT);
}

beforeEach(() => {
  document.body.innerHTML = "";
});
afterEach(() => {
  vi.unstubAllGlobals();
  Reflect.deleteProperty(navigator, "connection");
});

describe("the demo video's script (D9)", () => {
  it("plays the video, muted, once half of it is on screen, and pauses it when it leaves", () => {
    setUp();
    expect(FakeObserver.made).toHaveLength(1);
    expect(FakeObserver.made[0].observed).toEqual([video]);
    const observer = FakeObserver.made[0];
    observer.show(0.3);
    expect(played).toBe(0);
    video.muted = false;
    observer.show(0.6);
    expect(played).toBe(1);
    expect(video.muted).toBe(true);
    observer.show(0.2);
    expect(paused).toBe(1);
    observer.show(0.8);
    expect(played).toBe(2);
  });

  it("does nothing for a visitor who asks for reduced motion", () => {
    setUp({ reducedMotion: true });
    expect(FakeObserver.made).toHaveLength(0);
    expect(played).toBe(0);
  });

  it("does nothing for a visitor who saves data", () => {
    setUp({ saveData: true });
    expect(FakeObserver.made).toHaveLength(0);
    expect(played).toBe(0);
  });

  it("leaves the video to the visitor once they paused it themselves", () => {
    setUp();
    const observer = FakeObserver.made[0];
    observer.show(0.7);
    expect(played).toBe(1);
    // The visitor's own pause: not one the script asked for.
    Object.defineProperty(video, "paused", { get: () => true, configurable: true });
    video.dispatchEvent(new Event("pause"));
    observer.show(0.1);
    observer.show(0.9);
    expect(played).toBe(1);
    expect(paused).toBe(0);
  });

  it("plays once: after the end, scrolling back does not start it again", () => {
    setUp();
    const observer = FakeObserver.made[0];
    observer.show(0.7);
    Object.defineProperty(video, "ended", { get: () => true, configurable: true });
    video.dispatchEvent(new Event("ended"));
    observer.show(0.1);
    observer.show(0.9);
    expect(played).toBe(1);
  });
});
