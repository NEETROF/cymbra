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

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/screens/score_upload_screen.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/file_picker_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/src/rust/api/musicxml.dart'
    show InstrumentKind, ScoreSummary, ValidationOutcome;
import 'package:music/services/preferences_service.dart';
import 'package:music/services/private_soundfont_service.dart';
import 'package:music/services/soundfont_catalog_service.dart';
import 'package:music/services/soundfont_importer.dart';
import 'package:music/services/soundfont_source.dart';
import 'package:music/state/drums_access.dart';
import 'package:music/state/score_font.dart';
import 'package:music/state/score_catalog.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';
import '../support/notation_fakes.dart';

class _FakePicker implements FilePickerService {
  _FakePicker(this.next);
  final PickedScoreFile? next;
  @override
  Future<PickedScoreFile?> pickScore() async => next;

  @override
  Future<List<PickedScoreFile>> pickScores() async =>
      next == null ? const [] : [next!];
}

class _FakeUpload implements ScoreUploadService {
  final List<({PracticeLevel level, RightsBasis basis})> uploads = [];
  Object? uploadError;

  @override
  Future<ContributedScore> upload({
    required Uint8List data,
    required String filename,
    required PracticeLevel level,
    required RightsBasis rightsBasis,
    required bool rightsAck,
    String? fallbackTitle,
    String? fallbackComposer,
  }) async {
    uploads.add((level: level, basis: rightsBasis));
    if (uploadError != null) throw uploadError!;
    return ContributedScore(
      id: 'new-id',
      level: level,
      createdAt: DateTime.utc(2026),
      measureCount: 4,
      timeSig: '4/4',
      keyFifths: 0,
      title: 'Sample',
    );
  }

  final List<String> proposed = [];

  @override
  Future<List<ContributedScore>> listMyScores() async => const [];

  @override
  Future<List<ContributedScore>> listMyScoresInCollection(
    String collectionId,
  ) async => const [];

  @override
  Future<List<ScoreCollection>> listCollections() async => const [];

  @override
  Future<ScoreCollection> createCollection(String name) async =>
      ScoreCollection(id: 'c1', name: name, createdAt: DateTime.utc(2026));

  @override
  Future<void> renameCollection(String id, String name) async {}

  @override
  Future<void> deleteCollection(String id) async {}

  @override
  Future<void> addToCollection(String collectionId, String scoreId) async {}

  @override
  Future<void> removeFromCollection(
    String collectionId,
    String scoreId,
  ) async {}

  @override
  Future<UploadAllowance> uploadAllowance() async => const UploadAllowance(
    remaining: 100,
    max: 100,
    windowDays: 7,
    upgradeRaisesLimit: false,
  );
  @override
  Future<void> deleteScore(String id) async {}
  @override
  Future<void> setFavorite(String id, bool favorite) async {}
  @override
  Future<void> propose({
    required String scoreId,
    required String license,
    required bool attestation,
    String attribution = '',
    String? resubmissionNote,
  }) async {
    proposed.add(scoreId);
  }

  @override
  Future<ScoreBytesResult> fetchScoreBytes(
    String id, {
    String? ifNoneMatch,
  }) async => ScoreBytesResult(data: Uint8List(0), etag: '', unchanged: false);
}

PickedScoreFile _validFile() => PickedScoreFile(
  name: 'song.musicxml',
  bytes: Uint8List.fromList(const [1, 2, 3]),
);

