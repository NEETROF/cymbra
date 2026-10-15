// The capture's images and video (add-site-lingua-showcase D7, D8): crops computed from the
// surfaces' boxes, encoding by ffmpeg — present on the owner's Mac, checked first — so that the
// extension gains no native npm dependency, and every store PNG's header read back for an alpha
// channel. The geometry, the arguments and the header reader are pure; `ffmpeg` runs them.
import { spawnSync } from "node:child_process";

/** The PNG signature, then the IHDR chunk: its colour type is byte 25 of the file. */
const PNG_SIGNATURE = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

/**
 * A PNG's header: its size and colour type (0 grey, 2 RGB, 3 palette, 4 grey + alpha, 6 RGB + alpha).
 * @param {Uint8Array} bytes
 */
export function pngHeader(bytes) {
  if (bytes.length < 29 || PNG_SIGNATURE.some((b, i) => bytes[i] !== b)) throw new Error("not a PNG file");
  const ihdr = String.fromCharCode(bytes[12], bytes[13], bytes[14], bytes[15]);
  if (ihdr !== "IHDR") throw new Error("a PNG whose first chunk is not IHDR");
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  return { width: view.getUint32(16), height: view.getUint32(20), colourType: bytes[25] };
}

/**
 * Refuse a store capture that is not an RGB PNG of the expected size: the Chrome Web Store takes no
 * alpha, and colour type 2 has none. The message names the file.
 */
export function assertStorePng(name, bytes, { width, height }) {
  const header = pngHeader(bytes);
  if (header.colourType !== 2) {
    throw new Error(`${name}: colour type ${header.colourType}, the stores need 2 (RGB, no alpha)`);
  }
  if (header.width !== width || header.height !== height) {
    throw new Error(`${name}: ${header.width}×${header.height}, expected ${width}×${height}`);
  }
}

/**
 * A rectangle of the aspect `aspect` (width / height) holding `box` and `padding` around it, at
 * least `minWidth` wide, moved inside `bounds` and shrunk to them when it cannot fit. Integers.
 * With `snapTop`, a rectangle that would start less than `snapTop` below the window's top starts at
 * it instead: a page's title is shown whole rather than cut through.
 * @param {{ box: {x:number,y:number,width:number,height:number}, aspect: number, padding?: number,
 *   minWidth?: number, snapTop?: number, bounds: {width:number,height:number} }} options
 */
export function cropAround({ box, aspect, padding = 0, minWidth = 0, snapTop = 0, bounds }) {
  let width = Math.max(box.width + 2 * padding, minWidth);
  let height = box.height + 2 * padding;
  if (width / height < aspect) width = height * aspect;
  else height = width / aspect;
  if (width > bounds.width) {
    width = bounds.width;
    height = width / aspect;
  }
  if (height > bounds.height) {
    height = bounds.height;
    width = height * aspect;
  }
  const cx = box.x + box.width / 2;
  const cy = box.y + box.height / 2;
  const x = Math.min(Math.max(cx - width / 2, 0), bounds.width - width);
  let y = Math.min(Math.max(cy - height / 2, 0), bounds.height - height);
  if (y < snapTop && box.y + box.height + 8 <= height) y = 0;
  return { x: Math.round(x), y: Math.round(y), width: Math.round(width), height: Math.round(height) };
}

/**
 * A `width`-wide rectangle of the aspect `aspect` that starts `padding` before the text column's
 * left edge — so that its lines start whole — moved right only as far as `focus` (and `padding`
 * after it) needs, centred on `focus` vertically, inside `bounds`. Integers.
 */
export function columnCrop({ column, focus, width, aspect, padding = 0, bounds }) {
  const height = Math.min(width / aspect, bounds.height);
  let x = Math.min(column.x, focus.x) - padding;
  x = Math.max(x, focus.x + focus.width + padding - width);
  x = Math.min(Math.max(x, 0), bounds.width - width);
  const cy = focus.y + focus.height / 2;
  const y = Math.min(Math.max(cy - height / 2, 0), bounds.height - height);
  return { x: Math.round(x), y: Math.round(y), width: Math.round(width), height: Math.round(height) };
}

/**
 * `box` grown by `padding` on every side (by `bottom` below it, when given: the next block may start
 * closer than `padding`) and cut to `bounds`, in whole pixels: a crop that follows its content.
 */
