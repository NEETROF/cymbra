// The output folder build.mjs writes a variant into (`LINGUA_DIST_DIR`, add-site-lingua-showcase
// D1). The build empties that folder first (`rmSync` with `recursive`), so a mistyped value — `.`,
// `..`, `src` — would delete the app or the repository: the option takes a folder named `dist-…`
// (`dist-capture/`, the capture tool's; `dist-plain-…`, its temporary comparison build), never the
// app's folder or one holding it. Pure.
import { basename, resolve, sep } from "node:path";

/** What is wrong with `value` as the output folder of a build run in `appDir`; null when it will do. */
export function distDirProblem(appDir, value) {
  const app = resolve(appDir);
  const dir = resolve(app, value);
  if (dir === app || app.startsWith(dir.endsWith(sep) ? dir : `${dir}${sep}`)) {
    return `LINGUA_DIST_DIR (${value}) is the app's folder or one holding it: the build would empty it`;
  }
  if (!/^dist-[\w.-]+$/.test(basename(dir))) {
    return `LINGUA_DIST_DIR (${value}) must name a folder called dist-… (dist-capture, for instance): the build empties it first`;
  }
  return null;
}