ProviderContainer _container({
  PickedScoreFile? pick,
  _FakeUpload? upload,
  FakeNotationEngine? engine,
  bool drumsEnabled = false,
  RecordingAudioService? audio,
}) {
  final c = ProviderContainer(
    overrides: [
      filePickerProvider.overrideWithValue(_FakePicker(pick)),
      notationEngineProvider.overrideWithValue(engine ?? FakeNotationEngine()),
      scoreUploadServiceProvider.overrideWithValue(upload ?? _FakeUpload()),
      audioServiceProvider.overrideWithValue(audio ?? RecordingAudioService()),
      drumsEnabledProvider.overrideWithValue(drumsEnabled),
      // The preview installs the score's family font through the same
      // controller the player uses, so its seams must be doubled here.
      preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
      soundFontSourceProvider.overrideWithValue(FakeSoundFontSource()),
      soundFontImporterProvider.overrideWithValue(FakeSoundFontImporter()),
      privateSoundFontServiceProvider.overrideWithValue(
        FakePrivateSoundFontService(),
      ),
      soundFontCatalogServiceProvider.overrideWithValue(
        FakeSoundFontCatalogService(),
      ),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

/// A validation outcome whose summary is a valid percussion (drum) score.
ValidationOutcome _percussionOutcome() => const ValidationOutcome(
  summary: ScoreSummary(
    title: 'Groove',
    composer: 'A. Drummer',
    titleNorm: 'groove',
    workKey: 'a. drummer::groove',
    instrument: InstrumentKind.percussion,
    staves: 1,
    keyFifths: 0,
    timeSig: '4/4',
    measureCount: 4,
    noteCount: 8,
  ),
);

Future<void> _pump(WidgetTester tester, ProviderContainer container) async {
  await tester.binding.setSurfaceSize(const Size(900, 1400));
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(const ScoreUploadScreen()),
    ),
  );
}

/// The verify step drives a Ticker, so `pumpAndSettle` never settles — pump a
/// fixed number of frames instead.
Future<void> _pumpFrames(WidgetTester tester, [int n = 8]) async {
  for (var i = 0; i < n; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

/// Unmounts the tree so the verify step's Ticker is disposed before the test ends.
Future<void> _teardown(WidgetTester tester) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
}

bool _enabled(WidgetTester tester, String label) =>
    tester.widget<TextButton>(find.widgetWithText(TextButton, label)).enabled;

/// Hosts the wizard on a **pushed** route: its post-upload propose step closes
/// the wizard with `Navigator.pop`, which needs a route underneath.
Future<void> _pumpPushed(
  WidgetTester tester,
  ProviderContainer container,
) async {
  await tester.binding.setSurfaceSize(const Size(900, 1400));
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(
        Builder(
          builder: (context) => Scaffold(
            body: Center(
              child: ElevatedButton(
                onPressed: () => Navigator.of(context).push(
                  MaterialPageRoute<void>(
                    builder: (_) => const ScoreUploadScreen(),
                  ),
                ),
                child: const Text('open wizard'),
              ),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('open wizard'));
  await _pumpFrames(tester);
}

/// Drives the wizard through pick → attest → verify → confirm → submit, leaving
/// it on the success screen.
Future<void> _driveToSuccess(WidgetTester tester) async {
  await tester.tap(find.text('Choose a file'));
  await _pumpFrames(tester);
  await tester.tap(find.text('I am the author'));
  await tester.pump();
  await tester.tap(find.byType(CheckboxListTile));
  await tester.pump();
  await tester.tap(find.text('Verify'));
  await _pumpFrames(tester);
  await tester.tap(find.text('Continue'));
  await _pumpFrames(tester);
  await tester.tap(find.text('Beginner'));
  await tester.pump();
  await tester.tap(find.text('Submit'));
  await _pumpFrames(tester);
}

void main() {
  testWidgets('Verify is gated on a validated file AND the rights attestation', (
    tester,
  ) async {
    final container = _container(pick: _validFile());
    await _pump(tester, container);

    // Nothing picked yet: the forward action is present but disabled.
    expect(find.text('Verify'), findsOneWidget);
    expect(_enabled(tester, 'Verify'), isFalse);

    // Pick + validate.
    await tester.tap(find.text('Choose a file'));
    await _pumpFrames(tester);
    expect(find.textContaining('is valid'), findsOneWidget);
    // A validated file alone is not enough — the attestation is still missing.
    expect(_enabled(tester, 'Verify'), isFalse);

    // Declare a basis and tick the confirmation.
    await tester.tap(find.text('Public domain / open licence'));
    await tester.pump();
    await tester.tap(find.byType(CheckboxListTile));
    await tester.pump();
    expect(_enabled(tester, 'Verify'), isTrue);
    await _teardown(tester);
  });

  testWidgets('full flow: upload → tempo-locked verify → confirm → submit', (
    tester,
  ) async {
    final upload = _FakeUpload();
    final container = _container(pick: _validFile(), upload: upload);
    await _pump(tester, container);

    // Upload step: pick + attest.
    await tester.tap(find.text('Choose a file'));
    await _pumpFrames(tester);
    await tester.tap(find.text('I am the author'));
    await tester.pump();
    await tester.tap(find.byType(CheckboxListTile));
    await tester.pump();

    // → Verify: read-only metadata, and NO practice controls (no tempo slider).
    await tester.tap(find.text('Verify'));
    await _pumpFrames(tester);
    expect(find.text('Detected information (read-only)'), findsOneWidget);
    expect(find.byType(Slider), findsNothing);

    // → Confirm: difficulty gate.
    await tester.tap(find.text('Continue'));
    await _pumpFrames(tester);
    expect(find.text('Difficulty level'), findsOneWidget);
    // 'Submit' is disabled until a level is chosen.
    expect(_enabled(tester, 'Submit'), isFalse);

    await tester.tap(find.text('Intermediate')); // localized label (en default)
    await tester.pump();
    expect(_enabled(tester, 'Submit'), isTrue);

    await tester.tap(find.text('Submit'));
    await _pumpFrames(tester);

    // Submitted with the chosen level/basis; success screen shown.
    expect(upload.uploads.single.level, PracticeLevel.intermediate);
    expect(upload.uploads.single.basis, RightsBasis.author);
    expect(find.text('Score added to your contributions.'), findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('the AppBar back arrow steps back through the wizard', (
    tester,
  ) async {
    final container = _container(pick: _validFile());
    await _pump(tester, container);

    // Drive to the confirm step.
    await tester.tap(find.text('Choose a file'));
    await _pumpFrames(tester);
    await tester.tap(find.text('I am the author'));
    await tester.pump();
    await tester.tap(find.byType(CheckboxListTile));
    await tester.pump();
    await tester.tap(find.text('Verify'));
    await _pumpFrames(tester);
    await tester.tap(find.text('Continue'));
    await _pumpFrames(tester);
    expect(find.text('Difficulty level'), findsOneWidget);

    // Back: confirm → verify.
    await tester.tap(find.byIcon(Icons.arrow_back));
    await _pumpFrames(tester);
    expect(find.text('Detected information (read-only)'), findsOneWidget);

    // Back: verify → upload.
    await tester.tap(find.byIcon(Icons.arrow_back));
    await _pumpFrames(tester);
    expect(find.textContaining('Accepted formats'), findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('a rejected file shows the reject banner and blocks Verify', (
    tester,
  ) async {
    final engine = FakeNotationEngine(
      validateOutcome: ValidationOutcome(rejectCode: 'no_notes'),
    );
    final container = _container(pick: _validFile(), engine: engine);
    await _pump(tester, container);

    await tester.tap(find.text('Choose a file'));
    await _pumpFrames(tester);

    // No "valide" banner, no attestation controls, forward stays disabled.
    expect(find.textContaining('is valid'), findsNothing);
    expect(find.byType(CheckboxListTile), findsNothing);
    expect(_enabled(tester, 'Verify'), isFalse);
    await _teardown(tester);
  });

  testWidgets('a submit failure surfaces a friendly banner and keeps inputs', (
    tester,
  ) async {
    final upload = _FakeUpload()
      ..uploadError = const AuthException(
        AuthError.rateLimited,
        'quota exceeded',
      );
    final container = _container(pick: _validFile(), upload: upload);
    await _pump(tester, container);

    await tester.tap(find.text('Choose a file'));
    await _pumpFrames(tester);
    await tester.tap(find.text('I am the author'));
    await tester.pump();
    await tester.tap(find.byType(CheckboxListTile));
    await tester.pump();
    await tester.tap(find.text('Verify'));
    await _pumpFrames(tester);
    await tester.tap(find.text('Continue'));
    await _pumpFrames(tester);
    await tester.tap(find.text('Beginner'));
    await tester.pump();
    await tester.tap(find.text('Submit'));
    await _pumpFrames(tester);

    // Friendly FR message (never the raw exception); still on the confirm step.
    expect(
      find.textContaining('quota'),
      findsOneWidget,
      reason: 'the rate-limit message mentions the quota',
    );
    expect(find.textContaining('AuthException'), findsNothing);
    expect(find.text('Difficulty level'), findsOneWidget);
    await _teardown(tester);
  });

  // The opt-in propose step (change: add-score-catalog-proposal): offered AFTER a
  // successful upload, never folded into the submission itself.
  testWidgets(
    'declining the post-upload propose step leaves the score private',
    (tester) async {
      final upload = _FakeUpload();
      final container = _container(pick: _validFile(), upload: upload);
      await _pumpPushed(tester, container);
      await _driveToSuccess(tester);

      // The proposal is offered as a separate, un-ticked step.
      expect(
        find.text('Propose this score to the public catalog?'),
        findsOneWidget,
      );
      expect(find.text('Propose to catalog'), findsOneWidget);

      await tester.tap(find.text('Not now'));
      await _pumpFrames(tester, 24); // let the route pop transition finish

      // Nothing proposed, and the wizard closed back to the host route.
      expect(upload.proposed, isEmpty);
      expect(find.byType(ScoreUploadScreen), findsNothing);
      await _teardown(tester);
    },
  );

  testWidgets('accepting the propose step calls the propose seam', (
    tester,
  ) async {
    final upload = _FakeUpload();
    final container = _container(pick: _validFile(), upload: upload);
    await _pumpPushed(tester, container);
    await _driveToSuccess(tester);

    await tester.tap(find.text('Propose to catalog'));
    await _pumpFrames(tester);

    // Same shared sheet as the contributions list: gated on the attestation.
    await tester.tap(find.byType(Checkbox));
    await _pumpFrames(tester);
    await tester.tap(find.widgetWithText(FilledButton, 'Propose'));
    await _pumpFrames(tester);

    // The just-uploaded score went through the injectable seam.
    expect(upload.proposed, ['new-id']);
    await _teardown(tester);
  });

  testWidgets(
    'the Verify summary shows the DETECTED instrument, with no control',
    (tester) async {
      // The default fake summary is a keyboard score.
      final container = _container(pick: _validFile());
      await _pump(tester, container);

      await tester.tap(find.text('Choose a file'));
      await _pumpFrames(tester);
      await tester.tap(find.text('I am the author'));
      await tester.pump();
      await tester.tap(find.byType(CheckboxListTile));
      await tester.pump();
      await tester.tap(find.text('Verify'));
      await _pumpFrames(tester);

      // Read-only row naming the detected family — display only: no dropdown,
      // switch or radio offers to change it.
      expect(find.text('Instrument'), findsOneWidget);
      expect(find.text('Piano'), findsOneWidget);
      expect(find.byType(DropdownButton), findsNothing);
      expect(find.byType(Switch), findsNothing);
      await _teardown(tester);
    },
  );

  testWidgets('a percussion upload shows Drums in the Verify summary', (
    tester,
  ) async {
    final engine = FakeNotationEngine(validateOutcome: _percussionOutcome());
    final container = _container(
      pick: _validFile(),
      engine: engine,
      drumsEnabled: true,
    );
    await _pump(tester, container);

    await tester.tap(find.text('Choose a file'));
    await _pumpFrames(tester);
    await tester.tap(find.text('I am the author'));
    await tester.pump();
    await tester.tap(find.byType(CheckboxListTile));
    await tester.pump();
    await tester.tap(find.text('Verify'));
    await _pumpFrames(tester);

    expect(find.text('Instrument'), findsOneWidget);
    expect(find.text('Drums'), findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('the Verify preview of a drum upload asks for the kit and never '
      'sounds a drum score through the piano', (tester) async {
    // The defect this pins: the preview parsed the groove correctly and even
    // labelled it "Batterie", then sounded its General MIDI numbers as PIANO
    // pitches — the font-follows-the-score rule reached the player only.
    final audio = RecordingAudioService();
    final engine = FakeNotationEngine(
      validateOutcome: _percussionOutcome(),
      document: sampleDrumDocument(),
    );
    final container = _container(
      pick: _validFile(),
      engine: engine,
      drumsEnabled: true,
      audio: audio,
    );
    await _pump(tester, container);

    await tester.tap(find.text('Choose a file'));
    await _pumpFrames(tester);
    await tester.tap(find.text('I am the author'));
    await tester.pump();
    await tester.tap(find.byType(CheckboxListTile));
    await tester.pump();
    await tester.tap(find.text('Verify'));
    // The chain is longer than the player's: parse → rebuild with the family →
    // awaited swap → ready.
    await _pumpFrames(tester, 30);

    // The preview asked for its family's font: it is no longer sitting on the
    // piano (`inactive` is the keyboard state).
    expect(
      container.read(scoreFontProvider),
      isNot(KitFontStatus.inactive),
      reason: 'the preview never asked for the drum kit',
    );

    // Play the preview: a drum score must NEVER sound melodic notes — that is
    // the defect (piano pitches for General MIDI drum numbers). Whether the
    // kit has finished installing only decides between drum strokes and
    // honest silence, never piano.
    await tester.tap(find.byIcon(Icons.play_arrow));
    await _pumpFrames(tester, 12);
    expect(
      audio.noteOns,
      isEmpty,
      reason: 'a drum score sounded melodic notes (the piano-font defect)',
    );
    expect(audio.noteOffs, isEmpty);
    await _teardown(tester);
  });

  testWidgets(
    'a percussion file is refused with a localized reason when drums are invisible',
    (tester) async {
      final engine = FakeNotationEngine(validateOutcome: _percussionOutcome());
      // drumsEnabled defaults to false — the drum feature is not visible.
      final container = _container(pick: _validFile(), engine: engine);
      await _pump(tester, container);

      await tester.tap(find.text('Choose a file'));
      await _pumpFrames(tester);

      // The localized refusal shows; the flow does not advance (no attestation
      // controls, Verify stays disabled) — and never a raw technical string.
      expect(
        find.text('Drum scores are not available on your account yet.'),
        findsOneWidget,
      );
      expect(find.textContaining('is valid'), findsNothing);
      expect(find.byType(CheckboxListTile), findsNothing);
      expect(_enabled(tester, 'Verify'), isFalse);
      expect(find.textContaining('drums_not_available'), findsNothing);
      await _teardown(tester);
    },
  );
}
