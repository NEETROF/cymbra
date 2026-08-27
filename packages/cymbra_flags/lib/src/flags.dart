import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import 'flag_cache.dart';
import 'flag_service.dart';
import 'flag_snapshot.dart';

part 'flags.g.dart';

// --- input seams the host app overrides ------------------------------------

/// The shared gRPC channel. The host app MUST override this with its channel
/// (e.g. `flagChannelProvider.overrideWith((ref) => ref.watch(cymbraChannelProvider))`).
@Riverpod(keepAlive: true)
ClientChannel flagChannel(Ref ref) =>
    // coverage:ignore-line
    throw UnimplementedError(
      'override flagChannelProvider with the app gRPC channel',
    );

/// The app scope keys resolve for. Defaults to `music`; a `live` app overrides.
@Riverpod(keepAlive: true)
String flagApp(Ref ref) =>
    const String.fromEnvironment('CYMBRA_FLAG_APP', defaultValue: 'music');

/// The current account id (or `null` when signed out). The host app overrides
/// this with its session identity so the snapshot resets on sign-out / user
/// switch and the persisted cache is keyed per identity.
///
/// **The account, and nothing else.** Changing it *scraps* the snapshot: every
/// flag falls back to its code default until a network round trip lands, which
/// is right for "this is a different person" and wrong for anything else. What
/// merely changes the *answers* for the same person — a plan, an enrolment —
/// belongs in [flagAudienceProvider], which re-evaluates without blanking.
@Riverpod(keepAlive: true)
String? flagIdentity(Ref ref) => null;

/// Everything about the caller *other than who they are* that the server's
/// evaluation depends on: the plan, the beta campaigns they are enrolled in…
/// Overridden by the host app; the default `''` means "nothing else matters".
///
/// A change here **refreshes** the snapshot ([Flags.refresh]) — it never resets
/// it. That distinction is the fix for a beta-only entry point vanishing
/// mid-session: with the plan folded into the identity, a purchase, a plan
/// refetch or a beta enrolment rebuilt the notifier, and every flag read
/// `false` for as long as the refetch took — long enough for a whole home
/// screen to lose its drums section and get it back.
@Riverpod(keepAlive: true)
String flagAudience(Ref ref) => '';

/// Supplies the current bearer token (`null` ⇒ anonymous). The host app overrides
/// this to read its token store.
@Riverpod(keepAlive: true)
FlagBearer flagBearer(Ref ref) => const AnonymousFlagBearer();

/// How often to poll in the foreground (version-guarded, cheap). `null` disables
/// polling (used in tests). Design default ~10 minutes.
@Riverpod(keepAlive: true)
Duration? flagPollInterval(Ref ref) => const Duration(minutes: 10);

/// Production fetch service. Override in tests with a fake.
@Riverpod(keepAlive: true)
FlagService flagService(Ref ref) =>
    // coverage:ignore-line
    GrpcFlagService(ref.watch(flagChannelProvider));

/// The persisted-cache KV seam. Override to reuse the app's own store.
@Riverpod(keepAlive: true)
FlagPreferences flagPreferences(Ref ref) =>
    // coverage:ignore-line
    SharedPreferencesFlagPreferences();

@Riverpod(keepAlive: true)
FlagCache flagCache(Ref ref) => FlagCache(ref.watch(flagPreferencesProvider));

/// A bearer-token source seam.
abstract class FlagBearer {
  /// The bearer to present, or `null` when the caller is genuinely signed out.
  Future<String?> token();

  /// Ask the host app to renew the session after the server refused [token],
  /// and return the fresh bearer — `null` when there is none to be had (signed
  /// out, or the renewal failed).
  ///
  /// The flag read is **optional-auth**: a caller with no token is a legitimate
  /// anonymous caller. That makes a *stale* token uniquely dangerous, because
  /// it used to be answered the same way — with an anonymous set carrying a
  /// perfectly valid version, which the client stored, losing every
  /// entitlement-gated key until some later poll happened to run after another
  /// RPC had refreshed the token. The server now refuses a stale bearer
  /// outright, and this is the other half: the cue to renew and retry.
  ///
  /// Defaults to "no renewal available", which is correct for the anonymous
  /// bearer and for any host that has no session to renew.
  Future<String?> renewed() async => null;
}

/// The default anonymous bearer (no token).
class AnonymousFlagBearer implements FlagBearer {
  const AnonymousFlagBearer();
  @override
  Future<String?> token() async => null;
  @override
  Future<String?> renewed() async => null;
}

// --- the flag client notifier ----------------------------------------------

