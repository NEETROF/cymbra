import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import sharp from "sharp";
import { describe, expect, it } from "vitest";

// Run after `yarn build` (see `vitest.build.config.ts`). What search engines and link previews read
// on every built page (change: add-site-lingua-showcase, site-discoverability): the canonical
// address and `og:url` of every indexable page, in the form the host serves, and none on a personal
// page; a 1200×630 share image at an absolute address, with its alt text, on every page; the sitemap
// and the robots file. Nothing here asks the site's helpers what to expect: it reads `dist/`.

const dist = resolve(__dirname, "../../dist");
const SITE = "https://cymbra.app";

/** Every built page, relative to `dist/`. */
function builtPages(dir = dist): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) return builtPages(path);
    return entry.name.endsWith(".html") ? [relative(dist, path)] : [];
  });
}
const read = (file: string) => readFileSync(resolve(dist, file), "utf8");
const headOf = (html: string) => html.match(/<head>(.*?)<\/head>/s)?.[1] ?? "";
/** A `<meta>`'s content, by its `property` or `name`; null when the head has none. */
function meta(head: string, key: string): string | null {
  for (const [, , name, content] of head.matchAll(/<meta (property|name)="([^"]*)" content="([^"]*)">/g)) {
    if (name === key) return content;
  }
  return null;
}
const canonicalOf = (head: string) => head.match(/<link rel="canonical" href="([^"]*)">/)?.[1] ?? null;
const noindex = (head: string) => head.includes('<meta name="robots" content="noindex">');
/** The address the host serves for a page written at `file` (`lingua/index.html` → `/lingua/`). */
const servedFor = (file: string) => `${SITE}/${file.replace(/index\.html$/, "")}`;

/** A JPEG's size, from its first start-of-frame marker. */
function jpegSize(bytes: Buffer): { width: number; height: number } | null {
  if (bytes[0] !== 0xff || bytes[1] !== 0xd8) return null;
  let at = 2;
  while (at + 9 < bytes.length) {
    if (bytes[at] !== 0xff) return null;
    const marker = bytes[at + 1];
    const length = bytes.readUInt16BE(at + 2);
    if (marker >= 0xc0 && marker <= 0xcf && ![0xc4, 0xc8, 0xcc].includes(marker)) {
      return { height: bytes.readUInt16BE(at + 5), width: bytes.readUInt16BE(at + 7) };
    }
    at += 2 + length;
  }
  return null;
}

const pages = builtPages();

describe("canonical addresses", () => {
  it("has pages to check", () => {
    expect(pages.length).toBeGreaterThan(20);
  });

  it("every indexable page names its own served address, as canonical and og:url; a noindex page names none", () => {
    const wrong: string[] = [];
    for (const page of pages) {
      const head = headOf(read(page));
      const canonical = canonicalOf(head);
      if (noindex(head)) {
        if (canonical !== null || meta(head, "og:url") !== null) wrong.push(`${page}: noindex, yet a canonical or og:url`);
        continue;
      }
      if (canonical !== servedFor(page)) wrong.push(`${page}: canonical ${canonical}, expected ${servedFor(page)}`);
      if (meta(head, "og:url") !== canonical) wrong.push(`${page}: og:url ${meta(head, "og:url")}`);
    }
    expect(wrong).toEqual([]);
  });

  it("every hreflang names the served form: a trailing slash, at cymbra.app", () => {
    const wrong = pages.flatMap((page) =>
      [...headOf(read(page)).matchAll(/<link rel="alternate" hreflang="[a-z]+" href="([^"]*)">/g)]
        .map((m) => m[1])
        .filter((href) => !href.startsWith(`${SITE}/`) || !href.endsWith("/"))
        .map((href) => `${page}: ${href}`),
    );
    expect(wrong).toEqual([]);
  });

  it("the Lingua page, a legal page and a personal page, as the spec says", () => {
    const lingua = headOf(read("lingua/index.html"));
    expect(canonicalOf(lingua)).toBe(`${SITE}/lingua/`);
    expect(lingua).toContain(`<link rel="alternate" hreflang="en" href="${SITE}/en/lingua/">`);
    const privacy = headOf(read("en/privacy/index.html"));
    expect(canonicalOf(privacy)).toBe(`${SITE}/en/privacy/`);
    expect(privacy).toContain(`<link rel="alternate" hreflang="fr" href="${SITE}/confidentialite/">`);
    const account = headOf(read("account/index.html"));
    expect(noindex(account)).toBe(true);
    expect(canonicalOf(account)).toBeNull();
  });
});

