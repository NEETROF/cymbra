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
import 'package:music/services/auth_service.dart';
import 'package:music/services/catalog_service.dart';
import 'package:music/services/grpc_client.dart';
import 'package:music/services/play_sync_service.dart';
import 'package:music/services/profile_service.dart';
import 'package:music/services/rating_service.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/services/token_refresher.dart';

/// The gRPC adapters are built lazily over a [ClientChannel] that only dials on
/// the first RPC, so the whole authenticated-service graph can be wired up
/// without a backend. This pins the production DI (channel → refresher → runner →
/// each adapter) so a mis-wire is caught here rather than at runtime.
void main() {
  test('the authenticated-service provider graph builds without a backend', () {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    expect(container.read(tokenRefresherProvider), isA<TokenRefresher>());
    expect(container.read(authedRunnerProvider), isA<AuthedRunner>());
    expect(container.read(authServiceProvider), isA<AuthService>());
    expect(container.read(accountServiceProvider), isA<AccountService>());
    expect(container.read(playSyncServiceProvider), isA<PlaySyncService>());
    expect(container.read(profileServiceProvider), isA<ProfileService>());
    expect(container.read(catalogServiceProvider), isA<CatalogService>());
    expect(
      container.read(scoreUploadServiceProvider),
      isA<ScoreUploadService>(),
    );
    expect(container.read(ratingServiceProvider), isA<RatingService>());
  });
}
