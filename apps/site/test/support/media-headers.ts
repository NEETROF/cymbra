// Reads the few facts the media checks need straight from the files' headers (change:
// add-site-music-showcase): a PNG's size, and an MP4's duration and track kinds. No
// decoder, no ffprobe — the checks run in CI, where neither is installed.

/** Width and height from a PNG's IHDR chunk. */
export function pngSize(bytes: Uint8Array): { width: number; height: number } {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const signature = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
  if (!signature.every((b, i) => bytes[i] === b)) throw new Error("not a PNG");
  return { width: view.getUint32(16), height: view.getUint32(20) };
}

interface Box {
  type: string;
  start: number; // payload start
  end: number;
}

function boxes(view: DataView, from: number, to: number): Box[] {
  const out: Box[] = [];
  let at = from;
  while (at + 8 <= to) {
    let size = view.getUint32(at);
    const type = String.fromCharCode(...[4, 5, 6, 7].map((i) => view.getUint8(at + i)));
    let header = 8;
    if (size === 1) {
      size = Number(view.getBigUint64(at + 8));
      header = 16;
    } else if (size === 0) size = to - at;
    if (size < header || at + size > to) throw new Error(`malformed MP4 box ${type} at ${at}`);
    out.push({ type, start: at + header, end: at + size });
    at += size;
  }
  return out;
}

function child(view: DataView, parent: Box, type: string): Box | undefined {
  return boxes(view, parent.start, parent.end).find((b) => b.type === type);
}

/** Duration in seconds (from `mvhd`) and the handler type of every track (`vide`, `soun`…). */
export function mp4Info(bytes: Uint8Array): { seconds: number; tracks: string[] } {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const moov = boxes(view, 0, bytes.byteLength).find((b) => b.type === "moov");
  if (!moov) throw new Error("no moov box");
  const mvhd = child(view, moov, "mvhd");
  if (!mvhd) throw new Error("no mvhd box");
  const version = view.getUint8(mvhd.start);
  const [timescale, duration] =
    version === 1
      ? [view.getUint32(mvhd.start + 20), Number(view.getBigUint64(mvhd.start + 24))]
      : [view.getUint32(mvhd.start + 12), view.getUint32(mvhd.start + 16)];
  const tracks = boxes(view, moov.start, moov.end)
    .filter((b) => b.type === "trak")
    .map((trak) => {
      const mdia = child(view, trak, "mdia");
      const hdlr = mdia && child(view, mdia, "hdlr");
      if (!hdlr) return "?";
      return String.fromCharCode(...[8, 9, 10, 11].map((i) => view.getUint8(hdlr.start + i)));
    });
  return { seconds: duration / timescale, tracks };
}

/** Whether `moov` comes before `mdat`, so a browser can start playing before the download ends. */
export function mp4FastStart(bytes: Uint8Array): boolean {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const order = boxes(view, 0, bytes.byteLength).map((b) => b.type);
  return order.indexOf("moov") !== -1 && order.indexOf("moov") < order.indexOf("mdat");
}
