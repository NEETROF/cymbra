import { afterEach, describe, expect, it, vi } from "vitest";
import { MUSICXML_MIME, musicxmlFileName, saveBytesAsFile } from "@/lib/download";

describe("musicxmlFileName", () => {
  it("uses the sanitised title with a .musicxml suffix", () => {
    expect(musicxmlFileName("Clair de Lune", "id-1")).toBe("Clair de Lune.musicxml");
  });

  it("strips path separators and reserved characters from the title", () => {
    expect(musicxmlFileName('a/b\\c:d*e?"f<g>h|i', "id-2")).toBe("a b c d e f g h i.musicxml");
  });

  it("keeps ordinary punctuation, dashes and parentheses", () => {
    expect(musicxmlFileName("Sonata No.1 (Allegro) - Draft", "id-3")).toBe("Sonata No.1 (Allegro) - Draft.musicxml");
  });

  it("collapses whitespace and trims trailing dots/spaces", () => {
    expect(musicxmlFileName("  Nocturne   in   C .  ", "id-4")).toBe("Nocturne in C.musicxml");
  });

  it("falls back to the identifier when the title is empty or unusable", () => {
    expect(musicxmlFileName("", "id-5")).toBe("id-5.musicxml");
    expect(musicxmlFileName("   ", "id-6")).toBe("id-6.musicxml");
    expect(musicxmlFileName("///", "id-7")).toBe("id-7.musicxml");
    expect(musicxmlFileName(null, "id-8")).toBe("id-8.musicxml");
    expect(musicxmlFileName(undefined, "id-9")).toBe("id-9.musicxml");
  });

  it("caps an over-long title", () => {
    const name = musicxmlFileName("x".repeat(500), "id-10");
    // 120-char base + ".musicxml" (9 chars).
    expect(name.length).toBe(129);
    expect(name.endsWith(".musicxml")).toBe(true);
  });
});

describe("saveBytesAsFile", () => {
  afterEach(() => vi.restoreAllMocks());

  it("wraps the bytes in a MusicXML Blob and triggers a named download, revoking the URL", () => {
    const createObjectURL = vi.fn<(blob: Blob | MediaSource) => string>(() => "blob:fake");
    const revokeObjectURL = vi.fn();
    vi.stubGlobal("URL", { ...URL, createObjectURL, revokeObjectURL });
    const clicks: string[] = [];
    const click = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(function (this: HTMLAnchorElement) {
      clicks.push(this.download);
    });

    saveBytesAsFile(new Uint8Array([1, 2, 3]), "Nocturne.musicxml");

    expect(createObjectURL).toHaveBeenCalledOnce();
    const blob = createObjectURL.mock.calls[0][0] as Blob;
    expect(blob).toBeInstanceOf(Blob);
    expect(blob.type).toBe(MUSICXML_MIME);
    expect(clicks).toEqual(["Nocturne.musicxml"]);
    // The transient anchor is cleaned up and the object URL revoked (no leak).
    expect(revokeObjectURL).toHaveBeenCalledWith("blob:fake");
    expect(document.querySelector("a[download]")).toBeNull();
    click.mockRestore();
    vi.unstubAllGlobals();
  });
});
