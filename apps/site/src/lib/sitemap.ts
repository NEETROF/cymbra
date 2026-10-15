// `sitemap.xml`, from the built pages (change: add-site-lingua-showcase, D13): `astro.config.mjs`
// reads every page `astro build` wrote and hands each one's HTML here. A page enters the sitemap by
// existing and leaves it by being `noindex`, with no list to keep; its `<loc>` is its canonical link
// and its alternates the `hreflang` links it declares (`Base.astro`). An indexable page without a
// canonical fails the build. No `lastmod`: no page has a date we could trust.

/** What a built page's head says to a crawler. */
export interface PageHead {
  noindex: boolean;
  canonical: string | null;
  alternates: { hreflang: string; href: string }[];
}

/** An attribute of a tag's source, or null. */
function attribute(tag: string, name: string): string | null {
  return tag.match(new RegExp(`\\s${name}="([^"]*)"`))?.[1] ?? null;
}

/** The robots meta, the canonical link and the `hreflang` links of a built page. */
export function readPageHead(html: string): PageHead {
  const head = html.match(/<head>([\s\S]*?)<\/head>/)?.[1] ?? "";
  const tags = [...head.matchAll(/<(?:meta|link)\b[^>]*>/g)].map((m) => m[0]);
  const noindex = tags.some(
    (tag) => tag.startsWith("<meta") && attribute(tag, "name") === "robots" && /noindex/.test(attribute(tag, "content") ?? ""),
  );
  const canonical = tags.find((tag) => tag.startsWith("<link") && attribute(tag, "rel") === "canonical");
  const alternates = tags
    .filter((tag) => tag.startsWith("<link") && attribute(tag, "rel") === "alternate" && attribute(tag, "hreflang"))
    .map((tag) => ({ hreflang: attribute(tag, "hreflang")!, href: attribute(tag, "href")! }));
  return { noindex, canonical: canonical ? attribute(canonical, "href") : null, alternates };
}

/** One `<url>` of the sitemap. */
export interface SitemapEntry {
  loc: string;
  alternates: { hreflang: string; href: string }[];
}

/**
 * The sitemap's entries: every page that is not `noindex`, sorted by address. A page that is
 * indexable and names no canonical is refused, naming it (`file` is its path in `dist/`).
 */
export function sitemapEntries(pages: { file: string; html: string }[]): SitemapEntry[] {
  const entries: SitemapEntry[] = [];
  const missing: string[] = [];
  for (const { file, html } of pages) {
    const head = readPageHead(html);
    if (head.noindex) continue;
    if (!head.canonical) missing.push(file);
    else entries.push({ loc: head.canonical, alternates: head.alternates });
  }
  if (missing.length > 0) {
    throw new Error(`sitemap: an indexable page names no canonical link — ${missing.join(", ")}`);
  }
  return entries.sort((a, b) => a.loc.localeCompare(b.loc));
}

const escapeXml = (text: string): string =>
  text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

/** The sitemap document, with each page's translations as `xhtml:link` alternates. */
export function sitemapXml(entries: SitemapEntry[]): string {
  const urls = entries.map((entry) => {
    const links = entry.alternates.map(
      (a) => `    <xhtml:link rel="alternate" hreflang="${escapeXml(a.hreflang)}" href="${escapeXml(a.href)}"/>`,
    );
    return [`  <url>`, `    <loc>${escapeXml(entry.loc)}</loc>`, ...links, `  </url>`].join("\n");
  });
  return [
    `<?xml version="1.0" encoding="UTF-8"?>`,
    `<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">`,
    ...urls,
    `</urlset>`,
    ``,
  ].join("\n");
}
