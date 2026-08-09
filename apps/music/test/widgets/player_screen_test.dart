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

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/piano_keyboard_painter.dart';
import 'package:music/painters/piano_layout.dart';
import 'package:music/painters/staff_painter.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/platform_info.dart';
import 'package:music/state/countdown.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/theme/cymbra_theme.dart';

import '../support/fakes.dart';
import '../support/localized.dart';

void main() {
  late FakeMidiService midi;
  late RecordingAudioService audio;
  late ProviderContainer container;

  PlayerData state() => container.read(playerProvider);
  Player notifier() => container.read(playerProvider.notifier);

  /// Global position of the center of [pitch]'s key on the on-screen keyboard.
  /// [y] picks the vertical band: ~120 is the white-only region, ~30 the black
  /// band. Mirrors the layout the screen builds from the keyboard width and the
  /// current keyboard bounds.
  Offset keyPos(WidgetTester tester, int pitch, {double y = 120}) {
    final rect = tester.getRect(find.byKey(const Key('onscreen-keyboard')));
    final bounds = state().keyboardBounds;
    final layout = PianoLayout(
      width: rect.width,
      lowPitch: bounds.low,
      highPitch: bounds.high,
    );
    return rect.topLeft + Offset(layout.centerX(pitch), y);
  }

  Future<void> pumpScreen(
    WidgetTester tester, {
    List<String> ports = const ['Piano'],
    String? connected = 'Piano',
    Size size = const Size(1600, 900),
    RecordingAudioService? audioService,
    bool isAndroid = false,
  }) async {
    await tester.binding.setSurfaceSize(size);
    midi = FakeMidiService(ports: ports, connected: connected);
    audio = audioService ?? RecordingAudioService();
    container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
        audioServiceProvider.overrideWithValue(audio),
        // Drive the Android-only OTG guidance deterministically (the test VM
        // would otherwise report the host OS).
        isAndroidProvider.overrideWithValue(isAndroid),
      ],
    );
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: localizedApp(const PlayerScreen()),
      ),
    );
    await tester.pump(); // flush score load + first rebuild
  }

  Future<void> teardownScreen(WidgetTester tester) async {
    await tester.pumpWidget(const SizedBox()); // unmount the screen
    await tester.pump(); // let the auto-dispose provider tear down its timer
    container.dispose();
    await midi.close();
    await tester.binding.setSurfaceSize(null);
  }

  testWidgets('renders title, tempo, and MIDI status (no device name)', (
    tester,
  ) async {
    await pumpScreen(tester);
    expect(find.text('Cymbra Music'), findsOneWidget);
    expect(find.text('Tempo: 80'), findsOneWidget);
    // The status chip shows the connection state, not the device name (that's
    // listed in the settings menu instead).
    expect(find.text('Connected'), findsOneWidget);
    expect(find.text('Piano'), findsNothing);
    await teardownScreen(tester);
  });

  testWidgets('play/pause toggles the transport icon', (tester) async {
    await pumpScreen(tester);
    expect(find.byIcon(Icons.play_arrow), findsOneWidget);
    await tester.tap(find.byIcon(Icons.play_arrow));
    await tester.pump();
    expect(state().isPlaying, isTrue);
    expect(find.byIcon(Icons.pause), findsOneWidget);
    await teardownScreen(tester);
  });

  testWidgets('mode toggle switches Synthesia ⇄ Staff', (tester) async {
    await pumpScreen(tester);
    await tester.tap(find.text('Staff'));
    await tester.pump();
    expect(state().mode, RenderMode.staff);
    await teardownScreen(tester);
  });

  testWidgets('speed and wait-mode controls update state', (tester) async {
    await pumpScreen(tester);
    await tester.tap(find.byIcon(Icons.add));
    await tester.pump();
    expect(state().speed, greaterThan(1.0));
    await tester.tap(find.byIcon(Icons.remove));
    await tester.pump();

    expect(state().waitMode, isTrue);
    await tester.tap(find.text('Wait'));
    await tester.pump();
    expect(state().waitMode, isFalse);
    await teardownScreen(tester);
  });

  /// Opens the settings surface — the pre-play popup reopened in-game (the gear
  /// button). The screen runs a Ticker (never settles), so pump explicitly.
  Future<void> openSettingsPopup(WidgetTester tester) async {
    await tester.tap(find.byIcon(Icons.tune));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));
  }

  /// Taps the popup's Apply button, which commits the drafted settings.
  Future<void> applyPopup(WidgetTester tester) async {
    await tester.tap(find.widgetWithText(FilledButton, 'Apply'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));
  }

  testWidgets('settings popup › keyboard size updates the range mode', (
    tester,
  ) async {
    await pumpScreen(tester);
    // Defaults to auto-fit.
    expect(state().keyboardRange, KeyboardRangeMode.auto);

    // Keyboard size is a dropdown in the popup; open it, pick a non-default fixed
    // size (88 keys), then Apply.
    await openSettingsPopup(tester);
    final dropdown = find.byType(DropdownButton<KeyboardRangeMode>);
    await tester.ensureVisible(dropdown);
    await tester.tap(dropdown);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));
    await tester.tap(find.text('88 keys').last);
    await tester.pump();
    await applyPopup(tester);

    expect(state().keyboardRange, KeyboardRangeMode.keys88);
    await teardownScreen(tester);
  });

  testWidgets('settings popup › MIDI device selects a port', (tester) async {
    await pumpScreen(tester, ports: ['Piano', 'Synth'], connected: 'Piano');
    await openSettingsPopup(tester);
    final dropdown = find.byType(DropdownButton<String?>);
    await tester.ensureVisible(dropdown);
    await tester.tap(dropdown);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));
    await tester.tap(find.text('Synth').last);
    await tester.pump();
    await applyPopup(tester);

    expect(state().connectedDevice, 'Synth');
    await teardownScreen(tester);
  });

  /// Opens the settings popup (the MIDI section is shown inline, no drill-in).
  Future<void> openMidiDeviceCategory(WidgetTester tester) async {
    await openSettingsPopup(tester);
  }

  testWidgets('settings menu › MIDI device shows OTG guidance on Android when '
      'no port is detected', (tester) async {
    await pumpScreen(tester, ports: const [], connected: null, isAndroid: true);
    await openMidiDeviceCategory(tester);

    // Android empty-state: actionable OTG/cable guidance, not the plain row.
    expect(find.textContaining('No MIDI device detected'), findsOneWidget);
    expect(find.textContaining('USB OTG'), findsOneWidget);
    expect(find.text('No device detected'), findsNothing);
    await teardownScreen(tester);
  });

  testWidgets('settings menu › MIDI device hides OTG guidance when a port is '
      'present, and on non-Android when empty', (tester) async {
    // A port is present on Android → guidance is gone (this branch only runs
    // when the list is empty).
    await pumpScreen(
      tester,
      ports: ['Piano'],
      connected: 'Piano',
      isAndroid: true,
    );
    await openMidiDeviceCategory(tester);
    expect(find.textContaining('No MIDI device detected'), findsNothing);
    expect(find.textContaining('USB OTG'), findsNothing);
    await teardownScreen(tester);

    // Empty list but non-Android → the plain "No device detected" row, never the
    // Android-specific OTG guidance.
    await pumpScreen(tester, ports: const [], connected: null);
    await openMidiDeviceCategory(tester);
    expect(find.text('No device detected'), findsOneWidget);
    expect(find.textContaining('USB OTG'), findsNothing);
    await teardownScreen(tester);
  });

  testWidgets('settings popup pauses playback and resumes on close', (
    tester,
  ) async {
    await pumpScreen(tester);
    await tester.tap(find.byIcon(Icons.play_arrow));
    await tester.pump();
    expect(state().isPlaying, isTrue);

    // Opening the settings popup pauses the session.
    await openSettingsPopup(tester);
    expect(state().isPlaying, isFalse);

    // Closing it (Apply) restores the prior play state.
    await applyPopup(tester);
    expect(state().isPlaying, isTrue);
    await teardownScreen(tester);
  });

  testWidgets('right-correct assist key plays the expected right-hand note', (
    tester,
  ) async {
    // Demo notes are staff 1 (right hand); C4 (60) is due at t=0.
    await pumpScreen(tester);
    await tester.sendKeyDownEvent(LogicalKeyboardKey.keyZ);
    await tester.pump();
    expect(state().activeNotes, contains(60));
    await tester.sendKeyUpEvent(LogicalKeyboardKey.keyZ);
    await tester.pump();
    expect(state().activeNotes, isNot(contains(60)));
    await teardownScreen(tester);
  });

  testWidgets('near-miss assist key plays a nearby wrong note', (tester) async {
    await pumpScreen(tester);
    await tester.sendKeyDownEvent(LogicalKeyboardKey.keyS); // right near-miss
    await tester.pump();
    final active = state().activeNotes;
    expect(active, isNotEmpty);
    expect(active, isNot(contains(60))); // never the expected note
    expect(active.every((p) => (p - 60).abs() <= 3), isTrue);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.keyS);
    await tester.pump();
    expect(state().activeNotes, isEmpty);
    await teardownScreen(tester);
  });

  testWidgets('right-correct assist key satisfies Wait Mode', (tester) async {
    await pumpScreen(tester);
    await tester.tap(find.byIcon(Icons.play_arrow)); // play, wait-mode on
    await tester.pump(const Duration(milliseconds: 16));
    await tester.pump();
    expect(state().blocked, isTrue);

    await tester.sendKeyDownEvent(LogicalKeyboardKey.keyZ);
    await tester.pump(const Duration(milliseconds: 16)); // advance unblocks
    await tester.pump();
    expect(state().blocked, isFalse);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.keyZ);
    await teardownScreen(tester);
  });

  testWidgets('near-miss assist key does not satisfy Wait Mode', (
    tester,
  ) async {
    await pumpScreen(tester);
    await tester.tap(find.byIcon(Icons.play_arrow));
    await tester.pump(const Duration(milliseconds: 16));
    await tester.pump();
    expect(state().blocked, isTrue);

    await tester.sendKeyDownEvent(LogicalKeyboardKey.keyS); // right near-miss
    await tester.pump(const Duration(milliseconds: 16));
    await tester.pump();
    expect(state().blocked, isTrue); // wrong note → still blocked
    await tester.sendKeyUpEvent(LogicalKeyboardKey.keyS);
    await teardownScreen(tester);
  });

  testWidgets('former pitch keys no longer produce notes', (tester) async {
    await pumpScreen(tester);
    await tester.sendKeyDownEvent(LogicalKeyboardKey.keyD); // was E4
    await tester.pump();
    expect(state().activeNotes, isEmpty);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.keyD);
    await teardownScreen(tester);
  });

  testWidgets('MIDI indicator shows "No MIDI device" when none detected', (
    tester,
  ) async {
    await pumpScreen(tester, ports: const [], connected: null);
    expect(find.text('No MIDI device'), findsOneWidget);
    await teardownScreen(tester);
  });

  testWidgets('MIDI indicator shows a connecting state', (tester) async {
    await pumpScreen(tester, ports: ['Piano'], connected: null);
    expect(find.text('Connecting…'), findsOneWidget);
    await teardownScreen(tester);
  });

  testWidgets('fits a tablet-width (1024px) window without overflow', (
    tester,
  ) async {
    await pumpScreen(tester, size: const Size(1024, 768));
    expect(tester.takeException(), isNull);
    expect(find.text('Cymbra Music'), findsOneWidget);
    await teardownScreen(tester);
  });

  testWidgets('a narrow, short desktop window lays out without overflow', (
    tester,
  ) async {
    // A shrunken desktop window: the top-bar trailing cluster used to overflow
    // by a few pixels; it now scales down as one block.
    await pumpScreen(tester, size: const Size(820, 460));
    expect(tester.takeException(), isNull);
    expect(find.byKey(const Key('onscreen-keyboard')), findsOneWidget);
    await teardownScreen(tester);
  });

  testWidgets(
    'a blocked cascade pulses the expected keys instead of a banner',
    (tester) async {
      await pumpScreen(tester);

      PianoKeyboardPainter keyboard() =>
          tester
                  .widgetList<CustomPaint>(find.byType(CustomPaint))
                  .firstWhere((w) => w.painter is PianoKeyboardPainter)
                  .painter
              as PianoKeyboardPainter;

      // Steady highlight before playing.
      expect(keyboard().waitPulse, 0);

      await tester.tap(find.byIcon(Icons.play_arrow)); // play, wait-mode on
      await tester.pump(const Duration(milliseconds: 16)); // one ticker frame
      await tester.pump(); // rebuild after blocked=true
      expect(state().blocked, isTrue);
      expect(find.byType(StaffPainter), findsNothing); // still synthesia mode

      // No text banner covers the play surface any more…
      expect(find.text('⏸  Play the expected note to continue'), findsNothing);
      // …the expected keys breathe instead: mid-cycle the pulse is non-zero.
      await tester.pump(const Duration(milliseconds: 550));
      expect(keyboard().waitPulse, greaterThan(0));

      await teardownScreen(tester);
    },
  );

  testWidgets('on-screen key press/release toggles the note', (tester) async {
    await pumpScreen(tester);
    final gesture = await tester.startGesture(keyPos(tester, 60)); // C4
    await tester.pump();
    expect(state().activeNotes, contains(60));
    await gesture.up();
    await tester.pump();
    expect(state().activeNotes, isNot(contains(60)));
    await teardownScreen(tester);
  });

  testWidgets('multi-touch holds two keys and releases independently', (
    tester,
  ) async {
    await pumpScreen(tester);
    final g1 = await tester.startGesture(keyPos(tester, 60)); // C4
    final g2 = await tester.startGesture(keyPos(tester, 62)); // D4
    await tester.pump();
    expect(state().activeNotes, containsAll(<int>[60, 62]));

    // Releasing one pointer note-offs only its pitch.
    await g1.up();
    await tester.pump();
    expect(state().activeNotes, isNot(contains(60)));
    expect(state().activeNotes, contains(62));

    await g2.up();
    await tester.pump();
    expect(state().activeNotes, isNot(contains(62)));
    await teardownScreen(tester);
  });

  testWidgets('on-screen play satisfies the Wait Mode gate', (tester) async {
    await pumpScreen(tester);
    await tester.tap(find.byIcon(Icons.play_arrow)); // play, wait-mode on
    await tester.pump(const Duration(milliseconds: 16)); // one ticker frame
    await tester.pump();
    expect(state().blocked, isTrue); // waiting for C4 (60)

    final gesture = await tester.startGesture(keyPos(tester, 60));
    await tester.pump(const Duration(milliseconds: 16)); // advance unblocks
    await tester.pump();
    expect(state().blocked, isFalse);
    await gesture.up();
    await teardownScreen(tester);
  });

  testWidgets('keyboard responds in every render mode that shows it', (
    tester,
  ) async {
    await pumpScreen(tester);
    // The engraved Partition never shows the keyboard (by design), so it is
    // exercised in the modes that do.
    for (final mode in [RenderMode.synthesia, RenderMode.staff]) {
      container.read(playerProvider.notifier).setMode(mode);
      await tester.pump();
      final gesture = await tester.startGesture(keyPos(tester, 60));
      await tester.pump();
      expect(state().activeNotes, contains(60), reason: 'mode $mode');
      await gesture.up();
      await tester.pump();
      expect(state().activeNotes, isNot(contains(60)), reason: 'mode $mode');
    }
    await teardownScreen(tester);
  });

  /// The speed icon inside the Tempo chip (unique on the screen) — its colour
  /// reflects the metronome's active state.
  Icon tempoIcon(WidgetTester tester) =>
      tester.widget<Icon>(find.byIcon(Icons.speed));

  testWidgets('tapping the Tempo chip toggles the metronome and its style', (
    tester,
  ) async {
    await pumpScreen(tester);
    expect(state().metronomeEnabled, isFalse);
    // Inactive: the icon uses the muted variant colour.
    expect(tempoIcon(tester).color, CymbraColors.onSurfaceVariant);

    await tester.tap(find.text('Tempo: 80'));
    await tester.pump();
    expect(state().metronomeEnabled, isTrue);
    // Active: the icon switches to the primary colour.
    expect(tempoIcon(tester).color, CymbraColors.primary);

    await tester.tap(find.text('Tempo: 80'));
    await tester.pump();
    expect(state().metronomeEnabled, isFalse);
    await teardownScreen(tester);
  });

  testWidgets('the chip pulses on each beat without error', (tester) async {
    await pumpScreen(tester);
    notifier().toggleWaitMode(); // free-run
    notifier().toggleMetronome(); // enable
    await tester.tap(find.byIcon(Icons.play_arrow)); // play
    await tester.pump();
    notifier().advance(kCountdownStartMs); // clear the get-ready countdown

    // Drive a beat boundary through the notifier (the demo beats every 750ms),
    // then let the widget react to the beatCount change.
    notifier().advance(800);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100)); // run the pulse anim

    expect(state().beatCount, greaterThan(0));
    expect(tester.takeException(), isNull);
    await teardownScreen(tester);
  });

  testWidgets('visual pulse still updates when audio is unavailable', (
    tester,
  ) async {
    // A failed audio engine: clicks are no-ops at the seam, but the visual beat
    // (beatCount) must still advance and the chip must not crash.
    await pumpScreen(
      tester,
      audioService: RecordingAudioService(failInit: true),
    );
    notifier().toggleWaitMode();
    notifier().toggleMetronome();
    await tester.tap(find.byIcon(Icons.play_arrow));
    await tester.pump();
    notifier().advance(kCountdownStartMs); // clear the get-ready countdown

    notifier().advance(800);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(state().beatCount, greaterThan(0));
    expect(tester.takeException(), isNull);
    await teardownScreen(tester);
  });

  group('hide-keyboard toggle', () {
    testWidgets('hides the on-screen keyboard in a notation mode', (
      tester,
    ) async {
      await pumpScreen(tester);
      notifier().setMode(RenderMode.staff);
      notifier().setKeyboardVisible(false);
      await tester.pump();
      expect(find.byKey(const Key('onscreen-keyboard')), findsNothing);
      await teardownScreen(tester);
    });

    testWidgets('keeps the keyboard in Synthesia even when set hidden', (
      tester,
    ) async {
      await pumpScreen(tester);
      notifier().setMode(RenderMode.synthesia);
      notifier().setKeyboardVisible(false);
      await tester.pump();
      // Synthesia's cascade aligns to the keyboard, so it stays visible.
      expect(find.byKey(const Key('onscreen-keyboard')), findsOneWidget);
      await teardownScreen(tester);
    });

    Future<void> openSettings(WidgetTester tester) async {
      await tester.tap(find.byIcon(Icons.tune));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 300));
    }

    testWidgets('the settings category is hidden in Synthesia', (tester) async {
      await pumpScreen(tester); // default mode is Synthesia
      await openSettings(tester);
      expect(find.text('Keyboard display'), findsNothing);
      await teardownScreen(tester);
    });

    testWidgets('the settings category is offered in a notation mode', (
      tester,
    ) async {
      await pumpScreen(tester);
      notifier().setMode(RenderMode.staff);
      await tester.pump();
      await openSettings(tester);
      expect(find.text('Keyboard display'), findsOneWidget);
      await teardownScreen(tester);
    });

    testWidgets('toggling the visibility switch off hides the keyboard', (
      tester,
    ) async {
      await pumpScreen(tester);
      notifier().setMode(RenderMode.staff);
      await tester.pump();
      await openSettings(tester);
      // The visibility control is a switch (default on); toggle it off, Apply.
      final tile = find.widgetWithText(SwitchListTile, 'Keyboard display');
      await tester.ensureVisible(tile);
      await tester.tap(tile);
      await tester.pump();
      await tester.tap(find.widgetWithText(FilledButton, 'Apply'));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 300));
      expect(state().keyboardVisible, isFalse);
      expect(find.byKey(const Key('onscreen-keyboard')), findsNothing);
      await teardownScreen(tester);
    });

    testWidgets('the Partition never shows the keyboard nor offers the toggle', (
      tester,
    ) async {
      await pumpScreen(tester);
      notifier().setMode(RenderMode.partition);
      notifier().setKeyboardVisible(true); // even explicitly visible
      await tester.pump();
      // The engraving takes the full height; the expected-note emphasis in the
      // notation is the "what to play" cue.
      expect(find.byKey(const Key('onscreen-keyboard')), findsNothing);
      await openSettings(tester);
      expect(find.text('Keyboard display'), findsNothing);
      await teardownScreen(tester);
    });
  });

  group('adaptive smartphone layout', () {
    // A phone / tablet landscape viewport (shortest side 375 / 768), plus a
    // deliberately narrow phone (iPhone-SE-class) to stress the top-bar fit and
    // a desktop-class viewport (shortest side ≥ 900) for the bottom-bar path.
    const phone = Size(812, 375);
    const smallPhone = Size(667, 375);
    const tablet = Size(1024, 768);
    const desktop = Size(1600, 1000);

    // Forces the target platform to iOS so the size-based device-class path
    // drives the layout (not the desktop-platform override), then resets it
    // (and the view) before the test body ends — the framework asserts
    // foundation debug vars are clear before tearDown, so an addTearDown reset
    // would be too late.
    Future<void> onMobile(
      WidgetTester tester,
      Future<void> Function() body,
    ) async {
      debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
      try {
        await body();
      } finally {
        debugDefaultTargetPlatformOverride = null;
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      }
    }

    // Pumps the player at [size]. `setSurfaceSize` (via pumpScreen) drives the
    // render constraints, but `MediaQuery.size` — which the device-class helper
    // reads — comes from the view, so we set the view too (dpr 1 ⇒ logical ==
    // physical) to keep both in agreement at the intended device size.
    Future<void> pumpAt(WidgetTester tester, Size size) async {
      tester.view.devicePixelRatio = 1.0;
      tester.view.physicalSize = size;
      await pumpScreen(tester, size: size);
    }

    double keyboardHeight(WidgetTester tester) =>
        tester.getSize(find.byKey(const Key('onscreen-keyboard'))).height;

    double titleFontSize(WidgetTester tester) =>
        tester.widget<Text>(find.text('Cymbra Music')).style!.fontSize!;

    testWidgets(
      'keyboard shrinks on a phone vs a tablet, within the legible clamp',
      (tester) async {
        await onMobile(tester, () async {
          await pumpAt(tester, phone);
          final phoneKb = keyboardHeight(tester);
          await teardownScreen(tester);

          await pumpAt(tester, tablet);
          final tabletKb = keyboardHeight(tester);
          await teardownScreen(tester);

          // Clamp bands mirror _PlayerScreenState: phones use the shorter
          // 78..108 band, tablet/desktop the 96..150 band.
          expect(phoneKb, inInclusiveRange(78, 108));
          expect(tabletKb, inInclusiveRange(96, 150));
          expect(
            phoneKb,
            lessThan(tabletKb),
            reason: 'the shorter phone viewport yields a shorter keyboard',
          );
        });
      },
    );

    testWidgets(
      'the narrowest phone lays out without overflow and keeps a render area',
      (tester) async {
        await onMobile(tester, () async {
          await pumpAt(tester, smallPhone);
          expect(tester.takeException(), isNull);
          // The keyboard takes only part of the column; the render area above
          // it (plus the top bar) is taller than the keyboard itself, so the
          // render area keeps a usable, non-zero height.
          final kb = tester.getRect(find.byKey(const Key('onscreen-keyboard')));
          expect(kb.top, greaterThan(0));
          expect(
            kb.height,
            lessThan(kb.top),
            reason: 'content above the keyboard exceeds its height',
          );
          await teardownScreen(tester);
        });
      },
    );

    testWidgets('top bar uses compact type on a phone, full type on a tablet', (
      tester,
    ) async {
      await onMobile(tester, () async {
        await pumpAt(tester, phone);
        final phoneTitle = titleFontSize(tester);
        await teardownScreen(tester);

        await pumpAt(tester, tablet);
        final tabletTitle = titleFontSize(tester);
        await teardownScreen(tester);

        expect(phoneTitle, 15, reason: 'phone uses the compact title size');
        expect(tabletTitle, 18, reason: 'tablet keeps the full title size');
      });
    });

    testWidgets('Partition mode is offered on a tablet but hidden on a phone', (
      tester,
    ) async {
      await onMobile(tester, () async {
        await pumpAt(tester, phone);
        // Icon-only toggle on a phone offers Synthesia + Staff only — the
        // Partition segment (Icons.article) is dropped.
        expect(find.byIcon(Icons.waterfall_chart), findsOneWidget);
        expect(find.byIcon(Icons.music_note), findsOneWidget);
        expect(find.byIcon(Icons.article), findsNothing);
        await teardownScreen(tester);

        await pumpAt(tester, tablet);
        // The labelled tablet toggle still includes Partition.
        expect(find.text('Partition'), findsOneWidget);
        await teardownScreen(tester);
      });
    });

    testWidgets('a Partition mode set before switching to a phone renders '
        'without error and offers no Partition segment', (tester) async {
      await onMobile(tester, () async {
        await pumpAt(tester, phone);
        // Coerce the mode to Partition (as if set on a larger screen), then
        // rebuild on the phone layout: no crash, and Staff is the selection.
        container.read(playerProvider.notifier).setMode(RenderMode.partition);
        await tester.pump();
        expect(tester.takeException(), isNull);
        expect(find.byIcon(Icons.article), findsNothing);
        await teardownScreen(tester);
      });
    });

    testWidgets('bottom safe area is dropped on a phone, kept on a tablet', (
      tester,
    ) async {
      Iterable<bool> bottomInsets(WidgetTester tester) => tester
          .widgetList<SafeArea>(
            find.ancestor(
              of: find.byKey(const Key('transport-bar')),
              matching: find.byType(SafeArea),
            ),
          )
          .map((s) => s.bottom);

      await onMobile(tester, () async {
        await pumpAt(tester, phone);
        expect(
          bottomInsets(tester),
          contains(false),
          reason: 'phone lets the keyboard reach the bottom edge',
        );
        await teardownScreen(tester);

        await pumpAt(tester, tablet);
        expect(
          bottomInsets(tester),
          isNot(contains(false)),
          reason: 'tablet keeps the full safe area',
        );
        await teardownScreen(tester);
      });
    });

    testWidgets('transport controls rail on the right on every form factor', (
      tester,
    ) async {
      // A right-side vertical rail: taller than wide, sitting to the right of
      // the keyboard rather than below it — clear of the bottom home indicator.
      Future<void> expectRail(WidgetTester tester, Size size) async {
        await pumpAt(tester, size);
        final bar = tester.getRect(find.byKey(const Key('transport-bar')));
        final kb = tester.getRect(find.byKey(const Key('onscreen-keyboard')));
        expect(
          bar.height,
          greaterThan(bar.width),
          reason: 'controls form a vertical side rail at $size',
        );
        expect(
          bar.left,
          greaterThanOrEqualTo(kb.right - 1),
          reason: 'the rail sits to the right of the keyboard at $size',
        );
        await teardownScreen(tester);
      }

      await onMobile(tester, () async {
        // Every form factor rails on the right: phones clear the home
        // indicator, and short desktop windows keep the full height for the
        // notation + keyboard.
        await expectRail(tester, phone);
        await expectRail(tester, tablet);
      });
      await expectRail(tester, desktop);
    });

    testWidgets('the tablet rail keeps the full "% SPD" label and a labelled '
        'Wait, the phone rail slims them', (tester) async {
      await onMobile(tester, () async {
        // Phone (slim rail): the "SPD" suffix is dropped and Wait is icon-only
        // (a plain "100%" label would clash with the scoring overlay's own
        // percentage, so we assert on the suffix that's unique to the speed
        // label instead).
        await pumpAt(tester, phone);
        expect(find.text('100% SPD'), findsNothing);
        expect(find.widgetWithText(TextButton, 'Wait'), findsNothing);
        await teardownScreen(tester);

        // Tablet (roomier rail): the full label and the labelled Wait button.
        await pumpAt(tester, tablet);
        expect(find.text('100% SPD'), findsOneWidget);
        expect(find.widgetWithText(TextButton, 'Wait'), findsOneWidget);
        await teardownScreen(tester);
      });
    });
  });
}
