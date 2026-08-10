// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import 'dart:async';
import 'dart:ui' show Locale;

import 'package:flutter_test/flutter_test.dart';
import 'package:music/analytics/usage_actions.dart';
import 'package:music/analytics/usage_environment.dart';
import 'package:music/analytics/usage_event_record.dart';
import 'package:music/services/connectivity_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/usage_tracking_service.dart';
import 'package:music/state/app_locale.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/usage_consent.dart';
import 'package:music/state/usage_outbox_store.dart';
import 'package:music/state/usage_tracking_notifier.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../support/prefs_fakes.dart';

/// A connectivity seam with a manually-pumped "online" event.
class FakeConnectivityService implements ConnectivityService {
  final _controller = StreamController<void>.broadcast();
  @override
  Stream<void> get onOnline => _controller.stream;
  @override
  Stream<bool> get onlineStatus => const Stream.empty();
  @override
  Future<bool> isOnline() async => true;
  void dispose() => _controller.close();
}

/// Stateful recorder + toggleable failure — the "special case" hand fake the
/// testing convention allows (it inspects delivered batches and models an outage).
class FakeUsageTrackingService implements UsageTrackingService {
  final List<List<UsageEventRecord>> batches = [];
  bool fail = false;

  @override
  Future<void> report(List<UsageEventRecord> events) async {
    if (fail) throw Exception('network down');
    batches.add(List.of(events));
  }

  int get delivered => batches.fold(0, (n, b) => n + b.length);
}

class _FakeAppInfo implements AppInfoService {
  @override
  Future<String> version() async => '1.2.3';
}

ProviderContainer _container({
  required FakeUsageTrackingService service,
  required FakePreferencesService prefs,
  FakeConnectivityService? connectivity,
  bool online = true,
  bool killSwitch = true,
}) {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(prefs),
      usageTrackingServiceProvider.overrideWithValue(service),
      connectivityServiceProvider.overrideWithValue(
        connectivity ?? FakeConnectivityService(),
      ),
      appInfoServiceProvider.overrideWithValue(_FakeAppInfo()),
      // No periodic timer fires mid-test.
      usageFlushIntervalProvider.overrideWithValue(null),
      // Deterministic locale (avoids the host locale + real language restore).
      deviceLocaleProvider.overrideWithValue(const Locale('en')),
      canUseOnlineServicesProvider.overrideWithValue(online),
      usageCollectionKillSwitchProvider.overrideWithValue(killSwitch),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  UsageTrackingNotifier notifier(ProviderContainer c) =>
      c.read(usageTrackingNotifierProvider.notifier);

  test('records an event and delivers it when collection is enabled', () async {
    final service = FakeUsageTrackingService();
    final c = _container(service: service, prefs: FakePreferencesService());

    await notifier(c).record(UsageActions.playStart, subjectId: 'score-1');

    expect(service.delivered, 1);
    expect(service.batches.single.single.action, UsageActions.playStart);
    expect(service.batches.single.single.subjectId, 'score-1');
    expect(await c.read(usageOutboxStoreProvider).all(), isEmpty);
  });

  test('an event survives an outage and is delivered on a later flush', () async {
    final service = FakeUsageTrackingService()..fail = true;
    final c = _container(service: service, prefs: FakePreferencesService());
    final n = notifier(c);

    // Delivery fails: the event is KEPT (no loss), no error surfaces to the caller.
    await n.record(UsageActions.authSignIn);
    expect(await c.read(usageOutboxStoreProvider).all(), hasLength(1));
    expect(service.delivered, 0);

    // Connectivity recovers → the buffered event is delivered and dropped.
    service.fail = false;
    await n.flush();
    expect(service.delivered, 1);
    expect(await c.read(usageOutboxStoreProvider).all(), isEmpty);
  });

  test('a flush failure is silent (never throws to the caller)', () async {
    final service = FakeUsageTrackingService()..fail = true;
    final c = _container(service: service, prefs: FakePreferencesService());
    // Must not throw despite the failing transport.
    await notifier(c).record(UsageActions.playStop);
    expect(await c.read(usageOutboxStoreProvider).all(), hasLength(1));
  });

  test('the remote kill-switch suppresses emission entirely', () async {
    final service = FakeUsageTrackingService();
    final c = _container(
      service: service,
      prefs: FakePreferencesService(),
      killSwitch: false,
    );
    await notifier(c).record(UsageActions.playStart);
    expect(service.delivered, 0);
    expect(await c.read(usageOutboxStoreProvider).all(), isEmpty);
  });

  test(
    'per-user consent off suppresses emission and clears the buffer',
    () async {
      final service = FakeUsageTrackingService();
      final c = _container(service: service, prefs: FakePreferencesService());
      final n = notifier(c);

      // Opt out: the listener clears the buffer; subsequent records are no-ops.
      await c.read(usageConsentProvider.notifier).set(false);
      await Future<void>.value();
      await n.record(UsageActions.playStart);

      expect(service.delivered, 0);
      expect(await c.read(usageOutboxStoreProvider).all(), isEmpty);
    },
  );

  test(
    'an unauthenticated session does not emit (guest slice deferred)',
    () async {
      final service = FakeUsageTrackingService();
      final c = _container(
        service: service,
        prefs: FakePreferencesService(),
        online: false,
      );
      await notifier(c).record(UsageActions.playStart);
      expect(service.delivered, 0);
      expect(await c.read(usageOutboxStoreProvider).all(), isEmpty);
    },
  );

  test('the buffered event round-trips through JSON persistence', () async {
    final service = FakeUsageTrackingService()..fail = true;
    final prefs = FakePreferencesService();
    final c = _container(service: service, prefs: prefs);
    await notifier(
      c,
    ).record(UsageActions.settingsChange, variant: UsageVariants.tempo);

    // Re-read from the store: the persisted JSON decodes back to the same event.
    final persisted = await c.read(usageOutboxStoreProvider).all();
    expect(persisted, hasLength(1));
    expect(persisted.single.action, UsageActions.settingsChange);
    expect(persisted.single.variant, UsageVariants.tempo);
    expect(persisted.single.platform, isNotEmpty);
    expect(persisted.single.locale, 'en');
  });
}
