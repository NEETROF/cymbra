# Spike 1.4 — the engine, its models and the server, from the app's `lingua-app://app/` page

Throwaway branch `spike/lingua-app-engine` (from `main` at e07d23bd3). Nothing on it is meant to
merge. Run on 2026-10-14 by an agent, on a Mac and on the iOS Simulator. **The real iPhone and iPad
runs are still to do** (steps at the end).

## What was built

- `apps/lingua-extension/src/app/spike/spike.{html,ts}`: one page that runs four phases by itself
  on load and prints every result on screen and to the app's console (`[spike] OK|KO|INFO …`).
- `probe-worker.ts`: a minimal classic worker (`importScripts`, a fetch of the engine's `.wasm` from
  the custom scheme, `WebAssembly.compile`, IndexedDB from a worker).
- The extension's **own** `engine-worker.ts` and `model-worker.ts`, bundled unchanged as classic
  IIFE workers into `dist-app/`, with `engine/bergamot-translator.{js,wasm}` and the bundled
  `model-manifest.json` (spike block at the end of the `app` target in `build.mjs`).
- `echo_server.py`: prints the `Origin` of every request and answers CORS with that origin echoed —
  what the server does once an origin is in `CYMBRA_ALLOWED_WEB_ORIGINS`.
- `ViewController.swift`: always opens `lingua-app://app/spike.html?<SPIKE_QUERY>` (env), adds a
  `spike` script handler that prints lines, logs the web content process id
  (`_webProcessIdentifier`), and makes the web view inspectable (Safari → Développement).

Platforms: macOS 26.5.2 on an M2 Max (the app signed with the Apple Development certificate, team
NEETROF, sandboxed as shipped); iOS Simulator iPhone 17e (iOS 26.5), iPad Air 11-inch M4 (iOS 26.5)
and iPhone 15 (**iOS 17.2, the app's minimum**). The simulators run on the Mac's CPU and memory:
their timings are not a phone's.

## 1. IndexedDB under `lingua-app://app/` — works everywhere

| | Mac | iPhone 17e sim | iPad Air sim | iPhone 15 sim (17.2) |
|---|---|---|---|---|
| `isSecureContext` | true | true | true | true |
| `storage.estimate()` quota | 19 660 MiB | 9 830 MiB | 9 830 MiB | 9 830 MiB |
| 64 MiB written (8 MiB records, one tx each) | 359 ms | 697 ms | 720 ms | 519 ms |
| real en-fr model (24.6 MiB gz → 35.0 MiB stored), download + gunzip + sha256 + put | 13.2 s ¹ | 1.3 s | 1.3 s | 1.0 s |
| usage after both | 99.0 MiB | 99.0 MiB | 99.0 MiB | 99.0 MiB |
| **after relaunch** (process killed, app started again) | survived, bytes verified, model `complete` | same | same | same |
| quota error | none | none — and **1 024 MiB** written in 5.5 s, then the model, no error | none | none |
| `storage.persisted()` | false | false | false | false |

¹ First fetch of the day from Cloudflare (`cf-cache-status: MISS`); the simulators came after, on a
warm cache.

The download is the extension's own code (`model-worker.ts` → `downloadModels`, the bytes checked
against the bundled catalogue) running in a worker of the page: `models.cymbra.app` answers
`access-control-allow-origin: *`, so a `lingua-app://app` page may read it. **That header is now a
dependency of the app** — keep it on the `cymbra-models` Pages project.

Where WebKit keeps it: the app's own container, `Library/WebKit/<bundle id>/WebsiteData/Default/
<origin hash>/<origin hash>/IndexedDB/<db hash>/IndexedDB.sqlite3` on iOS
(`Library/WebKit/WebsiteData/Default/…` in the sandbox container on macOS) — `Library`, not
`Library/Caches`. The origin is not persistent (`persisted()` false): WebKit may evict it under
storage pressure. The model is a cache the app can download again (the D7 offer), so that is
acceptable; whether `navigator.storage.persist()` is granted to an app's custom scheme is left to
the real-device run.

## 2. A classic worker with `importScripts` and the engine's WebAssembly — works everywhere

The probe worker, everywhere: `location = lingua-app://app/probe-worker.js`, `importScripts` ok,
`engine/bergamot-translator.wasm` served `200 application/wasm` by `AppPageResource`, compiled in
15–50 ms, IndexedDB open from the worker ok.

