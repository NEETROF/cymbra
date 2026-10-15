// The address the host serves for a page (change: add-site-lingua-showcase, D12). Cloudflare Pages
// answers `/lingua` with a 308 to `/lingua/`: every page is a folder (`build.format: 'directory'`),
// so its served address ends with a slash. The canonical link, `og:url`, every `hreflang` and the
// sitemap name that one form; the header's language switch keeps the declared paths, as every other
// internal link does.

/** The site's origin, as `astro.config.mjs` declares it. */
export const SITE = "https://cymbra.app";

/** `path`, as a page declares it (`/lingua`, `/en/`, `/`), as the address the host serves. */
export function servedUrl(path: string): string {
  if (!path.startsWith("/")) throw new Error(`servedUrl: "${path}" is not a site path`);
  const trimmed = path.replace(/\/+$/, "");
  return `${SITE}${trimmed}/`;
}
