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

import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:grpc/grpc.dart';
import 'package:music/services/achievements_service.dart';
import 'package:music/services/app_platform.dart';
import 'package:music/services/catalog_service.dart';
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/curator_rewards_service.dart';
import 'package:music/services/global_leaderboard_service.dart';
import 'package:music/services/grpc_client.dart';
import 'package:music/services/leaderboard_service.dart';
import 'package:music/services/notification_service.dart';
import 'package:music/services/plan_service.dart';
import 'package:music/services/play_sync_service.dart';
import 'package:music/services/profile_service.dart';
import 'package:music/services/rating_service.dart';
import 'package:music/services/rpc_deadlines.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/services/soundfont_catalog_service.dart';
import 'package:music/services/streak_service.dart';
import 'package:music/state/leaderboard.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/services/token_store.dart';
import 'package:music/services/usage_tracking_service.dart';

/// Probe interceptor: applies the REAL deadline policy, then captures the
/// [CallOptions] that would reach the channel and aborts the call. Asserting on
/// what it captured proves "the deadline is on the call", not merely "the
/// constructor was passed a list" — through the full production DI.
class _ProbeDeadlines extends RpcDeadlines {
  _ProbeDeadlines();

  final Map<String, CallOptions> captured = {};

  @override
  ResponseFuture<R> interceptUnary<Q, R>(
    ClientMethod<Q, R> method,
    Q request,
    CallOptions options,
    ClientUnaryInvoker<Q, R> invoker,
  ) => super.interceptUnary(method, request, options, (m, q, o) {
    captured[m.path] = o;
    throw GrpcError.cancelled('probe');
  });
}

class _FakeTokenStore extends Fake implements TokenStore {
  @override
  Future<StoredTokens?> readTokens() async =>
      const StoredTokens(accessToken: 't', refreshToken: 'r');
}

