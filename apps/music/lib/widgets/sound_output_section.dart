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

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../state/audio_routing.dart';
import '../state/player_data.dart';
import '../state/player_preferences.dart';
import '../state/player_notifier.dart';
import '../theme/cymbra_theme.dart';
import 'app_snackbar.dart';

/// Starting offset offered when a wireless route becomes active — the middle of
/// the 100–300 ms a Bluetooth transport typically adds. Only ever *suggested*:
/// the real delay is the user's headset's, not a number the app can measure.
const int kSuggestedWirelessOffsetMs = 200;

/// The largest offset the slider offers. Past this the reference no longer
/// describes any real transport (the preferences store clamps harder still).
const int kMaxOutputOffsetMs = 500;

/// "Sound output": where the app's audio goes, plus the two settings that only
/// make sense next to it — the instrument-sounds-itself rule and the delay
/// compensation (change: add-audio-output-routing).
///
/// One shape, two platform realities: a device list on desktop, the active route
/// plus the system picker on mobile. Wrap it in a [SoundOutputListener] so the
/// section's side effects stay out of `build`.
class SoundOutputSection extends ConsumerWidget {
  const SoundOutputSection({super.key, this.title});

  /// Section heading builder used by the host modal so the section matches the
  /// ones around it. Falls back to a plain label.
  final Widget Function(String label)? title;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final routing = ref.watch(audioRoutingProvider);
    final data = ref.watch(playerProvider);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        title?.call(l10n.soundOutputTitle) ??
            _DefaultTitle(l10n.soundOutputTitle),
        if (routing.canSelectDevice)
          _OutputDevicePicker(routing: routing)
        else
          _ActiveRouteRow(routing: routing),
        if (routing.isWireless) ...[
          const SizedBox(height: 8),
          _WirelessWarning(message: l10n.soundOutputWirelessWarning),
          _OutputOffsetControl(offsetMs: data.outputOffsetMs),
        ],
        _InstrumentSoundsItselfTile(data: data),
        _ScoreAudioMutedTile(data: data),
      ],
    );
  }
}

/// Isolates the section's side effects — the localized message raised when a
/// device refuses to open — so no `ref.listen` lives in a `build` method and no
/// caller awaits a notifier action to learn what happened.
///
/// A **toast**, not a snackbar: the section lives inside the setup dialog, and a
/// snackbar is painted by the route below it.
class SoundOutputListener extends ConsumerWidget {
  const SoundOutputListener({super.key, required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.listen(audioRoutingProvider.select((s) => s.selectionFailed), (
      _,
      failed,
    ) {
      if (!failed) return;
      showAppToast(
        Overlay.of(context, rootOverlay: true),
        AppLocalizations.of(context).soundOutputSelectionFailed,
      );
      ref.read(audioRoutingProvider.notifier).acknowledgeFailure();
    });
    return child;
  }
}

class _DefaultTitle extends StatelessWidget {
  const _DefaultTitle(this.label);

  final String label;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(top: 12, bottom: 2),
    child: Text(
      label,
      style: const TextStyle(
        color: CymbraColors.onSurfaceVariant,
        fontSize: 12,
        fontWeight: FontWeight.w700,
        letterSpacing: 0.4,
      ),
    ),
  );
}

/// Desktop: the host's output devices, with "system default" first. The value
/// shown is the device **actually** in use, so a fallback is visible.
class _OutputDevicePicker extends ConsumerWidget {
  const _OutputDevicePicker({required this.routing});