/// Holds the caller's effective [FlagSnapshot]. Feature entry points read keys
/// synchronously off `ref.watch(flagsProvider)`. Fetches on launch (build) and
/// on resume (via [Flags.refresh]); stale-while-revalidate keeps the last-good
/// snapshot on a failed refresh; identity-scoped, so it resets on sign-out and
/// never inherits another user's set on a switch.
@Riverpod(keepAlive: true)
class Flags extends _$Flags {
  @override
  FlagSnapshot build() {
    final app = ref.watch(flagAppProvider);
    // Watching the identity makes the notifier rebuild — and thus reset — on
    // sign-out / user switch, so the previous identity's snapshot is never reused.
    final identity = ref.watch(flagIdentityProvider);

    // The audience is LISTENED to, never watched: a plan or beta change makes
    // the server evaluate a different set for the same person, so the snapshot
    // is refetched in place — stale-while-revalidate, exactly like the poll —
    // rather than scrapped and rebuilt from empty.
    ref.listen(flagAudienceProvider, (previous, next) {
      if (previous != next) unawaited(refresh());
    });

    final poll = ref.watch(flagPollIntervalProvider);
    if (poll != null) {
      final timer = Timer.periodic(poll, (_) => unawaited(refresh()));
      ref.onDispose(timer.cancel);
    }

    // Hydrate + fetch after build returns (never touch `state` synchronously).
    Future.microtask(bootstrap);
    // Flicker-free-ish cold start: begin on code defaults (empty snapshot).
    return FlagSnapshot.empty(app, identity);
  }

  bool _isCurrent(String app, String? identity) =>
      state.app == app && state.identity == identity;

  /// Load the persisted per-identity snapshot (if any), then refresh from the
  /// network. Called once on build; safe to call again.
  Future<void> bootstrap() async {
    final app = state.app;
    final identity = state.identity;
    final cached = await ref.read(flagCacheProvider).read(app, identity);
    // Only apply if we're still on the same identity and haven't already fetched.
    if (cached != null && _isCurrent(app, identity) && state.version.isEmpty) {
      state = cached;
    }
    await refresh();
  }

  /// Fetch the latest set (stale-while-revalidate). On a changed set: swap
  /// atomically + persist. On "unchanged": keep. On any error: keep the last-good
  /// snapshot (the cache is presentation-only; the backend is authoritative).
  ///
  /// Two rules protect a signed-in caller's set from being replaced by a weaker
  /// one (change: add-drum-input-mapping — beta fix). The snapshot carries no
  /// evidence of *how* it was evaluated — flags and a content hash, nothing
  /// else — so an anonymous answer is indistinguishable from a real one once it
  /// is stored, and it silently removes every entitlement-gated key:
  ///
  /// * a snapshot scoped to an identity is **never fetched without a bearer**;
  /// * a bearer the server refuses is **renewed once and retried**, rather than
  ///   letting the read fall back to whatever the server evaluates without it.
  Future<void> refresh() async {
    final app = state.app;
    final identity = state.identity;
    try {
      var bearer = await ref.read(flagBearerProvider).token();
      // Signed in with nothing to present — the token store has not caught up
      // with the session yet, or the stored pair was cleared. Renew rather than
      // ask anonymously: the answer to an anonymous question is a set for
      // nobody, and this snapshot belongs to someone.
      if (identity != null && (bearer == null || bearer.isEmpty)) {
        bearer = await _renew();
        if (bearer == null) return;
      }
      var fetch = await _fetch(app, identity, bearer);
      if (fetch == null) {
        // The bearer was refused. Renew ONCE and retry; a renewal that yields
        // nothing leaves the snapshot untouched rather than asking anonymously.
        final renewed = await _renew();
        if (renewed == null) return;
        fetch = await _fetch(app, identity, renewed);
        if (fetch == null) return;
      }
      // Never apply a fetch that resolved for a now-stale identity.
      if (!_isCurrent(app, identity)) return;
      final snap = fetch.snapshot;
      if (fetch.unchanged || snap == null) return;
      state = snap;
      await ref.read(flagCacheProvider).write(snap);
    } catch (_) {
      // Keep the last-good snapshot (never drop to an empty/gap state).
    }
  }

  /// A renewed bearer, or `null` when there is none to be had.
  Future<String?> _renew() async {
    final renewed = await ref.read(flagBearerProvider).renewed();
    return (renewed == null || renewed.isEmpty) ? null : renewed;
  }

  /// One fetch attempt: the result, or `null` when the bearer was refused.
  Future<FlagFetch?> _fetch(String app, String? identity, String? bearer) async {
    try {
      return await ref
          .read(flagServiceProvider)
          .fetch(
            app: app,
            identity: identity,
            knownVersion: state.version,
            bearer: bearer,
          );
    } on FlagAuthException {
      return null;
    }
  }

  // OpenFeature-shaped typed reads (also available via `state.getX`).
  bool boolFlag(String key, {bool or = false}) => state.getBool(key, or: or);
  int intConfig(String key, {int or = 0}) => state.getInt(key, or: or);
  double numberConfig(String key, {double or = 0}) =>
      state.getNumber(key, or: or);
  String stringConfig(String key, {String or = ''}) =>
      state.getString(key, or: or);
  Object? jsonConfig(String key, {Object? or}) => state.getJson(key, or: or);
}
