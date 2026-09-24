import { describe, expect, it } from "vitest";
import { protectionOf } from "@/reader/drm.ts";

// A protected book is refused at import; an obfuscated font is not protection (add-lingua-reader
// D7). The declarations below are shaped like the real ones each scheme writes.

function encryption(...data: { algorithm: string; uri: string; keyInfo?: string }[]): string {
  return `<?xml version="1.0"?>
<encryption xmlns="urn:oasis:names:tc:opendocument:xmlns:container"
            xmlns:enc="http://www.w3.org/2001/04/xmlenc#" xmlns:ds="http://www.w3.org/2000/09/xmldsig#">
${data
  .map(
    (d) => `  <enc:EncryptedData>
    <enc:EncryptionMethod Algorithm="${d.algorithm}"/>
    ${d.keyInfo ?? ""}
    <enc:CipherData><enc:CipherReference URI="${d.uri}"/></enc:CipherData>
  </enc:EncryptedData>`,
  )
  .join("\n")}
</encryption>`;
}

const IDPF_FONT = "http://www.idpf.org/2008/embedding";
const ADOBE_FONT = "http://ns.adobe.com/pdf/enc#RC";
const AES = "http://www.w3.org/2001/04/xmlenc#aes128-cbc";

describe("protectionOf", () => {
  it("lets a book with no encryption declaration through", () => {
    expect(protectionOf(null, ["mimetype", "OEBPS/content.opf"])).toBeNull();
  });

  it("lets IDPF font obfuscation through", () => {
    expect(protectionOf(encryption({ algorithm: IDPF_FONT, uri: "OEBPS/fonts/a.otf" }), [])).toBeNull();
  });

  it("lets Adobe font obfuscation through, alone or mixed with the IDPF one", () => {
    const xml = encryption(
      { algorithm: ADOBE_FONT, uri: "OEBPS/fonts/a.otf" },
      { algorithm: IDPF_FONT, uri: "OEBPS/fonts/b.otf" },
    );
    expect(protectionOf(xml, [])).toBeNull();
  });

  it("refuses Adobe ADEPT, by its key info or its rights file", () => {
    const adept = encryption({
      algorithm: AES,
      uri: "OEBPS/chapter1.xhtml",
      keyInfo: `<KeyInfo xmlns="http://www.w3.org/2000/09/xmldsig#"><resource xmlns="http://ns.adobe.com/adept">urn:uuid:x</resource></KeyInfo>`,
    });
    expect(protectionOf(adept, [])).toBe("adept");
    expect(protectionOf(encryption({ algorithm: AES, uri: "OEBPS/chapter1.xhtml" }), ["META-INF/rights.xml"])).toBe(
      "adept",
    );
  });

  it("refuses Readium LCP, by its licence even before reading the declaration", () => {
    expect(protectionOf(null, ["META-INF/license.lcpl"])).toBe("lcp");
    const lcp = encryption({
      algorithm: "http://www.w3.org/2001/04/xmlenc#aes256-cbc",
      uri: "OEBPS/chapter1.xhtml",
      keyInfo: `<ds:KeyInfo><ds:RetrievalMethod URI="license.lcpl#/encryption/content_key" Type="http://readium.org/2014/01/lcp#EncryptedContentKey"/></ds:KeyInfo>`,
    });
    expect(protectionOf(lcp, [])).toBe("lcp");
  });

  it("refuses Apple FairPlay, by its sinf file", () => {
    expect(protectionOf(encryption({ algorithm: AES, uri: "OEBPS/chapter1.xhtml" }), ["META-INF/sinf.xml"])).toBe(
      "fairplay",
    );
  });

  it("refuses content encrypted by a scheme it cannot name", () => {
    expect(protectionOf(encryption({ algorithm: AES, uri: "OEBPS/chapter1.xhtml" }), [])).toBe("unknown");
  });

  it("refuses a font obfuscated next to encrypted text", () => {
    const xml = encryption(
      { algorithm: IDPF_FONT, uri: "OEBPS/fonts/a.otf" },
      { algorithm: AES, uri: "OEBPS/chapter1.xhtml" },
    );
    expect(protectionOf(xml, [])).toBe("unknown");
  });

  it("names a scheme from the declaration's structure, not from its text", () => {
    const mention = encryption({
      algorithm: AES,
      uri: "OEBPS/chapter1.xhtml",
      keyInfo: "<!-- http://ns.adobe.com/adept -->",
    });
    expect(protectionOf(mention, [])).toBe("unknown");
    const fontsWithMention = encryption({
      algorithm: IDPF_FONT,
      uri: "a.otf",
      keyInfo: "<!-- http://ns.adobe.com/adept -->",
    });
    expect(protectionOf(fontsWithMention, [])).toBeNull();
  });

  it("refuses a declaration no parser can read", () => {
    expect(protectionOf("<encryption><EncryptedData>", [])).toBe("unknown");
  });
});
