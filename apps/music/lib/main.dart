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

import 'package:cymbra_flags/cymbra_flags.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'dart:async';

import 'l10n/gen/app_localizations.dart';
import 'screens/onboarding/onboarding_gate.dart';
import 'services/audio_capture_service.dart';
import 'services/audio_service.dart';
import 'services/flags_integration.dart';
import 'services/license_notices.dart';
import 'src/rust/frb_generated.dart';
import 'state/app_locale.dart';
import 'state/audio_routing.dart';
import 'state/catalog_daily_access_notifier.dart';
import 'widgets/post_play_rating.dart';
import 'state/foreground_notification_listener.dart';
import 'state/language_sync_listener.dart';
import 'state/push_registration_listener.dart';
import 'state/score_preview_playback.dart';
import 'state/selected_piano.dart';
import 'state/acoustic_input_access.dart';
import 'state/drums_access.dart';
import 'state/plan_notifier.dart';
import 'state/usage_tracking_notifier.dart';
import 'theme/cymbra_theme.dart';
import 'widgets/coach_layer.dart';
import 'widgets/foreground_notification_banner.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  // Third-party Rust crate notices for the "Open Source Licenses" screen
  // (change: add-oss-license-attributions), alongside the Dart/Flutter pub
  // licenses Flutter already registers on its own.
  registerRustLicenseNotices();
  // Lock to landscape: the on-screen keyboard (up to 88 keys) is only legible in
  // landscape. No-op on desktop/web.
  await SystemChrome.setPreferredOrientations(const [
    DeviceOrientation.landscapeLeft,
    DeviceOrientation.landscapeRight,
  ]);
  await RustLib.init();

  // Pre-warm the piano synth at launch (loads the ~50 MB SoundFont) so it is
  // ready before the user picks a piece — keeping the heavy one-time load off
  // the score-selection path. The container is shared with the app so the
  // player reuses this already-initialized AudioService instance.
  final container = ProviderContainer(
    overrides: [
      ...cymbraFlagOverrides(),
      // Wire the usage-collection kill-switch to the real remote flag in the app
      // (its default is a plain `true` so tests never build the flag client).
      usageCollectionKillSwitchProvider.overrideWith(
        (ref) => ref
            .watch(flagsProvider)
            .getBool(kAnalyticsCollectionFlag, or: true),
      ),
      // The plan system kill-switch (change: add-premium-subscription): plain
      // `false` in tests, the remote `plans.enabled` flag in the app.
      plansEnabledProvider.overrideWith(
        (ref) => ref.watch(flagsProvider).getBool('plans.enabled', or: false),
      ),
      // Drum-feature visibility (change: add-drums-access): plain `false` in
      // tests, the remote server-evaluated `drums.enabled` flag in the app.
      drumsEnabledProvider.overrideWith(
        (ref) => ref.watch(flagsProvider).getBool(kDrumsEnabledFlag, or: false),
      ),
      // Microphone input visibility (change: add-acoustic-piano-input): plain
      // `false` in tests, the remote `acoustic_input.enabled` flag in the app.
      acousticInputEnabledProvider.overrideWith(
        (ref) => ref
            .watch(flagsProvider)
            .getBool(kAcousticInputEnabledFlag, or: false),
      ),
    ],
  );
  unawaited(container.read(audioServiceProvider).init());

  // Restore the user's chosen piano and swap the synth to it once audio is up
  // (a no-op for the default, which init already loaded). Reading the provider
  // kicks off its persisted-selection restore at launch, so the choice survives
  // relaunches without waiting for the settings drawer to be opened.
  container.read(selectedPianoProvider);

  // Same for the chosen sound output (change: add-audio-output-routing): the
  // routing notifier's restore re-applies the remembered device, so the first
  // sound of the session already goes where the user sent it — not only once
  // the sound-output section has been opened.
  container.read(audioRoutingProvider);

  // Fetch the caller's effective feature flags on launch (identity-scoped,
  // flicker-free from the persisted cache); the observer refreshes on resume.
  container.read(flagsProvider);

  // Start feature-usage tracking (change: add-feature-usage-analytics): warm the
  // tracker (delivers any events left from a previous run) + its background flush
  // scheduler (periodic + connectivity). The scheduler is warmed ONLY here so it
  // never spins up timers/plugins in a test that merely exercises an instrumented
  // call site. Emission stays gated on consent + the remote kill-switch.
  container.read(usageTrackingNotifierProvider);
  container.read(usageFlushSchedulerProvider);

  // Silence the synth when the OS backgrounds/hides the app, so a held voice
  // (note pressed, no note-off yet) doesn't keep ringing while paused; refresh
  // the feature flags when the app returns to the foreground.
  WidgetsBinding.instance.addObserver(_AudioLifecycleObserver(container));

  runApp(
    UncontrolledProviderScope(container: container, child: const CymbraApp()),
  );
}

