import 'package:cymbra_flags/cymbra_flags.dart';
import 'package:cymbra_flags/src/grpc/flags.pb.dart' as pb;
import 'package:fixnum/fixnum.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

// --- fakes (behavioural in-memory doubles) ---------------------------------

class FakeService implements FlagService {
  FakeService(this.responder);
  final FlagFetch Function(String app, String? id, String known, String? bearer)
  responder;
  final calls = <({String app, String? id, String known, String? bearer})>[];

  @override
  Future<FlagFetch> fetch({
    required String app,
    required String? identity,
    required String knownVersion,
    required String? bearer,
  }) async {
    calls.add((app: app, id: identity, known: knownVersion, bearer: bearer));
    return responder(app, identity, knownVersion, bearer);
  }
}

class FakePrefs implements FlagPreferences {
  final store = <String, String>{};
  @override
  Future<String?> getString(String key) async => store[key];
  @override
  Future<void> setString(String key, String value) async => store[key] = value;
  @override
  Future<void> remove(String key) async => store.remove(key);
}

class FakeBearer implements FlagBearer {
  FakeBearer(this._token);
  final String? _token;
  @override
  Future<String?> token() async => _token;
}

FlagSnapshot _snap(
  String app,
  String? id,
  String version,
  Map<String, FlagEntry> e,
) => FlagSnapshot(app: app, identity: id, version: version, entries: e);

Future<void> _settle() async {
  for (var i = 0; i < 6; i++) {
    await Future<void>.delayed(Duration.zero);
  }
}

