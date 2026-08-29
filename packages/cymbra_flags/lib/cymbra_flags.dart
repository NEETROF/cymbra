/// Shared, app-agnostic Cymbra runtime feature-flag client (change:
/// add-runtime-feature-flags). Reusable by music, live, and future apps.
///
/// Wire it into a host app by overriding the input seams in the root
/// `ProviderScope`/`ProviderContainer`:
/// - [flagChannelProvider] → the app's shared gRPC channel;
/// - [flagIdentityProvider] → the current account id (or `null` when signed out);
/// - [flagBearerProvider] → a [FlagBearer] reading the app's token store.
///
/// Then read flags synchronously off `ref.watch(flagsProvider)` and call
/// `ref.read(flagsProvider.notifier).refresh()` on app resume.
library;

export 'src/flag_cache.dart'
    show FlagCache, FlagPreferences, SharedPreferencesFlagPreferences;
export 'src/flag_service.dart'
    show
        FlagAuthException,
        FlagFetch,
        FlagService,
        GrpcFlagService,
        entryFromWire,
        snapshotFromResponse;
export 'src/flag_snapshot.dart' show FlagEntry, FlagKind, FlagSnapshot;
export 'src/flags.dart';