  final AudioRoutingState routing;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final names = routing.outputs.map((o) => o.name).toList();
    // The control shows the user's **choice**, not the device in use: showing the
    // active device made every selection look ignored, because choosing an output
    // that resolves to the same device snapped the value straight back. Reality
    // is reported underneath instead.
    final chosen = ref.watch(
      playerPreferencesProvider.select((p) => p.audioOutput),
    );
    final active = routing.active?.name;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12),
          decoration: BoxDecoration(
            color: CymbraColors.surfaceContainerHigh,
            borderRadius: BorderRadius.circular(10),
          ),
          child: DropdownButtonHideUnderline(
            child: DropdownButton<String?>(
              key: const Key('sound-output-device'),
              isExpanded: true,
              value: names.contains(chosen) ? chosen : null,
              dropdownColor: CymbraColors.surfaceContainerHigh,
              iconEnabledColor: CymbraColors.onSurfaceVariant,
              style: const TextStyle(color: CymbraColors.onSurface),
              items: [
                DropdownMenuItem<String?>(
                  value: null,
                  child: Text(l10n.soundOutputSystemDefault),
                ),
                for (final route in routing.outputs)
                  DropdownMenuItem<String?>(
                    value: route.name,
                    // USB on Android is an informed opt-in: the platform's USB
                    // path is unreliable below the app, so the label says so
                    // rather than letting a broken route look endorsed.
                    child: Text(
                      routing.usbExperimental &&
                              route.kind == AudioRouteKind.usb
                          ? l10n.soundOutputExperimentalDevice(route.name)
                          : route.name,
                    ),
                  ),
              ],
              onChanged: (name) =>
                  ref.read(audioRoutingProvider.notifier).selectOutput(name),
            ),
          ),
        ),
        // What is *actually* in use, which after a fallback is not what was asked
        // for. Stated separately so the control can express intent and this can
        // state reality.
        if (active != null && active != chosen)
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Text(
              l10n.soundOutputActuallyUsing(active),
              style: const TextStyle(
                fontSize: 12,
                color: CymbraColors.onSurfaceVariant,
              ),
            ),
          ),
      ],
    );
  }
}

/// Mobile: the OS owns the route, so the app shows what is active and offers the
/// system's own picker rather than a list it could not honor.
class _ActiveRouteRow extends ConsumerWidget {
  const _ActiveRouteRow({required this.routing});

  final AudioRoutingState routing;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final route = routing.active;
    return Container(
      key: const Key('sound-output-route'),
      padding: const EdgeInsets.fromLTRB(12, 4, 4, 4),
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(10),
      ),
      child: Row(
        children: [
          Icon(_iconFor(route?.kind), color: CymbraColors.onSurfaceVariant),
          const SizedBox(width: 10),
          Expanded(
            child: Text(
              route?.name ?? l10n.soundOutputUnknownRoute,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(color: CymbraColors.onSurface),
            ),
          ),
          TextButton(
            key: const Key('sound-output-route-picker'),
            onPressed: () =>
                ref.read(audioRoutingProvider.notifier).presentRoutePicker(),
            child: Text(l10n.soundOutputChangeRoute),
          ),
        ],
      ),
    );
  }

  static IconData _iconFor(AudioRouteKind? kind) => switch (kind) {
    AudioRouteKind.bluetooth => Icons.bluetooth_audio,
    AudioRouteKind.headphones => Icons.headphones,
    AudioRouteKind.usb => Icons.usb,
    AudioRouteKind.builtin => Icons.speaker,
    _ => Icons.volume_up,
  };
}

/// Says plainly that a wireless route suits listening, not playing. Driven by
/// the route's *kind*, never by matching its name.
class _WirelessWarning extends StatelessWidget {
  const _WirelessWarning({required this.message});

  final String message;

  @override
  Widget build(BuildContext context) => Container(
    key: const Key('sound-output-wireless-warning'),
    padding: const EdgeInsets.all(10),
    decoration: BoxDecoration(
      color: CymbraColors.surfaceContainerHigh,
      borderRadius: BorderRadius.circular(10),
      border: Border.all(color: CymbraColors.primary.withValues(alpha: 0.35)),
    ),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Icon(
          Icons.timer_outlined,
          size: 18,
          color: CymbraColors.onSurfaceVariant,
        ),
        const SizedBox(width: 8),
        Expanded(
          child: Text(
            message,
            style: const TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 12,
            ),
          ),
        ),
      ],
    ),
  );
}

/// Delay compensation, revealed only while a wireless route is active so it does
/// not clutter the common case. A starting value is **suggested**, never applied
/// on the user's behalf.
class _OutputOffsetControl extends ConsumerWidget {
  const _OutputOffsetControl({required this.offsetMs});

