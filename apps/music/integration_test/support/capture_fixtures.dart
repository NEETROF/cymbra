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

/// The seeded state the store captures are taken against (change:
/// add-store-screenshot-harness, design D3/D4).
///
/// Everything a listing image shows is decided here rather than by a backend:
/// the July and August passes captured against someone's local Postgres, which
/// is exactly why they could not be reproduced. The *scores* and the *courses*
/// are the real shipped ones (bundled assets, real curriculum metadata) — only
/// the account-shaped seams around them are faked.
library;

import 'dart:async';

import 'package:flutter/widgets.dart' show Locale;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:music/courses/course_manifest.dart';
import 'package:music/services/account_service.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/connectivity_service.dart';
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/grpc_client.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/services/token_store.dart';
import 'package:music/src/rust/api/midi.dart'
    show MidiEcho, MidiEvent, MidiEventKind;
import 'package:music/state/app_locale.dart';
import 'package:music/state/coaching_notifier.dart';
import 'package:music/state/contributed_scores.dart';
import 'package:music/state/drums_access.dart';
import 'package:music/state/favorite_scores.dart';
import 'package:music/state/instrument_context.dart';
import 'package:music/state/onboarding_notifier.dart';
import 'package:music/state/saved_catalog_scores.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';

import 'capture_courses.dart';

/// The MIDI instrument the captures show as connected. A plausible product name
/// rather than a test string: it is legible in the player's settings menu.
const String kCaptureMidiDevice = 'Yamaha P-145';

/// How many lessons the captured account has finished — enough for the learning
/// path to show progress rather than an untouched curriculum.
const int kCaptureCompletedLessons = 7;

/// Everything the capture run overrides, in one list.
///
/// [locale] drives the app's display language (D2): [AppLocale] resolves it from
/// [deviceLocaleProvider], and the in-memory preferences carry no persisted
/// language that could win over it.
List<Override> captureOverrides({
  required String locale,
  required CaptureMidiService midi,
}) => [
  // --- Language (D2) --------------------------------------------------------
  deviceLocaleProvider.overrideWithValue(Locale(locale)),
  preferencesServiceProvider.overrideWithValue(CapturePrefs()),

  // --- Session --------------------------------------------------------------
  // A guest token store keeps the run off platform secure storage, while
  // `canUseOnlineServices` is forced on so the signed-in surfaces (courses,
  // favorites) render. No real account is ever touched: every service that
  // would reach one is faked below — in particular [AccountService], so
  // `AppLocale`'s restore can never push a `SetLocale` (D2 consequence).
  tokenStoreProvider.overrideWithValue(const CaptureTokenStore()),
  canUseOnlineServicesProvider.overrideWithValue(true),
  accountServiceProvider.overrideWithValue(const CaptureAccountService()),
  connectivityServiceProvider.overrideWithValue(const CaptureConnectivity()),

  // --- Content --------------------------------------------------------------
  // The favorites ARE the bundled catalog: the five public-domain scores the
  // app ships, so the library shot advertises real content and the player opens
  // a real score.
  favoriteScoresProvider.overrideWith(
    (ref) async => ref.watch(scoreCatalogProvider),
  ),
  // The favorites' two backend sources are watched directly by the library's
  // listener widget, which raises a "that didn't work" snackbar on failure — an
  // unreachable backend would post it over the very first capture. They carry
  // no content of their own here (the favorites above are the whole library),
  // so they resolve empty.
  myUploadsProvider.overrideWith(CaptureUploads.new),
  savedCatalogScoresProvider.overrideWith(CaptureSavedCatalogScores.new),
  courseCatalogServiceProvider.overrideWithValue(const CaptureCourseCatalog()),
  courseProgressServiceProvider.overrideWithValue(
    const CaptureCourseProgress(),
  ),

  // --- Drums (change: add-drum-notation-render … add-drum-scoring) ---------
  // The drum feature is server-gated in production; the captures show it, so
  // the visibility predicate is forced on here. Consequences handled above and
  // below: the bundled drum grooves join the catalog (and therefore the seeded
  // favorites), and the one-time instrument choice is pre-answered in
  // [CapturePrefs] so its modal can never open over a capture.
  drumsEnabledProvider.overrideWithValue(true),

  // --- Devices --------------------------------------------------------------
  midiServiceProvider.overrideWithValue(midi),
  // Silent audio: a capture needs no sound, and the real service extracts a
  // ~50 MB SoundFont and opens an output device — slow, and unavailable on some
  // simulators.
  audioServiceProvider.overrideWithValue(const CaptureAudioService()),
];

/// In-memory [PreferencesService] seeded so the app boots straight into its
/// steady state: the first-run sequence is done and every coach mark has been
/// seen, so no onboarding step or guided-tour bubble covers a capture.
class CapturePrefs implements PreferencesService {
  CapturePrefs()
    : _store = {
        Onboarding.languagePrefsKey: 'true',
        Onboarding.welcomePrefsKey: 'true',
        for (final hint in CoachHint.values) hint.prefsKey: 'true',
        // The instrument choice is already answered (keyboard), so the home
        // opens on the piano repertoire and the one-time modal can never cover
        // a capture; the drum surfaces are reached through the switcher, the
        // way a user reaches them (change: add-instrument-context).
        InstrumentContext.prefsKey:
            '{"context":"keyboard","choiceOffered":true}',
      };

  final Map<String, String> _store;

  @override
  Future<String?> getString(String key) async => _store[key];
  @override
  Future<void> setString(String key, String value) async => _store[key] = value;
  @override
  Future<void> remove(String key) async => _store.remove(key);
}

