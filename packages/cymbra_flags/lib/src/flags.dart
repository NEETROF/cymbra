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
@Riverpod(keepAlive: true)
String? flagIdentity(Ref ref) => null;

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
  Future<String?> token();
}

/// The default anonymous bearer (no token).
class AnonymousFlagBearer implements FlagBearer {
  const AnonymousFlagBearer();
  @override
  Future<String?> token() async => null;
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
  Future<void> refresh() async {
    final app = state.app;
    final identity = state.identity;
    try {
      final bearer = await ref.read(flagBearerProvider).token();
      final fetch = await ref
          .read(flagServiceProvider)
          .fetch(
            app: app,
            identity: identity,
            knownVersion: state.version,
            bearer: bearer,
          );
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

  // OpenFeature-shaped typed reads (also available via `state.getX`).
  bool boolFlag(String key, {bool or = false}) => state.getBool(key, or: or);
  int intConfig(String key, {int or = 0}) => state.getInt(key, or: or);
  double numberConfig(String key, {double or = 0}) =>
      state.getNumber(key, or: or);
  String stringConfig(String key, {String or = ''}) =>
      state.getString(key, or: or);
  Object? jsonConfig(String key, {Object? or}) => state.getJson(key, or: or);
}