describe("share previews", () => {
  it("every page has a 1200×630 JPEG og:image at an absolute address, its alt text, a site name and a large card", () => {
    const wrong: string[] = [];
    for (const page of pages) {
      const head = headOf(read(page));
      const image = meta(head, "og:image");
      if (!image?.startsWith(`${SITE}/`)) {
        wrong.push(`${page}: og:image ${image}`);
        continue;
      }
      const file = resolve(dist, `.${new URL(image).pathname}`);
      const size = existsSync(file) ? jpegSize(readFileSync(file)) : null;
      if (size?.width !== 1200 || size.height !== 630) wrong.push(`${page}: ${image} is ${JSON.stringify(size)}`);
      if (meta(head, "og:image:width") !== "1200" || meta(head, "og:image:height") !== "630") wrong.push(`${page}: size metas`);
      if (!meta(head, "og:image:alt")) wrong.push(`${page}: no og:image:alt`);
      if (meta(head, "og:site_name") !== "Cymbra") wrong.push(`${page}: og:site_name`);
      if (meta(head, "twitter:card") !== "summary_large_image") wrong.push(`${page}: twitter:card`);
    }
    expect(wrong).toEqual([]);
  });

  it("the Lingua and Music pages show their own capture, every other page the brand image", () => {
    const imageOf = (page: string) => new URL(meta(headOf(read(page)), "og:image")!).pathname;
    for (const page of ["lingua/index.html", "en/lingua/index.html"]) expect(imageOf(page)).toMatch(/^\/_astro\/share\./);
    for (const page of ["music/index.html", "en/music/index.html", "es/music/index.html"]) {
      expect(imageOf(page)).toMatch(/^\/_astro\/share\./);
    }
    expect(imageOf("es/privacidad/index.html")).toMatch(/^\/_astro\/brand-share\./);
    expect(imageOf("index.html")).toMatch(/^\/_astro\/brand-share\./);
    // Each page's alt text is in its language.
    expect(meta(headOf(read("es/privacidad/index.html")), "og:image:alt")).toBe("El nombre Cymbra sobre un degradado azul y violeta");
  });

  it("each Lingua and Music page shares the capture of its own language", async () => {
    // The built image is its source cut as the head cuts it (1200×630 from the top) and encoded
    // again, so it is compared by its look: each page's image, reduced to grey pixels, is nearest its
    // own language's source among the three.
    const look = async (input: string | Buffer) => {
      const cut = await sharp(input).resize(1200, 630, { fit: "cover", position: "top" }).png().toBuffer();
      return sharp(cut).resize(150, 79, { fit: "fill" }).greyscale().raw().toBuffer();
    };
    const distance = (a: Buffer, b: Buffer) => a.reduce((sum, value, i) => sum + Math.abs(value - b[i]), 0);
    const site = resolve(__dirname, "../..");
    const sets = {
      lingua: {
        pages: { fr: "lingua/index.html", en: "en/lingua/index.html", es: "es/lingua/index.html" },
        source: (lang: string) => resolve(site, `src/assets/lingua/${lang}/share.jpg`),
      },
      music: {
        pages: { fr: "music/index.html", en: "en/music/index.html", es: "es/music/index.html" },
        source: (lang: string) => resolve(site, `src/assets/music/${lang}/share.png`),
      },
    };
    for (const [name, { pages: byLang, source }] of Object.entries(sets)) {
      const sources: Record<string, Buffer> = Object.fromEntries(
        await Promise.all(Object.keys(byLang).map(async (lang) => [lang, await look(source(lang))] as const)),
      );
      for (const [lang, page] of Object.entries(byLang)) {
        const image = resolve(dist, `.${new URL(meta(headOf(read(page)), "og:image")!).pathname}`);
        const built = await look(readFileSync(image));
        const nearest = Object.entries(sources).sort(([, a], [, b]) => distance(built, a) - distance(built, b))[0][0];
        expect(nearest, `${name} ${page}`).toBe(lang);
      }
    }
  });
});

describe("the sitemap and the robots file", () => {
  const sitemap = read("sitemap.xml");
  const urls = [...sitemap.matchAll(/<url>(.*?)<\/url>/gs)].map((m) => ({
    loc: m[1].match(/<loc>([^<]*)<\/loc>/)?.[1],
    alternates: [...m[1].matchAll(/<xhtml:link rel="alternate" hreflang="([a-z]+)" href="([^"]*)"\/>/g)].map((a) => [a[1], a[2]]),
  }));

  it("lists exactly the built pages without noindex, at their served address", () => {
    const indexable = pages.filter((page) => !noindex(headOf(read(page)))).map(servedFor);
    expect(urls.map((u) => u.loc).sort()).toEqual(indexable.sort());
    for (const personal of ["account/", "redeem/", "checkout/", "en/404/"]) {
      expect(sitemap, personal).not.toContain(`<loc>${SITE}/${personal}</loc>`);
    }
  });

  it("gives each page the translations its head names", () => {
    for (const { loc, alternates } of urls) {
      const page = `${loc!.slice(SITE.length + 1)}index.html`;
      const hreflang = [...headOf(read(page)).matchAll(/<link rel="alternate" hreflang="([a-z]+)" href="([^"]*)">/g)].map((m) => [m[1], m[2]]);
      expect(alternates, loc).toEqual(hreflang);
    }
  });

  it("the robots file allows every agent, disallows nothing and names the sitemap", () => {
    const robots = read("robots.txt");
    expect(robots).toMatch(/^User-agent: \*$/m);
    expect(robots).not.toMatch(/^Disallow:\s*\S/m);
    expect(robots).toMatch(/^Sitemap: https:\/\/cymbra\.app\/sitemap\.xml$/m);
  });
});
