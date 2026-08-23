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
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/screens/open_score.dart';
import 'package:music/services/connectivity_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/offline_score_cache.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/score_catalog.dart';

import '../support/notation_fakes.dart';

/// A fetch that never completes — the load hangs behind the blocking dialog
/// until the user cancels.
class _HungUpload extends Fake implements ScoreUploadService {
  final pending = Completer<ScoreBytesResult>();
  @override
  Future<ScoreBytesResult> fetchScoreBytes(String id, {String? ifNoneMatch}) =>
      pending.future;
}

class _OnlineConn extends Fake implements ConnectivityService {
  @override
  Stream<bool> get onlineStatus => const Stream.empty();
  @override
  Future<bool> isOnline() async => true;
  @override
  Future<bool> isDefinitelyOffline() async => false;
}

class _Launcher extends ConsumerWidget {
  const _Launcher();

  @override
  Widget build(BuildContext context, WidgetRef ref) => Scaffold(
    body: Center(
      child: ElevatedButton(
        onPressed: () => openScore(
          context,
          ref,
          const CatalogEntry(
            id: 'contrib-1',
            title: 'My Upload',
            composer: 'Me',
            level: PracticeLevel.beginner,
            contributedId: '1',
            favorite: true,
          ),
        ),
        child: const Text('open'),
      ),
    ),
  );
}

void main() {
  testWidgets(
    'the blocking wait is cancellable: no banner, selection cleared',
    (tester) async {
      final upload = _HungUpload();
      final container = ProviderContainer(
        overrides: [
          notationEngineProvider.overrideWithValue(FakeNotationEngine()),
          offlineScoreCacheProvider.overrideWithValue(
            InMemoryOfflineScoreCache(),
          ),
          scoreUploadServiceProvider.overrideWithValue(upload),
          connectivityServiceProvider.overrideWithValue(_OnlineConn()),
        ],
      );
      addTearDown(container.dispose);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: MaterialApp(
            localizationsDelegates: AppLocalizations.localizationsDelegates,
            supportedLocales: AppLocalizations.supportedLocales,
            locale: const Locale('en'),
            home: const _Launcher(),
          ),
        ),
      );

      // Open: the load hangs, so the blocking dialog is up with its exit.
      await tester.tap(find.text('open'));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 100));
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      final cancel = find.widgetWithText(TextButton, 'Cancel');
      expect(
        cancel,
        findsOneWidget,
        reason: 'a blocking wait must expose an exit the whole time (D10)',
      );

      // Cancel: the dialog closes (single pop — the launcher screen survives),
      // the selection is cleared, and NO error banner appears.
      await tester.tap(cancel);
      // Let the min-spinner delay elapse, then the pop.
      await tester.pump(const Duration(milliseconds: 600));
      await tester.pumpAndSettle();

      expect(find.byType(CircularProgressIndicator), findsNothing);
      expect(
        find.text('open'),
        findsOneWidget,
        reason: 'still on the launcher',
      );
      expect(
        find.byType(SnackBar),
        findsNothing,
        reason: 'cancel is not a failure',
      );
      expect(container.read(selectedScoreProvider), isNull);

      // The abandoned fetch completing late changes nothing on screen.
      upload.pending.complete(
        ScoreBytesResult(data: Uint8List.fromList(const [1]), etag: 'e'),
      );
      await tester.pump();
      await tester.pumpAndSettle();
      expect(find.byType(SnackBar), findsNothing);
      expect(find.text('open'), findsOneWidget);
    },
  );
}
