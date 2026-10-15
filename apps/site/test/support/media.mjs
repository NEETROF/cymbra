// What the Lingua page's byte pin leaves aside (change: add-site-lingua-showcase): the addresses of
// its images — in `src`, `srcset`, `poster`, and the `href` of a figure's link to its larger size —
// which the build hashes (`/_astro/card.<hash>.webp`) and the Container API names by
// their source path on this machine (`/_image?href=…`), and the bodies of its scripts, bundled by
// the build and referenced by their source in the Container. Everything else in `<main>` — the
// markup, the alt texts, the captions, the video's attributes and address — stays pinned.

/** `html` with every image address and every script body replaced by a placeholder. */
export function withoutMediaUrls(html) {
  return html
    .replace(/<script\b[^>]*>[\s\S]*?<\/script[^>]*>/gi, "<script></script>")
    .replace(/ (src|srcset|poster|href)="([^"]*)"/g, (whole, name, value) =>
      /(^|[\s,])\/_(astro|image)\b/.test(value) ? ` ${name}="[image]"` : whole,
    );
}

/** The inner HTML of a page's `<main>`, its media addresses left aside. */
export function pinnedMain(html) {
  const main = html.match(/<main>(.*?)<\/main>/s)?.[1];
  return main === undefined ? undefined : withoutMediaUrls(main);
}