The extension's engine worker, `new Worker("engine-worker.js")`, unchanged: it reads the model from
`lingua-model` in the worker, instantiates Bergamot and translates with the `<b>` mark.

| | Mac | iPhone 17e sim | iPad Air sim | iPhone 15 sim (17.2) |
|---|---|---|---|---|
| cold `load` (worker start + importScripts + instantiate + model built from IndexedDB), first launch | 339 ms | 125 ms | 154 ms | 201 ms |
| cold `load`, next launch | 191 ms | 113 ms | 125 ms | 155 ms |
| first translation | 428 / 126 ms | 138 / 116 ms | 139 / 124 ms | 168 / 200 ms |
| warm translation | 26–31 ms ² | 25–31 ms | 26–31 ms | 28–35 ms |

² The Mac's first run read 112–276 ms warm, and 20 translations took 4 s while `footprint` sampled
the process each second (it inspects the task); without the sampler, 26–31 ms like the simulators.

Answers (identical on every platform, en-fr base-memory 2.0):

```
He finally decided to <b>give up</b> smoking after the doctor's warning.
  → Il a finalement décidé <b>d'arrêter</b> de fumer après l'avertissement du médecin.
The rope began to <b>slacken</b> as the boat drifted towards the shore.
  → La corde a commencé à <b>se relâcher</b> alors que le bateau dérivant vers le rivage.
She would <b>rather</b> stay at home than go to the party tonight.
  → Elle <b>préfère</b> rester à la maison plutôt que d'aller à la fête ce soir.
This phenomenon is <b>colloquially referred to as</b> a sugar rush.
  → Ce phénomène est <b>facilitamment appelé</b> une ruée <b>vers</b> le sucre.
```

