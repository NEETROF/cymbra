## Context

- 46 `debugPrint` sites in `apps/music/lib`, several `catch (_)`; no logger, no
  levels. Release builds on iOS/macOS emit nothing for `print`/`debugPrint`
  (verified 2026-08-17 on the release build); Android emits to logcat only for
  native prints. The Rust engine has its own platform log (`api/platform_log.rs`:
  liblog on Android, stderr elsewhere) for the audio/MIDI subsystems.
- The app already uses method channels (`audio_routing_service.dart`, `audio_service.dart`)
  with an injectable channel seam — the same pattern applies.
- Constraint: no third-party diagnostics service.

## Goals / Non-Goals

**Goals**
- Any surviving failure leaves a trace readable on a TestFlight / store build without
  a debugger.
- A tester can hand over the last minutes of the app's journal in two taps.
- No secret ever reaches a log line, by construction.

**Non-Goals**
- Remote shipping / crash reporting; backend correlation; changing the Rust engine's
  log path; a log viewer UI beyond copy/share.

## Decisions

### D1 — `package:logging`, hierarchical, one root
`Logger.root` is left alone; the app owns `Logger('cymbra')` (in `app_log.dart`) and
areas take children (`Logger('cymbra.auth')`). Levels: `FINE` = dev noise (dropped in
release), `INFO` = milestones (sign-in done, purchase reported), `WARNING` = handled
failure with cause, `SEVERE` = unexpected (also `FlutterError.onError` /
`PlatformDispatcher.onError`). *Alternative*: `logger`/`talker` packages — more UI, more
deps; not needed.

### D2 — Sinks: console in debug, platform log in release, ring buffer always
`installAppLogging()` in `main.dart` (before `runApp`) attaches one listener:
- `kDebugMode` → `debugPrint` formatted line (unchanged developer experience);
- else → `PlatformLogChannel.write(level, category, message)` → `os_log` (iOS/macOS,
  subsystem `app.cymbra.music`, category = logger name; `.error` for SEVERE, `.default`
  for WARNING/INFO) / `Log.println` (Android, tag `cymbra.<area>`) / stderr (desktop);
- both → the ring buffer (`INFO`+, 500 lines, `[HH:mm:ss.SSS] LEVEL area: message
  (error)`).
The channel is a provider seam (`platformLogChannelProvider`) with a fake in tests; a
failing channel never throws (log must never crash the app).

### D3 — Redaction in the sink, not at call sites
`redact(String)`: bearer tokens (`Bearer …`), JWT-shaped tokens (`xxx.yyy.zzz` base64url
≥ 3 segments), e-mail addresses (`***@domain`), and anything matching the access-code
shape (`XXXX-XXXX-…`) → replaced before formatting. Call sites still must not log
secrets on purpose; the sink is the belt. Unit-tested with fixtures.

### D4 — Diagnostic journal on the Help screen
`Help → Diagnostic log`: header (app version/build, platform/OS, locale, plan kind —
never the user id/e-mail), then the ring buffer; **Copy** (clipboard) and **Share**
(`share_plus` already a dependency? if not, clipboard only). No upload. The `INFO`+
threshold keeps it short; a "include debug detail" toggle is out of scope.

### D5 — Lint gate
`avoid_print` on in `analysis_options.yaml`; `debugPrint(` banned in `lib/` by a
`flutter.yml` step (`! grep -rn "debugPrint(" lib/`) with an allow-list of the one sink
file. Migration replaces each site with the area logger; `catch (_)` that intentionally
swallow get a `FINE`/`WARNING` line naming why.

### D6 — Rust engine untouched
`platform_log.rs` stays; a follow-up may route it through the same subsystem/tag for a
single Console.app filter.

### D7 — Testing
Unit: level routing per build mode (inject `isDebug`), redaction fixtures, ring buffer
capacity/order, channel failure swallowed. Widget: Help screen shows the journal, Copy
puts the header + lines on the clipboard (Clipboard mock). Existing tests unaffected
(fake channel default in the harness).

## Risks / Trade-offs
- **PII leaks via free-text messages** → redaction belt + review rule "log identifiers,
  not payloads"; the journal is opt-in share.
- **Log volume on Android** → `INFO`+ only in release; `FINE` dropped before the channel.
- **Method channel before Flutter is ready** → the sink buffers until the engine
  reports ready (first frame), then flushes; ring buffer works from process start.

## Migration Plan
1. Add logger + sink + channel + native handlers (dark: no call sites yet).
2. Migrate `debugPrint` sites area by area (auth/plans first — the ones that bit).
3. Enable the lint gate.
4. Help screen journal + README.

## Open Questions
- Share sheet vs clipboard only (depends on `share_plus` presence — check at
  implementation; clipboard is the minimum).
