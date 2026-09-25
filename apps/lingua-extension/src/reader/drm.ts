// Whether a book is protected by a rights-management scheme, read from what it declares
// (add-lingua-reader D7). An EPUB lists every encrypted resource in META-INF/encryption.xml.
// Two algorithms there are not protection at all — font obfuscation, which only keeps an
// embedded font from being lifted out, and which the renderer undoes itself. Anything else
// means the book's text is encrypted: such a book is refused at import, with one plain
// sentence, rather than opened half-broken.

/** The font-obfuscation algorithms of the IDPF and of Adobe: not DRM. */
const FONT_OBFUSCATION = new Set(["http://www.idpf.org/2008/embedding", "http://ns.adobe.com/pdf/enc#RC"]);

/** The scheme a protected book names, for the logs; the reader is told the same sentence for all. */
export type Protection = "adept" | "lcp" | "fairplay" | "unknown";

/** Adobe ADEPT's namespace: its key info (`<resource>`) and its rights file are declared in it. */
const ADEPT_NS = "http://ns.adobe.com/adept";

/** The prefix of Readium LCP's key-retrieval types (`…/lcp#EncryptedContentKey`). */
const LCP_KEY_TYPE = "http://readium.org/2014/01/lcp#";

/**
 * The protection a book carries, or null when it carries none. `encryptionXml` is the content
 * of META-INF/encryption.xml (null when absent) and `names` the archive's entries, whose
 * scheme-specific files (an LCP licence, Adobe's rights, Apple's sinf) name the scheme.
 *
 * The scheme is read from the declaration's structure — an element in ADEPT's namespace, an
 * LCP key-retrieval type — never by searching its text, which a title or a comment could fake.
 */
export function protectionOf(encryptionXml: string | null, names: readonly string[]): Protection | null {
  const has = (name: string): boolean => names.includes(name);
  // A Readium LCP book always carries its licence; its content is always encrypted.
  if (has("META-INF/license.lcpl")) return "lcp";
  if (encryptionXml == null) return null;
  const doc = new DOMParser().parseFromString(encryptionXml, "application/xml");
  // A declaration no parser can read may hide anything: refuse rather than render garbage.
  if (doc.getElementsByTagName("parsererror").length > 0) return "unknown";
  const algorithms = [...doc.getElementsByTagNameNS("*", "EncryptedData")].map(
    (data) => data.getElementsByTagNameNS("*", "EncryptionMethod")[0]?.getAttribute("Algorithm") ?? "",
  );
  if (algorithms.every((a) => FONT_OBFUSCATION.has(a))) return null;
  if (has("META-INF/rights.xml") || doc.getElementsByTagNameNS(ADEPT_NS, "*").length > 0) return "adept";
  if (has("META-INF/sinf.xml")) return "fairplay";
  const keyTypes = [...doc.getElementsByTagNameNS("*", "RetrievalMethod")].map((m) => m.getAttribute("Type") ?? "");
  if (keyTypes.some((t) => t.startsWith(LCP_KEY_TYPE))) return "lcp";
  return "unknown";
}
