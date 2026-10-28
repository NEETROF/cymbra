// The desktop downloads of Cymbra Music (change: add-music-download-page): the
// backend's public latest-release view (`GET /updates/desktop/latest`, spec
// `platform-app-update-feed`), the checks the browser still makes on it, and the
// pure projection the Downloads island renders.
//
// Windows ships through the Microsoft Store (a static web-installer button); Linux
// is the self-updating AppImage the view describes as `linux-x64` (design D6).

import type { Async } from "@cymbra/web-auth";

/** The project's GitHub Releases page — the fallback that is always on the page. */
export const RELEASES_PAGE = "https://github.com/NEETROF/cymbra/releases";

/** The only addresses a download link may point at (design D5). */
const ASSET_PREFIX = "/NEETROF/cymbra/releases/download/";

export interface DesktopTarget {
  kind: string;
  url: string;
  size: number;
  sha256: string;
}

/** The `200` body of the latest-release view, field names as the backend sends them. */
export interface DesktopRelease {
  version: string;
  released_at: string;
  notes_url?: string;
  targets: Record<string, DesktopTarget>;
}

export interface DesktopReleaseClient {
  /** The latest downloadable release, or `null` when the backend has none to show (`204`). */
  latest(): Promise<DesktopRelease | null>;
}

function isTarget(v: unknown): v is DesktopTarget {
  if (typeof v !== "object" || v === null) return false;
  const t = v as Record<string, unknown>;
  return (
    typeof t.kind === "string" &&
    typeof t.url === "string" &&
    typeof t.size === "number" &&
    typeof t.sha256 === "string"
  );
}

/** Narrow an untrusted body to a release, or throw — a malformed body is an error, not a card. */
export function parseRelease(body: unknown): DesktopRelease {
  if (typeof body !== "object" || body === null) throw new Error("malformed release view");
  const r = body as Record<string, unknown>;
  if (typeof r.version !== "string" || typeof r.released_at !== "string") {
    throw new Error("malformed release view");
  }
  if (r.notes_url !== undefined && typeof r.notes_url !== "string") throw new Error("malformed release view");
  if (typeof r.targets !== "object" || r.targets === null) throw new Error("malformed release view");
  const targets: Record<string, DesktopTarget> = {};
  for (const [key, t] of Object.entries(r.targets)) {
    if (!isTarget(t)) throw new Error("malformed release view");
    targets[key] = t;
  }
  return { version: r.version, released_at: r.released_at, notes_url: r.notes_url as string | undefined, targets };
}

/**
 * The browser client. A plain anonymous `GET`: no credentials, no account, no
 * version — the response is one cached document for everyone. Not `fetchJson`:
 * that helper is for the authenticated JSON calls and does not model a `204`.
 */
export function createDesktopReleaseClient(
  apiUrl: string,
  fetchImpl: typeof fetch = (...args) => fetch(...args),
): DesktopReleaseClient {
  const url = `${apiUrl}/updates/desktop/latest?product=music&channel=stable`;
  return {
    async latest() {
      const resp = await fetchImpl(url, { credentials: "omit" });
      if (resp.status === 204) return null;
      if (!resp.ok) throw new Error(`latest release view: HTTP ${resp.status}`);
      return parseRelease(await resp.json());
    },
  };
}

/** An HTTPS asset of the project's GitHub Releases, and nothing else (design D5). */
export function isReleaseAsset(address: string): boolean {
  let u: URL;
  try {
    u = new URL(address);
  } catch {
    return false;
  }
  return (
    u.protocol === "https:" &&
    u.hostname === "github.com" &&
    u.port === "" &&
    u.username === "" &&
    u.password === "" &&
    u.pathname.startsWith(ASSET_PREFIX) &&
    !u.pathname.includes("..") &&
    u.pathname.length > ASSET_PREFIX.length
  );
}

/**
 * The release page of a manifest version. `1.37.0+51` is tagged `music-v1.37.0`:
 * the build number is not part of the tag. Anything unparseable falls back to the
 * releases page rather than a dead link.
 */
export function releaseTagUrl(version: string): string {
  const m = /^(\d+)\.(\d+)\.(\d+)(\+\d+)?$/.exec(version);
  if (!m) return RELEASES_PAGE;
  return `${RELEASES_PAGE}/tag/music-v${m[1]}.${m[2]}.${m[3]}`;
}

/** The version a person reads: the build number is noise to them. */
export function displayVersion(version: string): string {
  return version.split("+")[0];
}

/** The Store's web installer for a product id, or `null` when none is configured. */
export function msStoreUrl(productId: string | null): string | null {
  if (!productId || !/^[0-9A-Z]{12}$/.test(productId)) return null;
  return `https://apps.microsoft.com/detail/${productId}?mode=direct`;
}

export type DesktopPlatform = "windows" | "linux" | null;

/** Which card to highlight (design D10: cosmetic — both always render). Android is not Linux here. */
export function detectPlatform(userAgent: string): DesktopPlatform {
  if (/Windows/i.test(userAgent)) return "windows";
  if (/Android|CrOS/i.test(userAgent)) return null;
  if (/Linux|X11/i.test(userAgent)) return "linux";
  return null;
}

/** What the Linux card shows. */
export type LinuxCard =
  /** No usable answer yet, or none at all (`204`, outage, malformed): the releases page. */
  | { kind: "fallback" }
  /** A release exists but carries no Linux build. */
  | { kind: "unavailable"; tagUrl: string }
  | {
      kind: "ready";
      version: string;
      releasedAt: string;
      url: string;
      size: number;
      sha256: string;
      tagUrl: string;
      notesUrl: string | null;
    };

/** The Linux card for a state of the request. */
export function linuxCard(state: Async<DesktopRelease | null>): LinuxCard {
  if (state.status !== "success" || state.data === null) return { kind: "fallback" };
  const r = state.data;
  const tagUrl = releaseTagUrl(r.version);
  const t = r.targets["linux-x64"];
  if (!t) return { kind: "unavailable", tagUrl };
  // A foreign address is never a link: the releases page instead.
  if (!isReleaseAsset(t.url)) return { kind: "fallback" };
  const notes = r.notes_url && /^https:\/\//.test(r.notes_url) ? r.notes_url : null;
  return {
    kind: "ready",
    version: displayVersion(r.version),
    releasedAt: r.released_at,
    url: t.url,
    size: t.size,
    sha256: t.sha256,
    tagUrl,
    notesUrl: notes,
  };
}

/** A download size in megabytes, in the page's language ("94,2 Mo" / "94.2 MB"). */
export function formatSize(lang: "fr" | "en" | "es", bytes: number): string {
  const unit = lang === "fr" ? "Mo" : "MB";
  const tag = { fr: "fr-FR", en: "en-US", es: "es-ES" }[lang];
  const value = new Intl.NumberFormat(tag, { maximumFractionDigits: 1 }).format(bytes / 1_000_000);
  return `${value} ${unit}`;
}
