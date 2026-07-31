import 'dart:convert';

/// The declared type of a flag/config value (mirrors the backend registry).
enum FlagKind { bool_, int_, number, string, json }

const _kindWire = {
  FlagKind.bool_: 'bool',
  FlagKind.int_: 'int',
  FlagKind.number: 'number',
  FlagKind.string: 'string',
  FlagKind.json: 'json',
};
final _kindByWire = {for (final e in _kindWire.entries) e.value: e.key};

/// A single typed value. [value] is a Dart primitive (`bool`/`int`/`double`/
/// `String`) or decoded JSON (`Map`/`List`) per [kind].
class FlagEntry {
  const FlagEntry(this.kind, this.value);
  final FlagKind kind;
  final Object value;

  Map<String, dynamic> toJson() => {'t': _kindWire[kind], 'v': value};

  static FlagEntry? fromJson(Map<String, dynamic> j) {
    final kind = _kindByWire[j['t']];
    if (kind == null || !j.containsKey('v')) return null;
    return FlagEntry(kind, j['v'] as Object);
  }

  @override
  bool operator ==(Object other) =>
      other is FlagEntry &&
      other.kind == kind &&
      jsonEncode(other.value) == jsonEncode(value);

  @override
  int get hashCode => Object.hash(kind, jsonEncode(value));
}

/// An immutable, identity-scoped snapshot of the caller's effective flags/config,
/// read locally and synchronously per key. Fetched in one gRPC call and cached
/// per identity for a flicker-free cold start.
class FlagSnapshot {
  const FlagSnapshot({
    required this.app,
    required this.identity,
    required this.version,
    required this.entries,
  });

  /// The app scope this snapshot was fetched for (`music`, `live`, …).
  final String app;

  /// The account id the snapshot belongs to, or `null` for the anonymous set.
  final String? identity;

  /// The version/ETag returned with the set — sent back for a cheap "unchanged".
  final String version;

  final Map<String, FlagEntry> entries;

  /// An empty snapshot (no overrides known) — every read returns the caller's
  /// supplied default. The safe cold-start / fail-safe state.
  factory FlagSnapshot.empty(String app, String? identity) => FlagSnapshot(
    app: app,
    identity: identity,
    version: '',
    entries: const {},
  );

  bool getBool(String key, {bool or = false}) {
    final e = entries[key];
    return (e != null && e.kind == FlagKind.bool_) ? e.value as bool : or;
  }

  int getInt(String key, {int or = 0}) {
    final e = entries[key];
    return (e != null && e.kind == FlagKind.int_) ? e.value as int : or;
  }

  double getNumber(String key, {double or = 0}) {
    final e = entries[key];
    return (e != null && e.kind == FlagKind.number)
        ? (e.value as num).toDouble()
        : or;
  }

  String getString(String key, {String or = ''}) {
    final e = entries[key];
    return (e != null && e.kind == FlagKind.string) ? e.value as String : or;
  }

  /// The decoded JSON value (`Map`/`List`) for [key], or [or] when absent.
  Object? getJson(String key, {Object? or}) {
    final e = entries[key];
    return (e != null && e.kind == FlagKind.json) ? e.value : or;
  }

  Map<String, dynamic> toJson() => {
    'app': app,
    'identity': identity,
    'version': version,
    'entries': {for (final e in entries.entries) e.key: e.value.toJson()},
  };

  /// Rebuild a snapshot from its persisted JSON, or `null` if unreadable.
  static FlagSnapshot? fromJson(Map<String, dynamic> j) {
    final app = j['app'];
    final version = j['version'];
    final rawEntries = j['entries'];
    if (app is! String || version is! String || rawEntries is! Map) return null;
    final entries = <String, FlagEntry>{};
    rawEntries.forEach((k, v) {
      if (v is Map) {
        final entry = FlagEntry.fromJson(Map<String, dynamic>.from(v));
        if (entry != null) entries[k as String] = entry;
      }
    });
    return FlagSnapshot(
      app: app,
      identity: j['identity'] as String?,
      version: version,
      entries: entries,
    );
  }

  /// Encode for persistence; decode with [decode].
  String encode() => jsonEncode(toJson());

  static FlagSnapshot? decode(String raw) {
    try {
      final j = jsonDecode(raw);
      return j is Map<String, dynamic> ? FlagSnapshot.fromJson(j) : null;
    } catch (_) {
      return null;
    }
  }
}
