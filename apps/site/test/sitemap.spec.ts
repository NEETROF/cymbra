import { describe, expect, it } from "vitest";
import { servedUrl } from "../src/lib/served-url";
import { readPageHead, sitemapEntries, sitemapXml } from "../src/lib/sitemap";

// The served address (change: add-site-lingua-showcase, D12) and the sitemap built from the pages'
// heads (D13). The built site itself is checked after the build (`test/post-build/head.spec.ts`).

describe("servedUrl", () => {
  it("names the address the host serves: the path with a trailing slash", () => {
    expect(servedUrl("/")).toBe("https://cymbra.app/");
    expect(servedUrl("/lingua")).toBe("https://cymbra.app/lingua/");
    expect(servedUrl("/en/")).toBe("https://cymbra.app/en/");
    expect(servedUrl("/es/privacidad")).toBe("https://cymbra.app/es/privacidad/");
  });

  it("refuses what is not a site path", () => {
    expect(() => servedUrl("lingua")).toThrow(/not a site path/);
  });
});

const page = (head: string) => `<!DOCTYPE html><html lang="fr"><head><meta charset="utf-8">${head}</head><body></body></html>`;
const LINGUA = page(
  '<link rel="canonical" href="https://cymbra.app/lingua/"><link rel="alternate" hreflang="fr" href="https://cymbra.app/lingua/"><link rel="alternate" hreflang="en" href="https://cymbra.app/en/lingua/">',
);
const ACCOUNT = page('<meta name="robots" content="noindex"><link rel="alternate" hreflang="fr" href="https://cymbra.app/account/">');

describe("the sitemap", () => {
  it("reads a page's robots meta, canonical and alternates", () => {
    expect(readPageHead(LINGUA)).toEqual({
      noindex: false,
      canonical: "https://cymbra.app/lingua/",
      alternates: [
        { hreflang: "fr", href: "https://cymbra.app/lingua/" },
        { hreflang: "en", href: "https://cymbra.app/en/lingua/" },
      ],
    });
    expect(readPageHead(ACCOUNT).noindex).toBe(true);
  });

  it("lists every page but the noindex ones, sorted by address", () => {
    const home = page('<link rel="canonical" href="https://cymbra.app/">');
    const entries = sitemapEntries([
      { file: "lingua/index.html", html: LINGUA },
      { file: "account/index.html", html: ACCOUNT },
      { file: "index.html", html: home },
    ]);
    expect(entries.map((e) => e.loc)).toEqual(["https://cymbra.app/", "https://cymbra.app/lingua/"]);
  });

  it("fails on an indexable page without a canonical, naming it", () => {
    expect(() => sitemapEntries([{ file: "new/index.html", html: page("") }])).toThrow(/new\/index\.html/);
  });

  it("writes each page with its translations as xhtml:link alternates", () => {
    const xml = sitemapXml(sitemapEntries([{ file: "lingua/index.html", html: LINGUA }]));
    expect(xml).toContain('<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">');
    expect(xml).toContain(
      "  <url>\n    <loc>https://cymbra.app/lingua/</loc>\n" +
        '    <xhtml:link rel="alternate" hreflang="fr" href="https://cymbra.app/lingua/"/>\n' +
        '    <xhtml:link rel="alternate" hreflang="en" href="https://cymbra.app/en/lingua/"/>\n  </url>',
    );
    expect(xml).not.toContain("lastmod");
  });
});
