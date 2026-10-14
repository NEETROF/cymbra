// Generates the share preview of each Music page (change: add-site-music-showcase, D5):
// `yarn gen:music-share`. A 1200×630 image per language — the iPad staff capture,
// cropped, under the page's headline — written to `src/assets/music/<lang>/share.png`
// and committed, so the build stays a pure copy. Vite gives the built file a content
// hash, so a new image is a new URL and no chat app keeps showing the old one.

import { mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import sharp from "sharp";
import { LANGS } from "../src/lib/i18n.ts";
import {
  CAPTURE_SOURCE,
  MEDIA_DIR,
  SHARE_COPY,
  SHARE_FILE,
  SHARE_HEIGHT,
  SHARE_WIDTH,
} from "../src/lib/music-media.ts";

const site = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const store = resolve(site, "../music/store");
const W = SHARE_WIDTH;
const H = SHARE_HEIGHT;
/** Where the crop starts in the capture scaled to `W` wide (2752×2064 → 1200×900). */
const STAFF_TOP = 200;

const escape = (s: string) =>
  s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

/** Splits a headline into lines of at most `max` characters, on word boundaries. */
function wrap(text: string, max: number): string[] {
  const lines: string[] = [];
  let line = "";
  for (const word of text.split(" ")) {
    if (line && (line + " " + word).length > max) {
      lines.push(line);
      line = word;
    } else line = line ? `${line} ${word}` : word;
  }
  if (line) lines.push(line);
  return lines;
}

function overlay(headline: string, sub: string): Buffer {
  const lines = wrap(headline, 30);
  const lineHeight = 64;
  const top = H - 70 - 44 - lines.length * lineHeight;
  const text = lines
    .map((l, i) => `<text x="64" y="${top + (i + 1) * lineHeight - 12}" class="h">${escape(l)}</text>`)
    .join("");
  return Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}">
  <defs>
    <linearGradient id="fade" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0.45" stop-color="#0d0f14" stop-opacity="0"/>
      <stop offset="0.62" stop-color="#0d0f14" stop-opacity="0.96"/>
    </linearGradient>
    <style>
      .k { font: 600 26px "Helvetica Neue", Arial, sans-serif; fill: #35d0a5; letter-spacing: 3px; }
      .h { font: 700 54px "Helvetica Neue", Arial, sans-serif; fill: #e7e9ee; }
      .s { font: 400 28px "Helvetica Neue", Arial, sans-serif; fill: #9aa3b2; }
    </style>
  </defs>
  <rect width="${W}" height="${H}" fill="url(#fade)"/>
  <text x="64" y="${top - 18}" class="k">CYMBRA MUSIC</text>
  ${text}
  <text x="64" y="${H - 56}" class="s">${escape(sub)}</text>
</svg>`);
}

for (const lang of LANGS) {
  const { headline, sub } = SHARE_COPY[lang];
  const capture = resolve(store, CAPTURE_SOURCE, lang, "03_staff.png");
  const out = resolve(site, MEDIA_DIR, lang, SHARE_FILE);
  mkdirSync(dirname(out), { recursive: true });
  // Full width, then the band holding the staff: it sits mid-capture, and the bottom
  // of the preview belongs to the headline.
  const scaled = await sharp(capture).resize({ width: W }).toBuffer();
  await sharp(scaled)
    .extract({ left: 0, top: STAFF_TOP, width: W, height: H })
    .composite([{ input: overlay(headline, sub) }])
    .png({ compressionLevel: 9 })
    .toFile(out);
  console.log(`${lang}: ${out}`);
}