export function padded(box, padding, bounds, { bottom: below = padding } = {}) {
  const x = Math.max(0, Math.floor(box.x - padding));
  const y = Math.max(0, Math.floor(box.y - padding));
  const right = Math.min(bounds.width, Math.ceil(box.x + box.width + padding));
  const bottom = Math.min(bounds.height, Math.ceil(box.y + box.height + below));
  return { x, y, width: right - x, height: bottom - y };
}

/** The smallest rectangle holding every box. */
export function union(...boxes) {
  const left = Math.min(...boxes.map((b) => b.x));
  const top = Math.min(...boxes.map((b) => b.y));
  const right = Math.max(...boxes.map((b) => b.x + b.width));
  const bottom = Math.max(...boxes.map((b) => b.y + b.height));
  return { x: left, y: top, width: right - left, height: bottom - top };
}

/** A rectangle in CSS pixels as device pixels (the captures are taken at a scale factor of 2). */
export function scaled(rect, factor) {
  return {
    x: Math.round(rect.x * factor),
    y: Math.round(rect.y * factor),
    width: Math.round(rect.width * factor),
    height: Math.round(rect.height * factor),
  };
}

/** Each format's encoder settings. WebP for the site, PNG (RGB, colour type 2) for the stores, JPEG for the share image. */
const FORMATS = {
  webp: ["-c:v", "libwebp", "-quality", "88", "-preset", "text", "-compression_level", "6"],
  png: ["-c:v", "png", "-pix_fmt", "rgb24"],
  jpeg: ["-c:v", "mjpeg", "-q:v", "3", "-pix_fmt", "yuvj420p"],
};

/**
 * ffmpeg's arguments for one still: read `input`, crop it (device pixels), scale it to `width`
 * (and `height`, else the crop's aspect), encode it as `format` into `output`.
 * @param {{ input: string, output: string, format: "webp" | "png" | "jpeg", crop?: {x:number,y:number,width:number,height:number},
 *   width?: number, height?: number, quality?: number }} options
 */
export function imageArgs({ input, output, format, crop, width, height, quality }) {
  const encoder = FORMATS[format];
  if (!encoder) throw new Error(`unknown image format ${format}`);
  const filters = [];
  if (crop) filters.push(`crop=${crop.width}:${crop.height}:${crop.x}:${crop.y}`);
  if (width) filters.push(`scale=${width}:${height ?? -2}:flags=lanczos`);
  const settings = [...encoder];
  if (quality !== undefined && format === "webp") settings[3] = String(quality);
  return [
    "-hide_banner",
    "-loglevel",
    "error",
    "-y",
    "-i",
    input,
    ...(filters.length ? ["-vf", filters.join(",")] : []),
    ...settings,
    "-frames:v",
    "1",
    "-update",
    "1",
    output,
  ];
}

/**
 * ffmpeg's arguments for the demo video (D8): `duration` seconds of `input` from `start`, H.264 in
 * an MP4, no audio track at all, the index at the front so it plays while it loads, 4:2:0 so every
 * browser decodes it, at `width`×`height` and 30 frames a second.
 */
export function videoArgs({ input, output, start = 0, duration, width = 1280, height = 800, crf = 23 }) {
  return [
    "-hide_banner",
    "-loglevel",
    "error",
    "-y",
    "-ss",
    String(start),
    ...(duration ? ["-t", String(duration)] : []),
    "-i",
    input,
    "-an",
    "-vf",
    `scale=${width}:${height}:flags=lanczos,fps=30`,
    "-c:v",
    "libx264",
    "-preset",
    "slow",
    "-crf",
    String(crf),
    "-pix_fmt",
    "yuv420p",
    "-movflags",
    "+faststart",
    output,
  ];
}

/** Whether ffmpeg runs; the tool checks it before anything else. */
export function hasFfmpeg() {
  return spawnSync("ffmpeg", ["-version"], { stdio: "ignore" }).status === 0;
}

/** Run ffmpeg; a failure throws with its own message. */
export function ffmpeg(args) {
  const run = spawnSync("ffmpeg", args, { encoding: "utf8" });
  if (run.status !== 0) throw new Error(`ffmpeg ${args.join(" ")} failed: ${run.stderr || run.error}`);
}