/// Cuts all audio when the app leaves the foreground. `paused`/`hidden` cover
/// mobile backgrounding (and desktop minimise); `inactive` is intentionally not
/// silenced so a brief focus change on desktop doesn't chop a sounding note.
class _AudioLifecycleObserver with WidgetsBindingObserver {
  _AudioLifecycleObserver(this._container);

  final ProviderContainer _container;

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.paused ||
        state == AppLifecycleState.hidden ||
        state == AppLifecycleState.detached) {
      _container.read(audioServiceProvider).allNotesOff();
      unawaited(_container.read(scorePreviewPlaybackProvider.notifier).stop());
      // The microphone never captures in the background (change:
      // add-acoustic-piano-input, spec: Microphone Capture Lifecycle). The
      // player re-opens its session on its next sync if it is still alive.
      final capture = _container.read(audioCaptureServiceProvider);
      capture.stopDetection();
      unawaited(capture.endCapture());
    } else if (state == AppLifecycleState.resumed) {
      // Re-fetch effective flags on foreground (cheap when unchanged via the
      // version/ETag) so a kill-switch flip is picked up without a restart.
      unawaited(_container.read(flagsProvider.notifier).refresh());
      // The daily free-open quota may have rolled over while backgrounded
      // (change: add-score-daily-access-rewards).
      unawaited(_container.read(catalogDailyAccessProvider.notifier).refresh());
    }
  }
}

/// Root app. `home` is the [OnboardingGate], which runs the first-run language
/// and welcome steps (no account required) before handing over to the session
/// routing (entry screen, guest/library, or handle onboarding).
class CymbraApp extends ConsumerWidget {
  const CymbraApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Drive the locale from state: selecting a language rebuilds MaterialApp and
    // re-resolves every AppLocalizations.of(context) — an immediate, restart-free
    // switch. The title stays the fixed brand.
    final locale = ref.watch(appLocaleProvider);
    return MaterialApp(
      title: 'Cymbra Music',
      debugShowCheckedModeBanner: false,
      theme: buildCymbraTheme(),
      locale: locale,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      // The coaching layer sits ABOVE the navigator so a spotlight can point at
      // a control that lives inside a dialog (the pre-play setup surface); the
      // foreground-notification banner sits there too, so it paints over
      // whatever screen is open (change: add-foreground-notifications).
      builder: (context, child) => Stack(
        children: [
          ?child,
          const ForegroundNotificationLayer(),
          const CoachLayer(),
        ],
      ),
      // Reconcile the account language into the UI after sign-in (change:
      // sync-account-language-preference), register this device for push
      // (change: add-push-notifications), and surface foreground notifications
      // (change: add-foreground-notifications) — each isolated in a dedicated
      // listener.
      home: const PostPlayRatingToastListener(
        child: LanguageSyncListener(
          child: PushRegistrationListener(
            child: ForegroundNotificationListener(child: OnboardingGate()),
          ),
        ),
      ),
    );
  }
}
