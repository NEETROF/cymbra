import { describe, expect, it } from "vitest";
import { linguaStores, musicStores, MUSIC_APP_STORE, MUSIC_GOOGLE_PLAY } from "../src/lib/stores";

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

  it("keeps the Lingua extension unpublished for now", () => {
    expect(linguaStores("fr").every((l) => !l.live)).toBe(true);
  });
});