  final int offsetMs;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final notifier = ref.read(playerProvider.notifier);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const SizedBox(height: 8),
        Row(
          children: [
            Expanded(
              child: Text(
                l10n.outputOffsetTitle,
                style: const TextStyle(
                  color: CymbraColors.onSurface,
                  fontSize: 13,
                  fontWeight: FontWeight.w700,
                ),
              ),
            ),
            Text(
              l10n.outputOffsetValue(offsetMs),
              style: const TextStyle(color: CymbraColors.onSurfaceVariant),
            ),
          ],
        ),
        Slider(
          key: const Key('sound-output-offset'),
          value: offsetMs.clamp(0, kMaxOutputOffsetMs).toDouble(),
          max: kMaxOutputOffsetMs.toDouble(),
          divisions: kMaxOutputOffsetMs ~/ 10,
          label: l10n.outputOffsetValue(offsetMs),
          onChanged: (v) => notifier.setOutputOffsetMs(v.round()),
        ),
        Text(
          l10n.outputOffsetHint,
          style: const TextStyle(
            color: CymbraColors.onSurfaceVariant,
            fontSize: 12,
          ),
        ),
        // Suggested only while the user has not set anything: proposing a number
        // over a value they chose would be second-guessing them.
        if (offsetMs == 0)
          Row(
            children: [
              Expanded(
                child: Text(
                  l10n.outputOffsetSuggestion(kSuggestedWirelessOffsetMs),
                  style: const TextStyle(
                    color: CymbraColors.onSurfaceVariant,
                    fontSize: 12,
                  ),
                ),
              ),
              TextButton(
                key: const Key('sound-output-offset-suggestion'),
                onPressed: () =>
                    notifier.setOutputOffsetMs(kSuggestedWirelessOffsetMs),
                child: Text(
                  l10n.outputOffsetApplySuggestion(kSuggestedWirelessOffsetMs),
                ),
              ),
            ],
          ),
      ],
    );
  }
}

/// The instrument-sounds-itself switch. Shown disabled *with its reason* when no
/// MIDI port is connected: the rule only ever suppresses notes arriving from an
/// instrument, so without one it would silently do nothing.
class _InstrumentSoundsItselfTile extends ConsumerWidget {
  const _InstrumentSoundsItselfTile({required this.data});

  final PlayerData data;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final available = data.instrumentSoundsItselfAvailable;
    return SwitchListTile(
      key: const Key('instrument-sounds-itself'),
      contentPadding: EdgeInsets.zero,
      title: Text(
        l10n.instrumentSoundsItselfTitle,
        style: TextStyle(
          color: available
              ? CymbraColors.onSurface
              : CymbraColors.onSurfaceVariant,
        ),
      ),
      subtitle: Text(
        available
            ? l10n.instrumentSoundsItselfBody
            : l10n.instrumentSoundsItselfUnavailable,
        style: const TextStyle(
          color: CymbraColors.onSurfaceVariant,
          fontSize: 12,
        ),
      ),
      value: data.instrumentSoundsItself,
      onChanged: available
          ? (v) => ref
                .read(playerProvider.notifier)
                .setInstrumentSoundsItself(enabled: v)
          : null,
    );
  }
}

/// The written-score mute (change: add-practice-focus-controls) — the switch
/// beside [_InstrumentSoundsItselfTile] and its exact counterpart: that one
/// silences the notes the player *plays*, this one the notes the app *asks for*.
///
/// Always available, unlike its neighbour: it needs no instrument, because the
/// thing it silences is the app's own playback. Nothing else about the session
/// changes — the score is still drawn, still gated on, still judged, and the
/// metronome still clicks.
class _ScoreAudioMutedTile extends ConsumerWidget {
  const _ScoreAudioMutedTile({required this.data});

  final PlayerData data;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    return SwitchListTile(
      key: const Key('score-audio-muted'),
      contentPadding: EdgeInsets.zero,
      title: Text(
        l10n.scoreAudioMutedTitle,
        style: const TextStyle(color: CymbraColors.onSurface),
      ),
      subtitle: Text(
        l10n.scoreAudioMutedBody,
        style: const TextStyle(
          color: CymbraColors.onSurfaceVariant,
          fontSize: 12,
        ),
      ),
      value: data.scoreAudioMuted,
      onChanged: (v) =>
          ref.read(playerProvider.notifier).setScoreAudioMuted(muted: v),
    );
  }
}