ProviderContainer _make({
  String? identity,
  required FlagService service,
  required FlagPreferences prefs,
  FlagBearer? bearer,
}) {
  final c = ProviderContainer(
    overrides: [
      flagPollIntervalProvider.overrideWithValue(null),
      flagIdentityProvider.overrideWithValue(identity),
      flagServiceProvider.overrideWithValue(service),
      flagPreferencesProvider.overrideWithValue(prefs),
      if (bearer != null) flagBearerProvider.overrideWithValue(bearer),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

void main() {
  group('FlagSnapshot', () {
    test('typed reads fall back to the supplied default', () {
      final s = _snap('music', null, 'v1', {
        'f.on': const FlagEntry(FlagKind.bool_, true),
        'n.votes': const FlagEntry(FlagKind.int_, 9),
        'n.thr': const FlagEntry(FlagKind.number, 2.5),
        's.name': const FlagEntry(FlagKind.string, 'hi'),
        'j.cfg': FlagEntry(FlagKind.json, {'a': 1}),
      });
      expect(s.getBool('f.on'), true);
      expect(s.getBool('missing', or: true), true);
      expect(s.getInt('n.votes', or: 1), 9);
      expect(s.getInt('missing', or: 3), 3);
      expect(s.getNumber('n.thr', or: 1), 2.5);
      expect(s.getString('s.name'), 'hi');
      expect(s.getJson('j.cfg'), {'a': 1});
      // wrong-type read returns the default, never coerces
      expect(s.getBool('n.votes', or: false), false);
    });

    test('encode/decode round-trips and is identity-tagged', () {
      final s = _snap('music', 'u1', 'v2', {
        'f': const FlagEntry(FlagKind.bool_, true),
        'j': FlagEntry(FlagKind.json, [1, 2, 3]),
      });
      final back = FlagSnapshot.decode(s.encode())!;
      expect(back.app, 'music');
      expect(back.identity, 'u1');
      expect(back.version, 'v2');
      expect(back.getBool('f'), true);
      expect(back.getJson('j'), [1, 2, 3]);
      expect(FlagSnapshot.decode('not json'), isNull);
    });
  });

  group('entryFromWire', () {
    test('maps each wire kind', () {
      expect(
        entryFromWire(pb.FlagValue(boolValue: true)),
        const FlagEntry(FlagKind.bool_, true),
      );
      expect(
        entryFromWire(pb.FlagValue(intValue: Int64(7))),
        const FlagEntry(FlagKind.int_, 7),
      );
      expect(
        entryFromWire(pb.FlagValue(numberValue: 2.0)),
        const FlagEntry(FlagKind.number, 2.0),
      );
      expect(
        entryFromWire(pb.FlagValue(stringValue: 'x')),
        const FlagEntry(FlagKind.string, 'x'),
      );
      expect(
        entryFromWire(pb.FlagValue(jsonValue: '{"a":1}')),
        FlagEntry(FlagKind.json, {'a': 1}),
      );
      expect(entryFromWire(pb.FlagValue(jsonValue: 'bad json')), isNull);
      expect(entryFromWire(pb.FlagValue()), isNull);
    });
  });

  group('FlagCache', () {
    test('write/read round-trips per identity and clears', () async {
      final cache = FlagCache(FakePrefs());
      final s = _snap('music', 'u1', 'v1', {
        'k': const FlagEntry(FlagKind.bool_, true),
      });
      await cache.write(s);
      expect((await cache.read('music', 'u1'))!.getBool('k'), true);
      // a different identity has its own key (no cross-read)
      expect(await cache.read('music', 'u2'), isNull);
      await cache.clear('music', 'u1');
      expect(await cache.read('music', 'u1'), isNull);
    });

    test('read rejects a row whose identity does not match the key', () async {
      final prefs = FakePrefs();
      // a snapshot tagged u2 stored under u1's key (defensive guard)
      prefs.store[FlagCache.cacheKey('music', 'u1')] = _snap(
        'music',
        'u2',
        'v',
        const {},
      ).encode();
      expect(await FlagCache(prefs).read('music', 'u1'), isNull);
    });
  });

  group('Flags notifier', () {
    test(
      'fetches on launch; per-key reads come from the one snapshot',
      () async {
        final fake = FakeService(
          (app, id, known, bearer) => FlagFetch.changed(
            _snap(app, id, 'v1', {
              'rating.enabled': const FlagEntry(FlagKind.bool_, true),
            }),
          ),
        );
        final c = _make(service: fake, prefs: FakePrefs());
        c.read(flagsProvider);
        await _settle();
        final s = c.read(flagsProvider);
        expect(s.getBool('rating.enabled'), true);
        expect(s.getInt('missing', or: 5), 5);
        expect(
          fake.calls.length,
          1,
          reason: 'a single fetch feeds all local reads',
        );
      },
    );

    test('sends the known version and keeps state on "unchanged"', () async {
      var first = true;
      final fake = FakeService((app, id, known, bearer) {
        if (first) {
          first = false;
          return FlagFetch.changed(
            _snap(app, id, 'v1', {'k': const FlagEntry(FlagKind.bool_, true)}),
          );
        }
        expect(known, 'v1'); // second call carries the known version
        return const FlagFetch.unchanged();
      });
      final c = _make(service: fake, prefs: FakePrefs());
      c.read(flagsProvider);
      await _settle();
      await c.read(flagsProvider.notifier).refresh();
      expect(c.read(flagsProvider).getBool('k'), true);
    });

    test('a failed refresh keeps the last-good snapshot', () async {
      var calls = 0;
      final fake = FakeService((app, id, known, bearer) {
        calls++;
        if (calls == 1) {
          return FlagFetch.changed(
            _snap(app, id, 'v1', {'k': const FlagEntry(FlagKind.bool_, true)}),
          );
        }
        throw Exception('offline');
      });
      final c = _make(service: fake, prefs: FakePrefs());
      c.read(flagsProvider);
      await _settle();
      await c.read(flagsProvider.notifier).refresh(); // throws internally
      expect(
        c.read(flagsProvider).getBool('k'),
        true,
        reason: 'last-good kept',
      );
    });

    test('flicker-free cold start hydrates the persisted snapshot', () async {
      final prefs = FakePrefs();
      // pre-seed a persisted snapshot for identity u1
      final cached = _snap('music', 'u1', 'v9', {
        'k': const FlagEntry(FlagKind.bool_, true),
      });
      prefs.store[FlagCache.cacheKey('music', 'u1')] = cached.encode();
      // server says unchanged, so the hydrated value stands
      final fake = FakeService(
        (app, id, known, bearer) => const FlagFetch.unchanged(),
      );
      final c = _make(identity: 'u1', service: fake, prefs: prefs);
      c.read(flagsProvider);
      await _settle();
      expect(c.read(flagsProvider).getBool('k'), true);
    });

    test('sign-out uses the anonymous set (no bearer)', () async {
      final fake = FakeService(
        (app, id, known, bearer) => FlagFetch.changed(
          _snap(app, id, 'v1', {
            'anon.only': FlagEntry(FlagKind.bool_, bearer == null),
          }),
        ),
      );
      final c = _make(identity: null, service: fake, prefs: FakePrefs());
      c.read(flagsProvider);
      await _settle();
      expect(c.read(flagsProvider).identity, isNull);
      expect(c.read(flagsProvider).getBool('anon.only'), true);
      expect(fake.calls.single.bearer, isNull);
    });

    test('exposes OpenFeature-shaped typed reads', () async {
      final fake = FakeService(
        (app, id, known, bearer) => FlagFetch.changed(
          _snap(app, id, 'v1', {
            'flag': const FlagEntry(FlagKind.bool_, true),
            'votes': const FlagEntry(FlagKind.int_, 9),
            'thr': const FlagEntry(FlagKind.number, 2.5),
            'name': const FlagEntry(FlagKind.string, 'hi'),
            'cfg': FlagEntry(FlagKind.json, {'a': 1}),
          }),
        ),
      );
      final c = _make(service: fake, prefs: FakePrefs());
      c.read(flagsProvider);
      await _settle();
      final n = c.read(flagsProvider.notifier);
      expect(n.boolFlag('flag'), true);
      expect(n.boolFlag('missing', or: true), true);
      expect(n.intConfig('votes'), 9);
      expect(n.numberConfig('thr'), 2.5);
      expect(n.stringConfig('name'), 'hi');
      expect(n.jsonConfig('cfg'), {'a': 1});
    });

    test('installs a foreground poll timer when an interval is set', () async {
      final fake = FakeService(
        (app, id, known, bearer) =>
            FlagFetch.changed(_snap(app, id, 'v1', const {})),
      );
      // Not null → the poll branch runs (timer created + cancelled on dispose).
      final c = ProviderContainer(
        overrides: [
          flagPollIntervalProvider.overrideWithValue(const Duration(hours: 1)),
          flagServiceProvider.overrideWithValue(fake),
          flagPreferencesProvider.overrideWithValue(FakePrefs()),
        ],
      );
      addTearDown(c.dispose);
      c.read(flagsProvider);
      await _settle();
      expect(c.read(flagsProvider).version, 'v1');
    });

    test(
      'user switch refetches and never inherits the previous snapshot',
      () async {
        final prefs = FakePrefs(); // shared device
        // user A: staff snapshot, persisted under A's key
        final aFake = FakeService(
          (app, id, known, bearer) => FlagFetch.changed(
            _snap(app, id, 'va', {
              'staff.only': const FlagEntry(FlagKind.bool_, true),
            }),
          ),
        );
        final ca = _make(
          identity: 'a',
          service: aFake,
          prefs: prefs,
          bearer: FakeBearer('token-a'),
        );
        ca.read(flagsProvider);
        await _settle();
        expect(ca.read(flagsProvider).getBool('staff.only'), true);
        expect(prefs.store.containsKey(FlagCache.cacheKey('music', 'a')), true);

        // user B on the same device: their own (non-staff) set; must not see A's.
        final bFake = FakeService(
          (app, id, known, bearer) => FlagFetch.changed(
            _snap(app, id, 'vb', {
              'staff.only': const FlagEntry(FlagKind.bool_, false),
            }),
          ),
        );
        final cb = _make(
          identity: 'b',
          service: bFake,
          prefs: prefs,
          bearer: FakeBearer('token-b'),
        );
        cb.read(flagsProvider);
        await _settle();
        final b = cb.read(flagsProvider);
        expect(b.identity, 'b');
        expect(
          b.getBool('staff.only'),
          false,
          reason: 'B never inherits A staff flags',
        );
        expect(bFake.calls.single.bearer, 'token-b');
      },
    );
  });
}
