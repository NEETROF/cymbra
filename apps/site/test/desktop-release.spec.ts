import { describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { failure, idle, loading, success } from "@cymbra/web-auth";
import MusicDownloadsIsland from "../src/components/MusicDownloadsIsland.vue";
import { setDesktopReleaseClientForTest, useLatestDesktopRelease } from "../src/composables/useLatestDesktopRelease";
import {
  createDesktopReleaseClient,
  type DesktopRelease,
  type DesktopReleaseClient,
  detectPlatform,
  displayVersion,
  formatSize,
  isReleaseAsset,
  linuxCard,
  msStoreUrl,
  parseRelease,
  RELEASES_PAGE,
  releaseTagUrl,
} from "../src/lib/desktop-release";
import { t } from "../src/lib/i18n";

const ASSET = "https://github.com/NEETROF/cymbra/releases/download/music-v1.37.0/Cymbra-Music-x86_64.AppImage";
const SHA = "ab".repeat(32);

function release(over: Partial<DesktopRelease> = {}): DesktopRelease {
  return {
    version: "1.37.0+51",
    released_at: "2026-10-28T10:00:00Z",
    notes_url: "https://github.com/NEETROF/cymbra/releases/tag/music-v1.37.0",
    targets: { "linux-x64": { kind: "appimage", url: ASSET, size: 98_765_432, sha256: SHA } },
    ...over,
  };
}

function fakeClient(answer: () => Promise<DesktopRelease | null>): DesktopReleaseClient {
  return { latest: vi.fn(answer) };
}

describe("release asset addresses (design D5)", () => {
  it("accepts an HTTPS asset of the project's releases", () => {
    expect(isReleaseAsset(ASSET)).toBe(true);
  });

  it.each([
    ["plain http", ASSET.replace("https:", "http:")],
    ["another repository", "https://github.com/evil/cymbra/releases/download/v1/x.AppImage"],
    ["another host", "https://evil.example/NEETROF/cymbra/releases/download/v1/x"],
    ["a lookalike host", "https://github.com.evil.example/NEETROF/cymbra/releases/download/v1/x"],
    ["credentials in the address", "https://u:p@github.com/NEETROF/cymbra/releases/download/v1/x"],
    ["a port", "https://github.com:8443/NEETROF/cymbra/releases/download/v1/x"],
    ["the releases page itself", "https://github.com/NEETROF/cymbra/releases/download/"],
    ["not an address", "javascript:alert(1)"],
    ["garbage", "::::"],
  ])("refuses %s", (_, address) => {
    expect(isReleaseAsset(address)).toBe(false);
  });
});

describe("release tag links", () => {
  it("drops the build number: 1.37.0+51 is tagged music-v1.37.0", () => {
    expect(releaseTagUrl("1.37.0+51")).toBe(`${RELEASES_PAGE}/tag/music-v1.37.0`);
    expect(releaseTagUrl("1.37.0")).toBe(`${RELEASES_PAGE}/tag/music-v1.37.0`);
  });

  it("falls back to the releases page for an unparseable version", () => {
    expect(releaseTagUrl("v1.37")).toBe(RELEASES_PAGE);
  });

  it("shows the version without its build number", () => {
    expect(displayVersion("1.37.0+51")).toBe("1.37.0");
  });
});

describe("the Store web installer", () => {
  it("is built from a configured product id", () => {
    expect(msStoreUrl("9PLF3P3MVM3S")).toBe("https://apps.microsoft.com/detail/9PLF3P3MVM3S?mode=direct");
  });

  it("is absent without an id, or with one that is not a Store id", () => {
    expect(msStoreUrl(null)).toBeNull();
    expect(msStoreUrl("")).toBeNull();
    expect(msStoreUrl("9plf3p3mvm3s/../x")).toBeNull();
  });
});

describe("platform highlight (design D10)", () => {
  it("tells Windows and desktop Linux apart, and leaves Android and ChromeOS alone", () => {
    expect(detectPlatform("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")).toBe("windows");
    expect(detectPlatform("Mozilla/5.0 (X11; Linux x86_64)")).toBe("linux");
    expect(detectPlatform("Mozilla/5.0 (Linux; Android 14; Pixel 8)")).toBeNull();
    expect(detectPlatform("Mozilla/5.0 (X11; CrOS x86_64 14541.0.0)")).toBeNull();
    expect(detectPlatform("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5)")).toBeNull();
  });
});

describe("parsing the view", () => {
  it("keeps a well-formed body", () => {
    expect(parseRelease(release())).toEqual(release());
  });

  it.each([
    ["null", null],
    ["no version", { released_at: "x", targets: {} }],
    ["targets not an object", { version: "1.0.0", released_at: "x", targets: 3 }],
    ["a target without size", { version: "1.0.0", released_at: "x", targets: { "linux-x64": { kind: "a", url: "u", sha256: "s" } } }],
    ["notes not a string", { version: "1.0.0", released_at: "x", notes_url: 1, targets: {} }],
  ])("refuses %s", (_, body) => {
    expect(() => parseRelease(body)).toThrow();
  });
});

describe("the client", () => {
  const json = (body: unknown, status = 200) =>
    new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json" } });

  it("asks the anonymous view, without credentials", async () => {
    const fetchImpl = vi.fn(async () => json(release()));
    const r = await createDesktopReleaseClient("https://api.example", fetchImpl).latest();
    expect(r?.version).toBe("1.37.0+51");
    expect(fetchImpl).toHaveBeenCalledWith(
      "https://api.example/updates/desktop/latest?product=music&channel=stable",
      { credentials: "omit" },
    );
  });

  it("reads 204 as nothing to show", async () => {
    const fetchImpl = vi.fn(async () => new Response(null, { status: 204 }));
    expect(await createDesktopReleaseClient("https://api.example", fetchImpl).latest()).toBeNull();
  });

  it("throws on an error status or a malformed body", async () => {
    await expect(createDesktopReleaseClient("x", vi.fn(async () => json({}, 500))).latest()).rejects.toThrow();
    await expect(createDesktopReleaseClient("x", vi.fn(async () => json({ nope: 1 }))).latest()).rejects.toThrow();
  });
});

describe("the Linux card", () => {
  it("falls back while nothing usable is known", () => {
    expect(linuxCard(idle)).toEqual({ kind: "fallback" });
    expect(linuxCard(loading)).toEqual({ kind: "fallback" });
    expect(linuxCard(failure("x"))).toEqual({ kind: "fallback" });
    expect(linuxCard(success(null))).toEqual({ kind: "fallback" });
  });

  it("describes a release with a Linux target", () => {
    expect(linuxCard(success(release()))).toEqual({
      kind: "ready",
      version: "1.37.0",
      releasedAt: "2026-10-28T10:00:00Z",
      url: ASSET,
      size: 98_765_432,
      sha256: SHA,
      tagUrl: `${RELEASES_PAGE}/tag/music-v1.37.0`,
      notesUrl: "https://github.com/NEETROF/cymbra/releases/tag/music-v1.37.0",
    });
  });

  it("says unavailable when the release has no Linux build", () => {
    expect(linuxCard(success(release({ targets: {} })))).toEqual({
      kind: "unavailable",
      tagUrl: `${RELEASES_PAGE}/tag/music-v1.37.0`,
    });
  });

  it("never links a foreign address", () => {
    const foreign = release({ targets: { "linux-x64": { kind: "appimage", url: "https://evil.example/x", size: 1, sha256: SHA } } });
    expect(linuxCard(success(foreign))).toEqual({ kind: "fallback" });
  });

  it("drops absent or non-HTTPS notes", () => {
    const card = (r: DesktopRelease) => linuxCard(success(r));
    expect(card(release({ notes_url: undefined }))).toMatchObject({ notesUrl: null });
    expect(card(release({ notes_url: "javascript:alert(1)" }))).toMatchObject({ notesUrl: null });
  });

  it("formats the size in the page's language", () => {
    expect(formatSize("fr", 98_765_432)).toBe("98,8 Mo");
    expect(formatSize("en", 98_765_432)).toBe("98.8 MB");
    expect(formatSize("es", 98_765_432)).toBe("98,8 MB");
  });
});

describe("useLatestDesktopRelease", () => {
  it("lands on success, on 204 as success(null), and on error without throwing", async () => {
    setDesktopReleaseClientForTest(fakeClient(async () => release()));
    let s = useLatestDesktopRelease();
    await s.load();
    expect(s.release.value).toEqual(success(release()));
    expect(s.linux.value.kind).toBe("ready");

    setDesktopReleaseClientForTest(fakeClient(async () => null));
    s = useLatestDesktopRelease();
    await s.load();
    expect(s.release.value).toEqual(success(null));
    expect(s.linux.value.kind).toBe("fallback");

    vi.spyOn(console, "warn").mockImplementation(() => undefined);
    setDesktopReleaseClientForTest(fakeClient(async () => Promise.reject(new TypeError("Failed to fetch"))));
    s = useLatestDesktopRelease();
    await s.load();
    expect(s.release.value.status).toBe("error");
    expect(s.linux.value.kind).toBe("fallback");
    setDesktopReleaseClientForTest(null);
  });
});

describe("MusicDownloadsIsland", () => {
  async function render(
    answer: () => Promise<DesktopRelease | null>,
    storeProductId: string | null = "9PLF3P3MVM3S",
    lang: "fr" | "en" | "es" = "fr",
  ) {
    setDesktopReleaseClientForTest(fakeClient(answer));
    const w = mount(MusicDownloadsIsland, { props: { lang, storeProductId } });
    await flushPromises();
    return w;
  }

  it("fills the Linux card with the latest release", async () => {
    const w = await render(async () => release());
    const linux = w.get('[data-testid="dl-linux"]');
    expect(linux.text()).toContain("1.37.0");
    expect(linux.text()).toContain("98,8 Mo");
    expect(w.get('[data-testid="dl-linux-download"]').attributes("href")).toBe(ASSET);
    expect(w.get('[data-testid="dl-linux-sha"]').text()).toBe(SHA);
    expect(w.get('[data-testid="dl-linux-notes"]').attributes("href")).toContain("music-v1.37.0");
    expect(w.get('[data-testid="dl-releases"]').attributes("href")).toBe(RELEASES_PAGE);
  });

  it("shows no notes link when the release has none", async () => {
    const w = await render(async () => release({ notes_url: undefined }));
    expect(w.find('[data-testid="dl-linux-notes"]').exists()).toBe(false);
  });

  it.each([
    ["204", async () => null],
    ["an outage", async () => Promise.reject(new TypeError("Failed to fetch"))],
  ])("falls back to the releases page on %s, with no error text", async (_, answer) => {
    vi.spyOn(console, "warn").mockImplementation(() => undefined);
    const w = await render(answer as () => Promise<DesktopRelease | null>);
    expect(w.get('[data-testid="dl-linux-fallback"]').text()).toBe(t("fr", "dlLinuxFallback"));
    expect(w.find('[data-testid="dl-linux-download"]').exists()).toBe(false);
    expect(w.get('[data-testid="dl-releases"]').attributes("href")).toBe(RELEASES_PAGE);
    expect(w.text()).not.toMatch(/fetch|HTTP|error|erreur/i);
  });

  it("says the Linux build is not available when the release has none", async () => {
    const w = await render(async () => release({ targets: {} }));
    expect(w.get('[data-testid="dl-linux"]').text()).toContain(t("fr", "dlLinuxUnavailable"));
    expect(w.find('[data-testid="dl-linux-download"]').exists()).toBe(false);
  });

  it("offers the Store web installer when the id is configured, and 'coming soon' otherwise", async () => {
    let w = await render(async () => null);
    const button = w.get('[data-testid="dl-windows"] a');
    expect(button.attributes("href")).toBe("https://apps.microsoft.com/detail/9PLF3P3MVM3S?mode=direct");
    expect(w.get('[data-testid="dl-windows"]').text()).toContain(t("fr", "dlWindowsBody"));

    w = await render(async () => null, null);
    expect(w.find('[data-testid="dl-windows"] a').exists()).toBe(false);
    expect(w.get('[data-testid="dl-windows"]').text()).toContain(t("fr", "dlWindowsSoon"));
  });

  it("keeps the install guidance visible in every language, and never suggests turning a protection off", async () => {
    for (const lang of ["fr", "en", "es"] as const) {
      const w = await render(async () => release(), "9PLF3P3MVM3S", lang);
      const text = w.text();
      for (const key of ["dlLinuxOneFile", "dlLinuxExecutable", "dlLinuxFolder", "dlLinuxFuse", "dlLinuxMenu", "dlWindowsBody"] as const) {
        expect(text).toContain(t(lang, key));
      }
      expect(text).toContain("chmod +x");
      expect(text).toContain("~/Applications");
      expect(text).not.toMatch(/Smart ?App ?Control|SmartScreen|désactiv|disable|desactiv/i);
    }
  });

  it("highlights the visitor's platform, and keeps both cards", async () => {
    vi.spyOn(navigator, "userAgent", "get").mockReturnValue("Mozilla/5.0 (X11; Linux x86_64)");
    const w = await render(async () => release());
    expect(w.get('[data-testid="dl-linux"]').classes()).toContain("highlight");
    expect(w.get('[data-testid="dl-windows"]').classes()).not.toContain("highlight");
    expect(w.get('[data-testid="dl-linux"]').text()).toContain(t("fr", "dlYourSystem"));
  });
});