void main() {
  late _ProbeDeadlines probe;
  late ProviderContainer container;

  setUp(() {
    probe = _ProbeDeadlines();
    container = ProviderContainer(
      overrides: [
        rpcDeadlinesProvider.overrideWithValue(probe),
        tokenStoreProvider.overrideWithValue(_FakeTokenStore()),
      ],
    );
    addTearDown(container.dispose);
  });

  /// Runs [call] (expected to fail on the probe) and asserts the captured
  /// options for [path] carry [deadline].
  Future<void> expectDeadline(
    String path,
    Duration deadline,
    Future<void> Function() call,
  ) async {
    try {
      await call();
      fail('the probe should have aborted the call');
    } catch (_) {
      // AuthException (mapped) or GrpcError — only the capture matters.
    }
    final sent = probe.captured[path];
    expect(sent, isNotNull, reason: 'no call reached the channel for $path');
    expect(sent!.timeout, deadline, reason: path);
  }

  test('every gRPC adapter sends its category deadline', () async {
    const svc = '/cymbra.music.v1.ScoreService';

    // Interactive (default) across every generated client type.
    await expectDeadline(
      '/cymbra.auth.v1.AuthService/SignInLocal',
      kInteractiveDeadline,
      () => container
          .read(authServiceProvider)
          .signInLocal(email: 'a@b.c', password: 'x'),
    );
    await expectDeadline(
      '/cymbra.user.v1.UserService/GetAccount',
      kInteractiveDeadline,
      () => container.read(accountServiceProvider).getAccount(),
    );
    await expectDeadline(
      '/cymbra.user.v1.UserService/GetPlayerProfile',
      kInteractiveDeadline,
      () => container.read(profileServiceProvider).getPlayerProfile('u'),
    );
    await expectDeadline(
      '$svc/SearchCatalog',
      kInteractiveDeadline,
      () => container.read(catalogServiceProvider).search(),
    );
    await expectDeadline(
      '$svc/ListCourses',
      kInteractiveDeadline,
      () => container.read(courseCatalogServiceProvider).listCourses(),
    );
    await expectDeadline(
      '$svc/GetCourseProgress',
      kInteractiveDeadline,
      () => container.read(courseProgressServiceProvider).completedCourseIds(),
    );
    await expectDeadline(
      '$svc/GetMyScoreRating',
      kInteractiveDeadline,
      () => container.read(ratingServiceProvider).myRating(catalogId: 'c'),
    );
    await expectDeadline(
      '$svc/GetAchievements',
      kInteractiveDeadline,
      () => container.read(achievementsServiceProvider).getAchievements(),
    );
    await expectDeadline(
      '$svc/GetCuratorRewards',
      kInteractiveDeadline,
      () => container.read(curatorRewardsServiceProvider).getRewards(),
    );
    await expectDeadline(
      '$svc/ListSoundFonts',
      kInteractiveDeadline,
      () => container.read(soundFontCatalogServiceProvider).listDownloadable(),
    );
    await expectDeadline(
      '/cymbra.music.v1.LeaderboardService/GetLeaderboard',
      kInteractiveDeadline,
      () => container
          .read(leaderboardServiceProvider)
          .getLeaderboard(scoreId: 'p', mode: LeaderboardMode.tempo),
    );
    await expectDeadline(
      '/cymbra.music.v1.GlobalLeaderboardService/ListGlobalSeasons',
      kInteractiveDeadline,
      () => container.read(globalLeaderboardServiceProvider).listSeasons(),
    );
    await expectDeadline(
      '/cymbra.music.v1.PlayService/GetPlayActivity',
      kInteractiveDeadline,
      () => container.read(playSyncServiceProvider).getPlayActivity('u'),
    );
    await expectDeadline(
      '/cymbra.music.v1.PlayService/GetStreak',
      kInteractiveDeadline,
      () => container.read(streakServiceProvider).getStreak(),
    );
    await expectDeadline(
      '/cymbra.notifications.v1.NotificationService/UnregisterPushToken',
      kInteractiveDeadline,
      () => container
          .read(notificationRegistryServiceProvider)
          .unregisterToken('t'),
    );
    await expectDeadline(
      '/cymbra.analytics.v1.UsageService/ReportEvents',
      kInteractiveDeadline,
      () => container.read(usageTrackingServiceProvider).report(const []),
    );
    await expectDeadline(
      '/cymbra.plans.v1.PlanService/GetMyPlan',
      kInteractiveDeadline,
      () => container.read(planServiceProvider).getMyPlan(AppPlatform.macos),
    );
  });

  test('the byte-transfer RPCs carry the transfer budget', () async {
    const svc = '/cymbra.music.v1.ScoreService';
    await expectDeadline(
      '$svc/GetCatalogScoreBytes',
      kTransferDeadline,
      () => container.read(catalogServiceProvider).fetchScoreBytes('c'),
    );
    await expectDeadline(
      '$svc/GetScoreBytes',
      kTransferDeadline,
      () => container.read(scoreUploadServiceProvider).fetchScoreBytes('id'),
    );
    await expectDeadline(
      '$svc/GetCourse',
      kTransferDeadline,
      () => container
          .read(courseCatalogServiceProvider)
          .getCourseManifestJson('c1'),
    );
  });

  test('the score upload carries the long budget', () async {
    await expectDeadline(
      '/cymbra.music.v1.ScoreService/UploadScore',
      kLongDeadline,
      () => container
          .read(scoreUploadServiceProvider)
          .upload(
            data: Uint8List.fromList(const [1]),
            filename: 'x.musicxml',
            level: PracticeLevel.beginner,
            rightsBasis: RightsBasis.author,
            rightsAck: true,
          ),
    );
  });

  test("the token refresher's own client is bounded too", () async {
    // It builds its AuthServiceClient directly, bypassing every adapter — an
    // unbounded Refresh would be a hung sign-in (task 2.4).
    await expectDeadline(
      '/cymbra.auth.v1.AuthService/Refresh',
      kInteractiveDeadline,
      () => container.read(tokenRefresherProvider).refresh(),
    );
  });
}
