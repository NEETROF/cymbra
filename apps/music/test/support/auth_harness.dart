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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/account_service.dart';
import 'package:music/services/curator_rewards_service.dart';
import 'package:music/services/grpc_client.dart';
import 'package:music/services/notification_service.dart';
import 'package:music/services/oidc_token_source.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/token_store.dart';
import 'package:music/analytics/usage_event_record.dart';
import 'package:music/services/streak_service.dart';
import 'package:music/services/usage_tracking_service.dart';

import 'auth_fakes.dart';
import 'prefs_fakes.dart';

/// A no-op curator-rewards seam: the account control now renders the standing
/// pill, so any test pumping `AccountMenu` needs this seam wired (returns an empty
/// standing so the pill shows its fallback without touching a channel).
class FakeCuratorRewardsService implements CuratorRewardsService {
  const FakeCuratorRewardsService();

  @override
  Future<CuratorRewardsView> getRewards() async => const CuratorRewardsView(
    lifetimePoints: 0,
    spendableBalance: 0,
    level: 0,
    levelFloor: 0,
    nextLevelAt: 50,
    totalRatings: 0,
    coverageContribution: 0,
    alignmentRate: 0,
    badges: [],
    recent: [],
  );

  @override
  Future<List<RewardShopItemView>> listShop() async => const [];

  @override
  Future<RedeemResultView> redeem(String rewardKey) async =>
      const RedeemResultView(owned: true, newBalance: 0);
}

/// A no-op notification-registry seam: the account menu reads the caller's
/// per-category notification preferences (change: add-push-notifications), so any
/// test pumping `AccountMenu` needs this wired (returns empty settings without
/// touching a channel).
class FakeNotificationRegistryService implements NotificationRegistryService {
  const FakeNotificationRegistryService();

  @override
  Future<void> registerToken({
    required String token,
    required String platform,
  }) async {}

  @override
  Future<void> unregisterToken(String token) async {}

  @override
  Future<void> setPref({
    required String category,
    required bool enabled,
  }) async {}

  @override
  Future<void> setTimezone(String timezone) async {}

  @override
  Future<NotificationSettings> settings() async => NotificationSettings.empty;
}

/// Override list for the Cymbra ID seams, so nothing touches a channel or
/// platform plugin. Compose with extra overrides (e.g. `scoreCatalogProvider`).
/// The account shell renders the streak chip; keep it off the channel.
class _FakeStreakService extends Fake implements StreakService {
  @override
  Future<StreakView> getStreak() async => StreakView.none;
}

/// Screens record usage telemetry on the way through; keep it off the channel
/// (change: add-client-transport-deadlines — a real call would arm a deadline
/// timer the test binding flags as pending).
class _FakeUsageTrackingService extends Fake implements UsageTrackingService {
  @override
  Future<void> report(List<UsageEventRecord> events) async {}
}

List<Override> authOverrides({
  FakeTokenStore? store,
  FakeAuthService? auth,
  FakeAccountService? account,
  FakeOidcTokenSource? oidc,
}) => [
  tokenStoreProvider.overrideWithValue(store ?? FakeTokenStore()),
  authServiceProvider.overrideWithValue(auth ?? FakeAuthService()),
  accountServiceProvider.overrideWithValue(account ?? FakeAccountService()),
  oidcTokenSourceProvider.overrideWithValue(oidc ?? FakeOidcTokenSource()),
  // The account control renders the curator standing pill (change: add-curation-
  // rewards), so wire its seam here too.
  curatorRewardsServiceProvider.overrideWithValue(
    const FakeCuratorRewardsService(),
  ),
  preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
  streakServiceProvider.overrideWithValue(_FakeStreakService()),
  usageTrackingServiceProvider.overrideWithValue(_FakeUsageTrackingService()),
  // The account menu renders a switch per declared notification category
  // (change: add-push-notifications) and reads the stored preferences.
  notificationRegistryServiceProvider.overrideWithValue(
    const FakeNotificationRegistryService(),
  ),
];

/// A [ProviderContainer] with every Cymbra ID seam overridden by a fake.
ProviderContainer authContainer({
  FakeTokenStore? store,
  FakeAuthService? auth,
  FakeAccountService? account,
  FakeOidcTokenSource? oidc,
}) {
  final container = ProviderContainer(
    overrides: authOverrides(
      store: store,
      auth: auth,
      account: account,
      oidc: oidc,
    ),
  );
  addTearDown(container.dispose);
  return container;
}

/// Helper to read a fresh [Account] with the given handle.
Account fakeAccount({String? handle, int version = 1}) =>
    Account(userId: 'user-1', version: version, handle: handle);