A blob worker (`URL.createObjectURL`, with `importScripts`/`fetch` rebased on the page's URL) also
works (iPad sim: load 125 ms, 33–38 ms warm) — **not needed**: the classic worker from the custom
scheme works, so the extension's worker ships as it is.

## 3. Memory of the web content process

Measured from the host with `footprint -p <pid>` (phys_footprint) and `ps -o rss`, once a second,
the pid given by the app (`_webProcessIdentifier`; a simulator's WebContent is a host process).

| | Mac | iPhone 17e sim | iPad Air sim | iPhone 15 sim (17.2) |
|---|---|---|---|---|
| page alone, before the engine | 22–41 MB | 14–55 MB | 14–49 MB | 65 MB |
| peak during load + first translations | 387 MB | 309 MB | 321 MB | 332–367 MB |
| settled, engine + model held | 249–315 MB | 270–297 MB | 275–290 MB | 278–330 MB |
| RSS settled | 449–558 MB | 555–681 MB | 577–669 MB | 528–685 MB |

So the engine with one model costs **≈ 250–290 MB of footprint** in the web content process, in
line with the extension's own 195 MiB working set plus the page; the peak is the first
translation's heap growth. A second model (a pivot route) would add its ~35 MB plus buffers. No
jetsam on the simulators — which say nothing about a phone's limit: **the real-device run decides
this**, on the oldest iPhone available (4 GB of RAM is the worrying case).

## 4. A call from the page to the server: `Origin: lingua-app://app` — direct transport works

Every platform, every request: WebKit sends **`Origin: lingua-app://app`** (never `null`), with
`Sec-Fetch-Site: cross-site`, `Sec-Fetch-Mode: cors`, no `Referer`. A gRPC-web-shaped POST
(`content-type: application/grpc-web+proto`, `x-grpc-web`, `authorization`) is preflighted
(`OPTIONS`, `Access-Control-Request-Headers: authorization,content-type,x-grpc-web`) with the same
origin, and **CORS passes** when the server echoes it (local echo server, Mac and the three
simulators). `https://httpbin.org/anything` reports the same `Origin` (reachable from a device).

Echo server log (Mac):

```
[echo] GET /echo Origin='lingua-app://app' Referer=None UA='Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) …'
[echo] OPTIONS /echo Origin='lingua-app://app' Referer=None …
[echo]   preflight ACRM='POST' ACRH='authorization,content-type,x-grpc-web'
[echo] POST /echo Origin='lingua-app://app' Referer=None …
```

The production server, today: the page's unauthenticated `OnlineTranslationService/Translate`
fails with `ConnectError: [unknown] Load failed` on every platform — a CORS refusal, not the
server: from `curl`, the preflight for `Origin: lingua-app://app` gets no
`access-control-allow-origin` (as for any unlisted origin), the one for `https://bo.cymbra.app`
does, and the POST itself reaches tonic (`grpc-status: 16`, `missing bearer token`). The server
matches origins exactly (`AllowOrigin::list`, `backend/server/src/main.rs`), and
`lingua-app://app` is a valid `HeaderValue`.

## Real iPhone 15 Pro Max (iPhone16,2, 8 GB, iOS 27.2) — 2026-10-14

Built for the device (`Cymbra Lingua (iOS)`, Debug, automatic signing, team NEETROF), installed
over the App Store app with `xcrun devicectl device install app`, launched with
`devicectl device process launch --console` (no trust or certificate prompt). The page now also
calls `navigator.storage.persist()`.

| | Launch 1 (empty) | Launch 2 (process killed by `--terminate-existing`, relaunched) |
|---|---|---|
| `storage.estimate()` quota | 9 830 MiB | 9 830 MiB |
| `persisted()` / `persist()` | false / **false** (refused, no prompt) | false / false |
| 64 MiB IndexedDB fill | 578 ms | — |
| en-fr model download + verify + store | 1.3 s (Wi-Fi) | — |
| `idb.relaunch` | — | **survived, verified, 64.0 MiB**; model `complete` |
| probe worker | importScripts ok, wasm `200 application/wasm`, compiled in 18 ms, IDB ok | same, 22 ms |
| engine cold `load` | 107 ms | 172 ms |
| first translation | 129 ms | 136 ms |
| warm translations | 25–30 ms | 25–29 ms; 40 in 786 ms (≈ 20 ms each) |
| `net.httpbin` Origin | `lingua-app://app` | `lingua-app://app` |
| `net.prod` | `Load failed` (CORS, as on the simulators) | same |

The `<b>` marks came back exactly as on the Mac and simulators. `navigator.hardwareConcurrency`
reads 4.

Memory (Instruments **Activity Monitor**, `xcrun xctrace record --device … --all-processes`,
one sample a second, launch 2 with 40 translations): the WebContent process (pid 4017) was at
**347 MiB** footprint at the first sample after the translations started (the peak may have been
slightly higher between samples), **289–292 MiB settled** with the engine and model held (real
memory 387 MiB); the app process itself 10–16 MiB. No jetsam kill. The echo-server lines fail on
the device by construction (`127.0.0.1` is the phone itself).

**Not yet done on the device:** survival across a device restart (needs Guillaume), an older
iPhone (4 GB) for the memory limit, and the iPad (unavailable).

## Recommendations

- **D7 — IndexedDB, in the page's origin, as designed.** No native download, no `AppPageResource`
  serving models. Ship `engine-worker.ts` and `model-worker.ts` unchanged as classic workers in
  `dist-app` (the spike's build block is the shape of task 5.2), the engine files served from the
  embedded extension (task 5.3) — the spike copied them into `dist-app`; `AppPageResource.mimeType`
  already types `.wasm` right. Keep `access-control-allow-origin: *` on `models.cymbra.app`.
  Treat the models as evictable (re-offer the download when they are gone).
- **D8 — direct transport.** The page's gRPC-web calls carry `Origin: lingua-app://app`; add it to
  `CYMBRA_ALLOWED_WEB_ORIGINS` (task 5.5). The bridge transport through `URLSession` is not needed.
- **Engine, not online-only** — unless the real devices show a jetsam kill or a translation slower
  than `TRANSLATION_WAIT_MS` on the oldest phone.

## What remains to verify on a real iPhone and iPad (and why the simulator cannot)

1. Memory: the web content process's limit on a phone (jetsam) — the simulator has none.
2. Speed: cold load and translation on a phone's CPU.
3. Survival across a real relaunch *and* a device restart, and whether the app's data survives a
   low-storage cleanup (iOS may evict non-persistent website data).
4. `navigator.storage.persist()` — granted or not to an app's custom scheme.
5. The `Origin` from a device (httpbin line) — expected identical.

## Repeating the spike on a real iPhone and iPad

1. Check out the branch and build the web side (production backend, or the device build is refused):
   ```bash
   git fetch origin && git switch spike/lingua-app-engine
   cd apps/lingua-extension
   env -u GITHUB_TOKEN yarn install && env -u GITHUB_TOKEN yarn fetch:engine
   yarn gen:wasm && yarn gen:proto && yarn gen:pack   # if not already there
   LINGUA_GRPC_WEB_URL=https://api.cymbra.app yarn build:safari
   yarn build:app                                    # carries spike.html, the workers, the engine
   ```
2. Open `apps/lingua-apple/Cymbra Lingua.xcodeproj`, scheme **Cymbra Lingua (iOS)**, choose the
   iPhone, Run (team NEETROF signs automatically). The app opens straight on « Lingua app — spike
   1.4 » — it does not need the extension's shared state. Use the oldest iPhone you have.
3. Read the page as it runs (≈ 5 s on Wi-Fi). Every line should be green; note the red ones.
   The lines to copy: `estimate.*`, `idb.fill`, `model.download`, `worker.probe`,
   `engine.classic.load`, the four `engine.classic.translate`, `net.httpbin` (its `Origin`),
   `net.prod` (red until the server lists the origin — expected).
   For a full log: Safari on the Mac → Développement → <device> → Cymbra Lingua → spike.html
   (Console), or Xcode's console (`[spike] …` lines).
4. Memory: in Xcode, Open Developer Tool → Instruments → **Activity Monitor**, target the device,
   record, filter `WebContent`. In the app tap **Translate ×20 (memory)** (twice) and read the
   WebContent row's *Memory* column at its peak and once settled.
5. Relaunch: swipe the app away in the app switcher, open it again from the home screen. The second
   line must read `idb.relaunch: {"survived":true,"verified":true,"stored":"64.0 MiB"…}` and
   `model.stored: en-fr already in lingua-model`, and `engine.classic.load` then times a load with
   no download. Then restart the device and open the app once more: same lines expected.
6. Optional quota probe: tap **IDB +100 MiB** a few times, watch for a red `idb.fill` (a quota
   error names how much was stored).
7. Repeat 2–5 on the iPad, and once on the Mac with scheme **Cymbra Lingua (macOS)**.
8. Clean up: tap **Wipe spike data** (deletes `spike-fill` and `lingua-model`), then delete the app
   (or reinstall a normal build) — this build always opens the spike page.

Optional, to see the Origin a server receives from the device itself: on the Mac,
`python3 apps/lingua-extension/src/app/spike/echo_server.py 8765 0.0.0.0`, then in Xcode → Edit
Scheme → Run → Environment Variables, `SPIKE_QUERY` = `echo=http://<mac's LAN IP>:8765`. An `http://`
address may be refused from the secure `lingua-app://` page (mixed content) or by Local Network
privacy; httpbin's line already shows the Origin over HTTPS.
