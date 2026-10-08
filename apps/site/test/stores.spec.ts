import { describe, expect, it } from "vitest";
import {
  LINGUA_APP_STORE,
  LINGUA_CHROME_WEB_STORE,
  LINGUA_FIREFOX_ADDONS,
  linguaStores,
  musicStores,
  MUSIC_APP_STORE,
  MUSIC_GOOGLE_PLAY,
} from "../src/lib/stores";

describe("store links", () => {
  it("points Music at the published listings", () => {
    const links = musicStores("fr");
    const live = links.filter((l) => l.live);
    expect(live.map((l) => l.href)).toEqual([MUSIC_APP_STORE, MUSIC_GOOGLE_PLAY]);
    // The App Store record covers the three Apple platforms: one link, one label.
    expect(live[0].label).toContain("iOS");
    expect(live[0].label).toContain("macOS");
  });

  it("never marks an unpublished channel as live", () => {
    const dead = [...musicStores("en"), ...linguaStores("en")].filter((l) => !l.live);
    expect(dead.length).toBeGreaterThan(0);
    for (const l of dead) expect(l.href).toBe("#");
  });

  it("labels the channels in each site language", () => {
    // The Spanish labels (change: add-site-lingua-matrix-pages); the store names do not move.
    expect(linguaStores("es").map((l) => l.label)).toEqual(["Chrome", "Firefox (escritorio)", "Safari (iPhone, iPad, Mac)"]);
    expect(musicStores("es")[2].label).toBe("Windows / Linux — próximamente");
    expect(linguaStores("fr")[1].label).toBe("Firefox (ordinateur)");
    expect(linguaStores("en")[1].label).toBe("Firefox (desktop)");
  });

  it("points Lingua at its three published listings, Safari's App Store record among them", () => {
    const links = linguaStores("fr");
    expect(links.every((l) => l.live)).toBe(true);
    expect(links.map((l) => l.href)).toEqual([LINGUA_CHROME_WEB_STORE, LINGUA_FIREFOX_ADDONS, LINGUA_APP_STORE]);
    expect(links[2].label).toContain("Safari");
  });
});
