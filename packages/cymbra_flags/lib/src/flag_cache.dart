import 'package:shared_preferences/shared_preferences.dart';

import 'flag_snapshot.dart';

/// Minimal key/value seam for the persisted snapshot cache. The package ships a
/// `shared_preferences` impl; an app may override it to reuse its own store.
abstract class FlagPreferences {
  Future<String?> getString(String key);
  Future<void> setString(String key, String value);
  Future<void> remove(String key);
}

/// `shared_preferences`-backed [FlagPreferences]. I/O glue behind the seam;
/// tests inject an in-memory double, so this is coverage-excluded.
// coverage:ignore-start
class SharedPreferencesFlagPreferences implements FlagPreferences {
  Future<SharedPreferences>? _prefs;
  Future<SharedPreferences> get _instance =>
      _prefs ??= SharedPreferences.getInstance();

  @override
  Future<String?> getString(String key) async =>
      (await _instance).getString(key);

  @override
  Future<void> setString(String key, String value) async =>
      (await _instance).setString(key, value);

  @override
  Future<void> remove(String key) async => (await _instance).remove(key);
}
// coverage:ignore-end

/// The persisted per-identity snapshot cache. Keys include the identity so one
/// user's flags (e.g. staff-only features) never load for another on a shared
/// device.
class FlagCache {
  FlagCache(this._prefs);
  final FlagPreferences _prefs;

  static String cacheKey(String app, String? identity) =>
      'cymbra.flags.$app.${identity ?? 'anon'}';

  Future<FlagSnapshot?> read(String app, String? identity) async {
    final raw = await _prefs.getString(cacheKey(app, identity));
    if (raw == null || raw.isEmpty) return null;
    final snap = FlagSnapshot.decode(raw);
    // Guard against a mismatched cache row (defensive: never load another
    // identity's set even if the key were reused).
    if (snap == null || snap.app != app || snap.identity != identity) {
      return null;
    }
    return snap;
  }

  Future<void> write(FlagSnapshot snap) =>
      _prefs.setString(cacheKey(snap.app, snap.identity), snap.encode());

  Future<void> clear(String app, String? identity) =>
      _prefs.remove(cacheKey(app, identity));
}
