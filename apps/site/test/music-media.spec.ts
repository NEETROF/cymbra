import { createHash } from "node:crypto";
import { existsSync, readFileSync, statSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { LANGS } from "../src/lib/i18n";
import {
  CAPTURE_SOURCE,
  GALLERY,
  MEDIA_DIR,
  POSTER_FILE,
  SHARE_COPY,
  SHARE_FILE,
  SHARE_HEIGHT,
  SHARE_WIDTH,
  VIDEO_COPY,
  VIDEO_FILE,
  VIDEO_MAX_BYTES,
  VIDEO_MAX_SECONDS,
  VIDEO_SOURCE,
} from "../src/lib/music-media";
import { mp4FastStart, mp4Info, pngSize } from "./support/media-headers";

// The Music page's media (change: add-site-music-showcase, spec site-music-page): every
// file the page shows is a byte-for-byte copy of what the Music capture harness generated,
// the video stays within its budget and carries sound, and every image says what it shows
// in every language. A capture refreshed without `yarn sync:music-media` fails here, naming
// the file — the `site` CI filter watches `apps/music/store/**` for exactly that.

const site = resolve(__dirname, "..");
const store = resolve(site, "../music/store");
const sha = (path: string) => createHash("sha256").update(readFileSync(path)).digest("hex");

describe("the Music page's media", () => {
  for (const lang of LANGS) {
    const copies = [
      ...GALLERY.map((s) => ({ name: s.file, source: resolve(store, CAPTURE_SOURCE, lang, s.file) })),
      ...[VIDEO_FILE, POSTER_FILE].map((f) => ({ name: f, source: resolve(store, VIDEO_SOURCE, lang, f) })),
    ];
    for (const { name, source } of copies) {
      it(`${lang}/${name} is the harness's file, unchanged`, () => {
        const copy = resolve(site, MEDIA_DIR, lang, name);
        expect(existsSync(copy), `${MEDIA_DIR}/${lang}/${name} is missing — run \`yarn sync:music-media\``).toBe(true);
        expect(sha(copy), `${MEDIA_DIR}/${lang}/${name} drifted from ${source} — run \`yarn sync:music-media\``).toBe(
          sha(source),
        );
      });
    }

    it(`${lang}: the video fits the budget, carries sound and streams`, () => {
      const path = resolve(site, MEDIA_DIR, lang, VIDEO_FILE);
      expect(statSync(path).size).toBeLessThanOrEqual(VIDEO_MAX_BYTES);
      const bytes = new Uint8Array(readFileSync(path));
      const { seconds, tracks } = mp4Info(bytes);
      expect(seconds).toBeGreaterThan(5);
      expect(seconds).toBeLessThanOrEqual(VIDEO_MAX_SECONDS);
      expect(tracks).toContain("vide");
      expect(tracks, "the video has no sound").toContain("soun");
      expect(mp4FastStart(bytes), "moov after mdat: re-encode with +faststart").toBe(true);
    });

    it(`${lang}: the share preview is ${SHARE_WIDTH}×${SHARE_HEIGHT}`, () => {
      const bytes = new Uint8Array(readFileSync(resolve(site, MEDIA_DIR, lang, SHARE_FILE)));
      expect(pngSize(bytes)).toEqual({ width: SHARE_WIDTH, height: SHARE_HEIGHT });
    });

    it(`${lang}: every image and the video say what they show`, () => {
      for (const shot of GALLERY) {
        expect(shot.alt[lang].trim(), `${shot.file} alt`).not.toBe("");
        expect(shot.caption[lang].trim(), `${shot.file} caption`).not.toBe("");
      }
      for (const text of Object.values(VIDEO_COPY[lang])) expect(text.trim()).not.toBe("");
      expect(SHARE_COPY[lang].alt.trim()).not.toBe("");
    });
  }
});

describe("the header readers", () => {
  it("rejects a file that is not a PNG", () => {
    expect(() => pngSize(new Uint8Array(32))).toThrow("not a PNG");
  });

  it("reads a minimal MP4: duration, tracks, box order", () => {
    const box = (type: string, payload: number[]) => {
      const size = 8 + payload.length;
      return [size >>> 24, (size >>> 16) & 255, (size >>> 8) & 255, size & 255, ...[...type].map((c) => c.charCodeAt(0)), ...payload];
    };
    const u32 = (n: number) => [n >>> 24, (n >>> 16) & 255, (n >>> 8) & 255, n & 255];
    // mvhd v0: version+flags, creation, modification, timescale 1000, duration 12500.
    const mvhd = box("mvhd", [0, 0, 0, 0, ...u32(0), ...u32(0), ...u32(1000), ...u32(12500)]);
    const hdlr = (kind: string) => box("hdlr", [0, 0, 0, 0, ...u32(0), ...[...kind].map((c) => c.charCodeAt(0))]);
    const trak = (kind: string) => box("trak", box("mdia", hdlr(kind)));
    const moov = box("moov", [...mvhd, ...trak("vide"), ...trak("soun")]);
    const mdat = box("mdat", [1, 2, 3]);
    const fast = new Uint8Array([...box("ftyp", u32(0)), ...moov, ...mdat]);
    expect(mp4Info(fast)).toEqual({ seconds: 12.5, tracks: ["vide", "soun"] });
    expect(mp4FastStart(fast)).toBe(true);
    expect(mp4FastStart(new Uint8Array([...mdat, ...moov]))).toBe(false);
    expect(() => mp4Info(mdat.length ? new Uint8Array(mdat) : new Uint8Array())).toThrow("no moov");
  });
});
