// foliate-js's zip reader, as its own rollup input declares it (rollup/zip.js), taken from the
// npm package instead of foliate's minified build of it. build.mjs maps `@zip.js/zip.js` to
// the package's lib/zip-core.js — the entry foliate bundles. See ../../VENDOR.md.
export { configure, ZipReader, BlobReader, TextWriter, BlobWriter } from "@zip.js/zip.js";