/// Always-online connectivity that opens no platform channel — the real plugin
/// is unnecessary here and misbehaves on headless hosts.
class CaptureConnectivity implements ConnectivityService {
  const CaptureConnectivity();
  @override
  Stream<void> get onOnline => const Stream<void>.empty();
  @override
  Stream<bool> get onlineStatus => const Stream<bool>.empty();
  @override
  Future<bool> isOnline() async => true;

  @override
  Future<bool> isDefinitelyOffline() async => false;
}

/// A guest [TokenStore]: no secure storage, no real identity.
class CaptureTokenStore implements TokenStore {
  const CaptureTokenStore();
  @override
  Future<bool> isGuest() async => true;
  @override
  Future<StoredTokens?> readTokens() async => null;
  @override
  Future<void> writeTokens(StoredTokens tokens) async {}
  @override
  Future<void> setGuest() async {}
  @override
  Future<void> clear() async {}
}

/// An [AccountService] that reaches no backend. Its whole job is to guarantee
/// that a capture run can never mutate a real account — `setLocale` in
/// particular, which `AppLocale` would otherwise push for the run's language.
class CaptureAccountService implements AccountService {
  const CaptureAccountService();

  @override
  Future<Account> getAccount() async => throw UnimplementedError();
  @override
  Future<Account> updateHandle({
    required String handle,
    required int expectedVersion,
  }) async => throw UnimplementedError();
  @override
  Future<Account> setLocale(String locale) async => throw UnimplementedError();
  @override
  Future<bool> checkHandleAvailability(String handle) async => false;
  @override
  Future<void> deleteAccount() async {}
  @override
  Future<List<LinkedIdentity>> listIdentities() async => const [];
}

/// No uploads of one's own — the capture account is a listener, not a curator.
class CaptureUploads extends MyUploads {
  @override
  Future<List<ContributedScore>> build() async => const [];
}

/// No separately-saved catalog scores: the favorites list is seeded whole.
class CaptureSavedCatalogScores extends SavedCatalogScores {
  @override
  Future<List<CatalogEntry>> build() async => const [];
}

/// Serves the real curriculum's listing metadata with no backend.
class CaptureCourseCatalog implements CourseCatalogService {
  const CaptureCourseCatalog();

  @override
  Future<List<CourseListing>> listCourses() async => kCaptureCourses;

  /// The captures show the learning *path*, never a lesson's interior, so no
  /// manifest body is needed.
  @override
  Future<String?> getCourseManifestJson(String id) async => null;
}

/// Reports the first [kCaptureCompletedLessons] lessons as done, so the path
/// shows a curriculum in progress instead of an untouched one.
class CaptureCourseProgress implements CourseProgressService {
  const CaptureCourseProgress();

  @override
  Future<Set<String>> completedCourseIds() async => {
    for (final course in kCaptureCourses.take(kCaptureCompletedLessons))
      course.id,
  };

  @override
  Future<void> recordCompletion(String courseId) async {}
}

/// A silent [AudioService] — captures are images.
class CaptureAudioService implements AudioService {
  const CaptureAudioService();

  @override
  Future<void> init() async {}
  @override
  Future<void> loadSoundFont(String sf2Path) async {}
  // "Installed" so a percussion capture is not blocked on readiness — captures
  // are images, the sound itself is irrelevant.
  @override
  Future<bool> loadSoundFontAwaited(String sf2Path) async => true;
  @override
  void noteOn(int pitch, {int velocity = AudioService.defaultVelocity}) {}
  @override
  void noteOff(int pitch) {}
  @override
  void drumOn(int key, {int velocity = AudioService.defaultVelocity}) {}
  @override
  void drumOff(int key) {}
  @override
  void allNotesOff() {}
  @override
  void metronomeClick({required bool accent}) {}
}

/// A connected MIDI instrument the harness plays itself (D4).
///
/// Reporting a connected port is what keeps the "No MIDI device" chip out of
/// every player capture; [press]/[release] then let the scenario perform the
/// piece, so the score gauge shows a real, earned percentage rather than the
/// 0% the August macOS pass shipped.
class CaptureMidiService implements MidiService {
  final StreamController<MidiEvent> _events =
      StreamController<MidiEvent>.broadcast();

  @override
  Stream<MidiEvent> events() => _events.stream;

  @override
  List<String> listPorts() => const [kCaptureMidiDevice];

  @override
  String? connectedPort() => kCaptureMidiDevice;

  @override
  void selectPort(String? name) {}

  /// The harness drives a *simulated* instrument, so there is no engine
  /// callback to echo from: the app keeps sounding these notes itself, which
  /// is exactly what [MidiEcho.off] means (change: add-drum-input-mapping).
  @override
  void setEcho(MidiEcho mode) {}

  /// No mapping is ever pushed for these captures: they drive a keyboard, and
  /// an uncalibrated device is the identity (change:
  /// add-drum-input-calibration).
  @override
  void setMapping(Map<int, int> table) {}

  /// Plays [pitch], as the instrument would.
  ///
  /// [channel] is reported, never enforced — the app's interpretation is
  /// channel-agnostic (change: add-drum-input-calibration) — so the keyboard
  /// default of 0 changes nothing about what these captures drive.
  void press(int pitch, {int velocity = 96, int channel = 0}) => _events.add(
    MidiEvent(
      kind: MidiEventKind.noteOn,
      pitch: pitch,
      velocity: velocity,
      channel: channel,
      timestampMs: BigInt.zero,
    ),
  );

  /// Releases [pitch].
  void release(int pitch, {int channel = 0}) => _events.add(
    MidiEvent(
      kind: MidiEventKind.noteOff,
      pitch: pitch,
      velocity: 0,
      channel: channel,
      timestampMs: BigInt.zero,
    ),
  );

  Future<void> dispose() => _events.close();
}
