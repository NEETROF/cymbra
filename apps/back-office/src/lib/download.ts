// Client-side file download helpers for the catalog. The RPC/data-fetch stays in the
// store (per the Vue architecture rule); building a Blob and triggering a browser
// save is a pure DOM concern, kept here so components stay thin and it's unit-testable.

/** The canonical (uncompressed) MusicXML media type. The bytes served by
 * `GetCatalogScoreBytes` are already `decode_canonical`-decompressed, so a download is
 * a `.musicxml` file — never the stored `.mxl`. */
export const MUSICXML_MIME = "application/vnd.recordare.musicxml+xml";

// Path separators and reserved filename characters (Windows/macOS/Linux) plus Unicode
// control chars (\p{Cc}) — replaced with a space before we collapse/trim. Ordinary
// punctuation, dashes and parentheses are kept.
const ILLEGAL_FILENAME = /[\p{Cc}/\\:*?"<>|]+/gu;
const MAX_BASE_LEN = 120;

/** Build a safe local filename for a score's MusicXML: sanitised title, falling back
 * to the identifier when the title is empty/unusable, always suffixed `.musicxml`. */
export function musicxmlFileName(title: string | null | undefined, id: string): string {
  const cleaned = (title ?? "")
    .replace(ILLEGAL_FILENAME, " ")
    .replace(/\s+/g, " ")
    .trim()
    .slice(0, MAX_BASE_LEN)
    .replace(/[. ]+$/, "");
  const base = cleaned.length > 0 ? cleaned : id;
  return `${base}.musicxml`;
}

/** Save a text document (e.g. minted codes `.txt`, a members `.csv`) — UTF-8. */
export function saveTextAsFile(text: string, fileName: string, mime = "text/plain;charset=utf-8"): void {
  saveBytesAsFile(new TextEncoder().encode(text), fileName, mime);
}

/** Save raw bytes to the operator's machine via a transient `<a download>`. */
export function saveBytesAsFile(bytes: Uint8Array, fileName: string, mime: string = MUSICXML_MIME): void {
  const url = URL.createObjectURL(new Blob([bytes], { type: mime }));
  try {
    const a = document.createElement("a");
    a.href = url;
    a.download = fileName;
    a.rel = "noopener";
    document.body.appendChild(a);
    a.click();
    a.remove();
  } finally {
    URL.revokeObjectURL(url);
  }
}
