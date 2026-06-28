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

import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../painters/partition_painter.dart';
import '../painters/piano_keyboard_painter.dart';
import '../painters/piano_layout.dart';
import '../painters/staff_painter.dart';
import '../painters/synthesia_painter.dart';
import '../src/rust/api/musicxml.dart' show System;
import '../state/notation_data.dart';
import '../state/notation_notifier.dart';
import '../state/player_data.dart';
import '../state/player_notifier.dart';
import '../theme/cymbra_theme.dart';

/// Main screen of the Cymbra player: top bar, rendering area
/// (Synthesia or Staff), keyboard, and transport bar.
class PlayerScreen extends ConsumerStatefulWidget {
  const PlayerScreen({super.key});

  @override
  ConsumerState<PlayerScreen> createState() => _PlayerScreenState();
}

class _PlayerScreenState extends ConsumerState<PlayerScreen>
    with SingleTickerProviderStateMixin {
  final FocusNode _focusNode = FocusNode();
  late final Ticker _ticker;
  Duration _lastTick = Duration.zero;

  /// Active on-screen-keyboard pointers → the pitch each is holding, so a finger
  /// release note-offs only its own pitch (independent multi-touch). Same-pitch
  /// from multiple sources is last-release-wins: releasing one source clears the
  /// shared [PlayerData.activeNotes] entry even if another still holds it — an
  /// accepted v1 simplification (chords use distinct pitches).
  final Map<int, int> _keyboardPointers = {};

  static const double _keyboardHeight = 150;

  /// Random source for the near-miss assist keys (q/s).
  final math.Random _rng = math.Random();

  /// Assist keys → the pitches each fired on key-down, so key-up note-offs
  /// exactly those (a near-miss picks a fresh random pitch each press).
  final Map<LogicalKeyboardKey, Set<int>> _assistPressed = {};

  /// Whether playback was running when the settings drawer opened, so closing it
  /// restores that state (the drawer pauses the session while open).
  bool _wasPlayingBeforeDrawer = false;

  /// Lets [_onEndDrawerChanged] reset the drawer to its category list each time
  /// it opens (its navigation state otherwise persists across open/close).
  final GlobalKey<_SettingsDrawerState> _settingsDrawerKey = GlobalKey();

  /// Pause the session while the settings drawer is open; restore the prior
  /// play/pause state when it closes. Also resets the drawer to its root.
  void _onEndDrawerChanged(bool isOpen) {
    final notifier = ref.read(playerProvider.notifier);
    if (isOpen) {
      _settingsDrawerKey.currentState?.resetToRoot();
      _wasPlayingBeforeDrawer = ref.read(playerProvider).isPlaying;
      if (_wasPlayingBeforeDrawer) notifier.setPlaying(false);
    } else {
      if (_wasPlayingBeforeDrawer) notifier.setPlaying(true);
      _wasPlayingBeforeDrawer = false;
    }
  }

  @override
  void initState() {
    super.initState();
    _ticker = createTicker(_onTick)..start();
  }

  void _onTick(Duration elapsed) {
    final dt = (elapsed - _lastTick).inMicroseconds / 1000.0; // ms
    _lastTick = elapsed;
    if (dt > 0 && dt < 100) {
      final speed = ref.read(playerProvider).speed;
      ref.read(playerProvider.notifier).advance(dt * speed);
    }
  }

  /// Desktop keyboard = four practice-assist keys (AZERTY 2×2 cluster):
  ///   a = left-hand correct,  z = right-hand correct,
  ///   q = left-hand near-miss, s = right-hand near-miss.
  /// The correct keys play all notes expected for that hand at the playhead
  /// (satisfying Wait Mode); the near-miss keys play a random nearby wrong note.
  /// Exact, arbitrary notes are played with the on-screen keyboard instead.
  KeyEventResult _onKey(FocusNode node, KeyEvent event) {
    final key = event.logicalKey;
    final bool rightHand;
    final bool nearMiss;
    if (key == LogicalKeyboardKey.keyA) {
      rightHand = false;
      nearMiss = false;
    } else if (key == LogicalKeyboardKey.keyZ) {
      rightHand = true;
      nearMiss = false;
    } else if (key == LogicalKeyboardKey.keyQ) {
      rightHand = false;
      nearMiss = true;
    } else if (key == LogicalKeyboardKey.keyS) {
      rightHand = true;
      nearMiss = true;
    } else {
      return KeyEventResult.ignored;
    }

    final notifier = ref.read(playerProvider.notifier);
    if (event is KeyDownEvent) {
      if (_assistPressed.containsKey(key)) {
        return KeyEventResult.handled; // already held; ignore stray repeat
      }
      final pitches = _assistPitches(rightHand: rightHand, nearMiss: nearMiss);
      if (pitches.isEmpty) return KeyEventResult.handled;
      _assistPressed[key] = pitches;
      for (final p in pitches) {
        notifier.noteOn(p);
      }
      return KeyEventResult.handled;
    } else if (event is KeyUpEvent) {
      final pitches = _assistPressed.remove(key);
      if (pitches != null) {
        for (final p in pitches) {
          notifier.noteOff(p);
        }
      }
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored; // ignore repeats
  }

  /// Pitches an assist key should sound now: the expected notes for [rightHand],
  /// or — for a [nearMiss] — a single random pitch near one of them that never
  /// equals an expected note. Empty when nothing is expected for that hand.
  Set<int> _assistPitches({required bool rightHand, required bool nearMiss}) {
    final data = ref.read(playerProvider);
    final expected = data.expectedNotesForHand(
      data.elapsedMs,
      rightHand: rightHand,
    );
    if (expected.isEmpty) return const {};
    if (!nearMiss) return expected;
    final bounds = data.keyboardBounds;
    return {
      nearMissPitch(
        expected.first,
        lowBound: bounds.low,
        highBound: bounds.high,
        avoid: expected,
        nextRandom: _rng.nextInt,
      ),
    };
  }

  // --- On-screen keyboard (mouse / touch) -------------------------------
  // Routes pointer presses through the same note-on/off path as MIDI and the
  // computer keyboard, so on-screen play drives feedback and the Wait Mode gate
  // identically. Works during playback and when stopped.

  void _onKeyboardPointerDown(PointerDownEvent event, PianoLayout layout) {
    final pitch = layout.pitchAt(event.localPosition, _keyboardHeight);
    if (pitch == null) return;
    _keyboardPointers[event.pointer] = pitch;
    ref.read(playerProvider.notifier).noteOn(pitch);
  }

  void _onKeyboardPointerUp(PointerEvent event) {
    final pitch = _keyboardPointers.remove(event.pointer);
    if (pitch == null) return;
    ref.read(playerProvider.notifier).noteOff(pitch);
  }

  @override
  void dispose() {
    _ticker.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Focus(
      focusNode: _focusNode,
      autofocus: true,
      onKeyEvent: _onKey,
      child: Scaffold(
        // Settings live in an end drawer (slides in from the right). Opening it
        // pauses the session; closing restores the prior play/pause state.
        endDrawer: _SettingsDrawer(key: _settingsDrawerKey),
        onEndDrawerChanged: _onEndDrawerChanged,
        body: SafeArea(
          child: Column(
            children: [
              const _TopBar(),
              Expanded(
                child: Consumer(
                  builder: (context, ref, child) {
                    final data = ref.watch(playerProvider);
                    return LayoutBuilder(
                      builder: (context, constraints) {
                        final bounds = data.keyboardBounds;
                        final layout = PianoLayout(
                          width: constraints.maxWidth,
                          lowPitch: bounds.low,
                          highPitch: bounds.high,
                        );
                        return Column(
                          children: [
                            // Clip the render area so a painter (e.g. high notes /
                            // beams in Staff mode) never draws over the top bar or
                            // the keyboard below.
                            Expanded(
                              child: ClipRect(
                                child: _buildRenderArea(layout, data),
                              ),
                            ),
                            SizedBox(
                              height: _keyboardHeight,
                              child: Listener(
                                key: const Key('onscreen-keyboard'),
                                onPointerDown: (e) =>
                                    _onKeyboardPointerDown(e, layout),
                                onPointerUp: _onKeyboardPointerUp,
                                onPointerCancel: _onKeyboardPointerUp,
                                child: CustomPaint(
                                  size: Size(
                                    constraints.maxWidth,
                                    _keyboardHeight,
                                  ),
                                  painter: PianoKeyboardPainter(
                                    layout: layout,
                                    activeNotes: data.activeNotes,
                                    requiredNotes: data.expectedKeys,
                                    leftHandNotes: data.expectedKeysForHand(
                                      rightHand: false,
                                    ),
                                  ),
                                ),
                              ),
                            ),
                          ],
                        );
                      },
                    );
                  },
                ),
              ),
              _TransportBar(),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildRenderArea(PianoLayout layout, PlayerData data) {
    if (data.mode == RenderMode.partition) {
      return const _PartitionView();
    }
    if (data.mode == RenderMode.synthesia) {
      return Stack(
        children: [
          Positioned.fill(
            child: CustomPaint(
              painter: SynthesiaPainter(
                layout: layout,
                notes: data.visibleNotes,
                elapsedMs: data.elapsedMs,
                activeNotes: data.activeNotes,
              ),
            ),
          ),
          if (data.blocked) const _WaitOverlay(),
        ],
      );
    }
    // Standard staff mode (synchronized, horizontal scrolling).
    return Container(
      color: CymbraColors.surfaceContainerLow,
      child: CustomPaint(
        painter: StaffPainter(
          notes: data.visibleNotes,
          elapsedMs: data.elapsedMs,
          activeNotes: data.activeNotes,
          bpm: data.bpm,
          songEndMs: data.songEndMs,
          keyFifths: data.keyFifths,
          beats: data.beats,
          beatType: data.beatType,
          measureStartMs: data.measureStartMs,
        ),
        size: Size.infinite,
      ),
    );
  }
}

/// Top bar: title, indicators and mode toggle.
///
/// A `const` [ConsumerWidget] that watches **only** the title and tempo — not
/// the playhead — so the player's per-frame rebuilds (while playing) do not
/// rebuild the top bar. Each interactive control is its own `const` consumer
/// watching its own slice, which keeps the open settings menu stable on touch
/// devices (an ever-rebuilding [MenuAnchor] flickered on iPad).
class _TopBar extends ConsumerWidget {
  const _TopBar();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Watch only the title here; the tempo/metronome chip watches its own slices
    // so the per-beat pulse doesn't rebuild the whole top bar.
    final title = ref.watch(playerProvider.select((d) => d.title));
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      decoration: const BoxDecoration(
        color: CymbraColors.surfaceContainerLowest,
        border: Border(
          bottom: BorderSide(color: CymbraColors.outlineVariant, width: 1),
        ),
      ),
      child: Row(
        children: [
          // Wired only when reached from the library (a route to pop back to).
          if (Navigator.of(context).canPop())
            IconButton(
              icon: const Icon(Icons.arrow_back, color: CymbraColors.onSurface),
              tooltip: 'Back to library',
              onPressed: () => Navigator.of(context).maybePop(),
            )
          else
            const Icon(Icons.arrow_back, color: CymbraColors.onSurface),
          const SizedBox(width: 16),
          // Expanded (instead of a fixed Column + Spacer) so the title absorbs
          // the free space and shrinks gracefully on narrow windows; the texts
          // ellipsize rather than overflowing the top bar.
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Cymbra Music',
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    color: CymbraColors.primary,
                    fontSize: 18,
                    fontWeight: FontWeight.w800,
                  ),
                ),
                Text(
                  'Now Playing: ${title ?? '—'}',
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    color: CymbraColors.onSurfaceVariant,
                    fontSize: 12,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(width: 12),
          // MIDI connection status (read-only at a glance); the device itself is
          // chosen from the settings menu.
          const _MidiStatusIndicator(),
          const SizedBox(width: 8),
          // Tap to toggle the metronome; pulses on each beat. Mode-independent
          // (lives in the shared top bar), so it works in Synthesia/Staff/Partition.
          const _TempoChip(),
          const SizedBox(width: 8),
          // Consolidated music settings (MIDI device, keyboard size, hand). Lives
          // in the mode-independent top bar, so it is reachable in Synthesia,
          // Staff and Partition alike.
          const _SettingsMenu(),
          const SizedBox(width: 8),
          // Rendering mode toggle.
          const _ModeToggle(),
        ],
      ),
    );
  }
}

/// Gear button that opens the settings **end drawer** (slides in from the
/// right). A drawer is a modal route with a scrim, so — unlike the dropdown
/// menus that flickered on iPad — it cannot dismiss itself; opening it also
/// pauses the session (see [_PlayerScreenState._onEndDrawerChanged]).
class _SettingsMenu extends StatelessWidget {
  const _SettingsMenu();

  @override
  Widget build(BuildContext context) {
    return IconButton(
      icon: const Icon(Icons.tune, color: CymbraColors.onSurface),
      tooltip: 'Settings',
      onPressed: () => Scaffold.of(context).openEndDrawer(),
    );
  }
}

/// A setting category shown in the drawer's top-level list: its title, icon, and
/// a short label of the value currently in effect.
typedef _Category = ({String title, IconData icon, String current});

/// The settings drawer: a master-detail panel. The first screen lists the
/// setting **categories** (MIDI device, Keyboard size, Hand); tapping one shows
/// just that category's values, so the options are never all on screen at once.
/// Reads/updates the live [PlayerData] selection.
class _SettingsDrawer extends ConsumerStatefulWidget {
  const _SettingsDrawer({super.key});

  @override
  ConsumerState<_SettingsDrawer> createState() => _SettingsDrawerState();
}

class _SettingsDrawerState extends ConsumerState<_SettingsDrawer> {
  static const Map<Hand, String> _handLabels = {
    Hand.left: 'Left',
    Hand.right: 'Right',
    Hand.both: 'Both',
  };

  /// The category whose values are shown; null shows the category list.
  String? _category;

  /// Returns the drawer to its top-level category list (called when it opens).
  void resetToRoot() {
    if (mounted && _category != null) setState(() => _category = null);
  }

  String _rangeLabel(KeyboardRangeMode m) =>
      m == KeyboardRangeMode.auto ? 'Auto (fit piece)' : '${m.label} keys';

  /// A radio-style value row with a leading "selected" check.
  Widget _option({
    required bool selected,
    required String label,
    required VoidCallback? onTap,
  }) => ListTile(
    leading: Icon(
      selected ? Icons.check_circle : Icons.radio_button_unchecked,
      size: 20,
      color: selected ? CymbraColors.tertiary : CymbraColors.onSurfaceVariant,
    ),
    title: Text(label, style: const TextStyle(color: CymbraColors.onSurface)),
    onTap: onTap,
  );

  /// The value rows for [category], built from the current selection.
  List<Widget> _valuesFor(
    String category, {
    required List<String> midiPorts,
    required String? connectedDevice,
    required KeyboardRangeMode keyboardRange,
    required Hand selectedHands,
    required Player notifier,
  }) {
    switch (category) {
      case 'MIDI device':
        return [
          _option(
            selected: connectedDevice == null,
            label: 'Auto (first real device)',
            onTap: () => notifier.selectMidiPort(null),
          ),
          for (final p in midiPorts)
            _option(
              selected: p == connectedDevice,
              label: p,
              onTap: () => notifier.selectMidiPort(p),
            ),
          if (midiPorts.isEmpty)
            _option(selected: false, label: 'No device detected', onTap: null),
        ];
      case 'Keyboard size':
        return [
          for (final m in KeyboardRangeMode.values)
            _option(
              selected: m == keyboardRange,
              label: _rangeLabel(m),
              onTap: () => notifier.setKeyboardRange(m),
            ),
        ];
      case 'Hand':
        return [
          for (final h in Hand.values)
            _option(
              selected: h == selectedHands,
              label: _handLabels[h]!,
              onTap: () => notifier.setSelectedHands(h),
            ),
        ];
      default:
        return const [];
    }
  }

  @override
  Widget build(BuildContext context) {
    final notifier = ref.read(playerProvider.notifier);
    final (
      midiPorts,
      connectedDevice,
      keyboardRange,
      selectedHands,
      twoStaves,
    ) = ref.watch(
      playerProvider.select(
        (d) => (
          d.midiPorts,
          d.connectedDevice,
          d.keyboardRange,
          d.selectedHands,
          d.hasMultipleStaves,
        ),
      ),
    );

    // Top-level categories (with the value currently in effect as a subtitle).
    final categories = <_Category>[
      (
        title: 'MIDI device',
        icon: Icons.piano,
        current: connectedDevice ?? 'Auto',
      ),
      (
        title: 'Keyboard size',
        icon: Icons.straighten,
        current: keyboardRange == KeyboardRangeMode.auto
            ? 'Auto'
            : keyboardRange.label,
      ),
      if (twoStaves)
        (
          title: 'Hand',
          icon: Icons.front_hand,
          current: _handLabels[selectedHands]!,
        ),
    ];

    final Widget body;
    if (_category == null) {
      // Master view: the list of categories.
      body = ListView(
        padding: EdgeInsets.zero,
        children: [
          const _DrawerHeader(title: 'Settings'),
          for (final c in categories)
            ListTile(
              leading: Icon(c.icon, color: CymbraColors.onSurfaceVariant),
              title: Text(
                c.title,
                style: const TextStyle(color: CymbraColors.onSurface),
              ),
              subtitle: Text(
                c.current,
                style: const TextStyle(color: CymbraColors.onSurfaceVariant),
              ),
              trailing: const Icon(
                Icons.chevron_right,
                color: CymbraColors.onSurfaceVariant,
              ),
              onTap: () => setState(() => _category = c.title),
            ),
        ],
      );
    } else {
      // Detail view: just the selected category's values, with a back affordance.
      body = ListView(
        padding: EdgeInsets.zero,
        children: [
          _DrawerHeader(
            title: _category!,
            onBack: () => setState(() => _category = null),
          ),
          ..._valuesFor(
            _category!,
            midiPorts: midiPorts,
            connectedDevice: connectedDevice,
            keyboardRange: keyboardRange,
            selectedHands: selectedHands,
            notifier: notifier,
          ),
        ],
      );
    }

    return Drawer(
      backgroundColor: CymbraColors.surfaceContainerHigh,
      child: SafeArea(child: body),
    );
  }
}

/// Drawer header: a title, optionally preceded by a back button (in the detail
/// view), with a bottom divider.
class _DrawerHeader extends StatelessWidget {
  final String title;
  final VoidCallback? onBack;
  const _DrawerHeader({required this.title, this.onBack});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: EdgeInsets.fromLTRB(onBack != null ? 4 : 20, 14, 12, 12),
          child: Row(
            children: [
              if (onBack != null) ...[
                IconButton(
                  tooltip: 'Back',
                  icon: const Icon(
                    Icons.arrow_back,
                    color: CymbraColors.onSurfaceVariant,
                  ),
                  onPressed: onBack,
                ),
                const SizedBox(width: 4),
              ],
              Expanded(
                child: Text(
                  title,
                  style: const TextStyle(
                    color: CymbraColors.onSurface,
                    fontSize: 18,
                    fontWeight: FontWeight.w800,
                  ),
                ),
              ),
            ],
          ),
        ),
        const Divider(height: 1, color: CymbraColors.outlineVariant),
      ],
    );
  }
}

/// Switch between the rendering modes (Synthesia / Staff / Partition).
class _ModeToggle extends ConsumerWidget {
  const _ModeToggle();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final mode = ref.watch(playerProvider.select((d) => d.mode));
    final notifier = ref.read(playerProvider.notifier);
    return SegmentedButton<RenderMode>(
      style: ButtonStyle(
        visualDensity: VisualDensity.compact,
        backgroundColor: WidgetStateProperty.resolveWith(
          (s) => s.contains(WidgetState.selected)
              ? CymbraColors.primaryContainer
              : CymbraColors.surfaceContainerHigh,
        ),
      ),
      // Labels only (no per-segment icons) to keep the top bar within narrow
      // tablet widths now that there are three modes.
      segments: const [
        ButtonSegment(value: RenderMode.synthesia, label: Text('Synthesia')),
        ButtonSegment(value: RenderMode.staff, label: Text('Staff')),
        ButtonSegment(value: RenderMode.partition, label: Text('Partition')),
      ],
      selected: {mode},
      onSelectionChanged: (s) => notifier.setMode(s.first),
      showSelectedIcon: false,
    );
  }
}

/// The header **Tempo** chip, doubling as the metronome toggle.
///
/// Tapping it flips [Player.toggleMetronome]; when the metronome is enabled the
/// chip takes an active (primary-tinted) style and **pulses once per beat**,
/// harder on the accented downbeat. The pulse is the visual half of the beat (the
/// audible click is the other) and, living in the mode-independent top bar, it is
/// visible the same way in Synthesia, Staff and Partition. Watches only its own
/// slices so the per-beat pulse never rebuilds the rest of the top bar.
class _TempoChip extends ConsumerStatefulWidget {
  const _TempoChip();

  @override
  ConsumerState<_TempoChip> createState() => _TempoChipState();
}

class _TempoChipState extends ConsumerState<_TempoChip>
    with SingleTickerProviderStateMixin {
  late final AnimationController _pulse = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 200),
  );
  bool _accent = false;

  @override
  void dispose() {
    _pulse.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final (bpm, enabled, beatCount, lastAccent) = ref.watch(
      playerProvider.select(
        (d) => (d.bpm, d.metronomeEnabled, d.beatCount, d.lastBeatAccent),
      ),
    );
    // Fire one pulse per beat: restart the decay animation whenever the beat
    // counter ticks. Done as a listen (not in build's body) so it reacts to the
    // change rather than the rebuild.
    ref.listen(playerProvider.select((d) => d.beatCount), (_, _) {
      _accent = lastAccent;
      _pulse.forward(from: 0);
    });

    return Semantics(
      button: true,
      toggled: enabled,
      label: 'Metronome',
      child: InkWell(
        onTap: () => ref.read(playerProvider.notifier).toggleMetronome(),
        borderRadius: BorderRadius.circular(8),
        child: AnimatedBuilder(
          animation: _pulse,
          builder: (context, child) {
            // Pulse intensity decays 1 → 0 over the animation; the accent pulses
            // brighter. Zero when the metronome is off.
            final intensity = enabled ? (1 - _pulse.value) : 0.0;
            final glow = intensity * (_accent ? 0.9 : 0.45);
            final baseColor = enabled
                ? Color.alphaBlend(
                    CymbraColors.primary.withValues(alpha: 0.18),
                    CymbraColors.surfaceContainerHigh,
                  )
                : CymbraColors.surfaceContainerHigh;
            return Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(
                color: Color.alphaBlend(
                  CymbraColors.primary.withValues(alpha: glow),
                  baseColor,
                ),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                  color: enabled
                      ? CymbraColors.primary
                      : CymbraColors.surfaceContainerHigh,
                  width: 1,
                ),
              ),
              child: child,
            );
          },
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                Icons.speed,
                size: 16,
                color: enabled
                    ? CymbraColors.primary
                    : CymbraColors.onSurfaceVariant,
              ),
              const SizedBox(width: 6),
              Text(
                'Tempo: $bpm',
                style: const TextStyle(
                  color: CymbraColors.onSurface,
                  fontSize: 13,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// MIDI connection status (read-only): a coloured dot + icon and a short state
/// label — green when connected, amber while a device is detected but not yet
/// connected, gray when none. The connected device's *name* is not shown here;
/// the device is listed and chosen from the settings menu (see [_SettingsMenu]).
class _MidiStatusIndicator extends ConsumerWidget {
  const _MidiStatusIndicator();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Watch only the connection state, not the playhead, so this chip is not
    // rebuilt on every player frame.
    final (connected, hasPorts) = ref.watch(
      playerProvider.select((d) => (d.midiConnected, d.midiPorts.isNotEmpty)),
    );

    final Color color;
    final String label;
    final IconData icon;

    if (connected) {
      color = CymbraColors.tertiary;
      icon = Icons.usb;
      label = 'Connected';
    } else if (hasPorts) {
      color = CymbraColors.secondary;
      icon = Icons.usb;
      label = 'Connecting…';
    } else {
      color = CymbraColors.outline;
      icon = Icons.usb_off;
      label = 'No MIDI device';
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withValues(alpha: 0.6)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Status dot.
          Container(
            width: 9,
            height: 9,
            decoration: BoxDecoration(
              color: color,
              shape: BoxShape.circle,
              boxShadow: [
                BoxShadow(color: color.withValues(alpha: 0.7), blurRadius: 6),
              ],
            ),
          ),
          const SizedBox(width: 8),
          Icon(icon, size: 16, color: color),
          const SizedBox(width: 6),
          ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 200),
            child: Text(
              label,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(
                color: CymbraColors.onSurface,
                fontSize: 13,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// Floating transport bar: restart, play/pause, speed, loop, Wait Mode.
class _TransportBar extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final data = ref.watch(playerProvider);
    final notifier = ref.read(playerProvider.notifier);

    return Container(
      margin: const EdgeInsets.all(16),
      padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 10),
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh.withValues(alpha: 0.9),
        borderRadius: BorderRadius.circular(999),
        border: Border.all(color: CymbraColors.outlineVariant),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          IconButton(
            onPressed: notifier.restart,
            icon: const Icon(
              Icons.skip_previous,
              color: CymbraColors.onSurface,
            ),
          ),
          const SizedBox(width: 8),
          // Play / pause.
          GestureDetector(
            onTap: notifier.togglePlay,
            child: CircleAvatar(
              radius: 26,
              backgroundColor: CymbraColors.primaryContainer,
              child: Icon(
                data.isPlaying ? Icons.pause : Icons.play_arrow,
                color: Colors.white,
                size: 28,
              ),
            ),
          ),
          const SizedBox(width: 16),
          // Speed.
          IconButton(
            onPressed: () => notifier.setSpeed(data.speed - 0.25),
            icon: const Icon(
              Icons.remove,
              color: CymbraColors.onSurfaceVariant,
            ),
          ),
          Text(
            '${(data.speed * 100).round()}% SPD',
            style: const TextStyle(
              color: CymbraColors.onSurface,
              fontWeight: FontWeight.w600,
            ),
          ),
          IconButton(
            onPressed: () => notifier.setSpeed(data.speed + 0.25),
            icon: const Icon(Icons.add, color: CymbraColors.onSurfaceVariant),
          ),
          const SizedBox(width: 8),
          // Wait Mode.
          TextButton.icon(
            onPressed: notifier.toggleWaitMode,
            icon: Icon(
              data.waitMode ? Icons.hourglass_top : Icons.hourglass_disabled,
              color: data.waitMode
                  ? CymbraColors.secondary
                  : CymbraColors.onSurfaceVariant,
            ),
            label: Text(
              'Wait',
              style: TextStyle(
                color: data.waitMode
                    ? CymbraColors.secondary
                    : CymbraColors.onSurfaceVariant,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// Overlay shown when Wait Mode freezes the cascade.
class _WaitOverlay extends StatelessWidget {
  const _WaitOverlay();

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
        decoration: BoxDecoration(
          color: CymbraColors.surfaceContainer.withValues(alpha: 0.85),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(color: CymbraColors.secondary),
        ),
        child: const Text(
          '⏸  Play the expected note to continue',
          style: TextStyle(color: CymbraColors.secondary, fontSize: 16),
        ),
      ),
    );
  }
}

/// Engraved-notation (Partition) render mode: draws the laid-out MusicXML of the
/// loaded score and re-lays it out as the available width changes. Shows a
/// loading/empty state when no score notation is available (e.g. the demo).
class _PartitionView extends ConsumerStatefulWidget {
  const _PartitionView();

  @override
  ConsumerState<_PartitionView> createState() => _PartitionViewState();
}

class _PartitionViewState extends ConsumerState<_PartitionView> {
  final ScrollController _scroll = ScrollController();

  /// The last scroll target we animated to, so we only scroll when the cursor
  /// moves to a new line (not every frame, which would restart the animation).
  double? _lastScrollTarget;

  @override
  void dispose() {
    _scroll.dispose();
    super.dispose();
  }

  /// Index of the system containing [measureIndex], or null if not found.
  int? _systemOf(int measureIndex, List<System> systems) {
    for (var i = 0; i < systems.length; i++) {
      if (systems[i].measures.contains(measureIndex)) return i;
    }
    return null;
  }

  /// Auto-scroll **per staff line (system)**, not per measure: the vertical
  /// target depends only on which system the cursor is in, so the view advances
  /// once when the playhead moves to a new line and stays put while it crosses
  /// measures within the same line (no back-and-forth jitter). The current line
  /// is centred in the viewport; look-ahead is provided by the next-line overlay
  /// (see [_buildNextLineOverlay]), not by scrolling ahead. Only while playing,
  /// so manual scrolling is undisturbed when paused.
  void _followCursor(
    PlayerData data,
    List<System> systems,
    PartitionPainter painter,
  ) {
    if (!data.isPlaying) return;
    final cursor = data.measureAt(data.elapsedMs);
    if (cursor == null) return;
    final sysIndex = _systemOf(cursor.index, systems);
    if (sysIndex == null) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!_scroll.hasClients) return;
      final max = _scroll.position.maxScrollExtent;
      if (max <= 0) return; // everything fits — no scrolling
      final viewport = _scroll.position.viewportDimension;
      final target =
          (painter.systemTopY(sysIndex) +
                  painter.systemStride / 2 -
                  viewport / 2)
              .clamp(0.0, max);
      // Only scroll when the line changes — re-issuing every frame would restart
      // (and stall) the animation.
      if (_lastScrollTarget != null &&
          (target - _lastScrollTarget!).abs() < 4) {
        return;
      }
      _lastScrollTarget = target;
      _scroll.animateTo(
        target,
        duration: const Duration(milliseconds: 300),
        curve: Curves.easeInOut,
      );
    });
  }

  /// A small "next up" overlay showing the first two measures of the **next**
  /// line, pinned top-left. It appears only once the playhead is past the middle
  /// of the current line (so the top-left, already-played area is free to cover)
  /// and only when there is a next line. Returns null otherwise.
  Widget? _buildNextLineOverlay(
    PlayerData data,
    NotationData notation,
    double width,
    PartitionPainter mainPainter,
  ) {
    final cursor = data.measureAt(data.elapsedMs);
    if (cursor == null) return null;
    final systems = notation.systems;
    final sysIndex = _systemOf(cursor.index, systems);
    if (sysIndex == null || sysIndex + 1 >= systems.length) return null;

    final current = systems[sysIndex];
    final pos = current.measures.indexOf(cursor.index);
    if (pos < 0) return null;
    final lineProgress = (pos + cursor.fraction) / current.measures.length;
    if (lineProgress < 0.5) return null; // only near the end of the line

    // Don't cover the score when the next line is already visible on screen
    // (e.g. a tall viewport shows it below the current line) — the overlay is
    // only useful when the next line is still below the fold.
    if (_scroll.hasClients) {
      final vpTop = _scroll.offset;
      final vpBottom = vpTop + _scroll.position.viewportDimension;
      final nextTop = mainPainter.systemTopY(sysIndex + 1);
      final nextBottom = nextTop + mainPainter.systemStride;
      final visible =
          (nextBottom < vpBottom ? nextBottom : vpBottom) -
          (nextTop > vpTop ? nextTop : vpTop);
      if (visible >= mainPainter.systemStride * 0.6) return null;
    }

    // Engrave the FULL next system at the same width as the main view (so the
    // notes are exactly the same size — no down-scaling) and clip the overlay to
    // its first measure (a two-measure peek was too wide). The clip width follows
    // the painter's justification: an approximate header plus that measure's
    // share of the system width.
    final next = systems[sysIndex + 1];
    final measures = notation.document!.measures;
    var total = 0.0;
    for (final m in next.measures) {
      total += measures[m].minWidth;
    }
    final firstMin = measures[next.measures.first].minWidth;
    const headerApprox = 96.0; // clef + key + time, roughly
    final usable = (width - headerApprox).clamp(0.0, width);
    final boxWidth =
        headerApprox + (total > 0 ? firstMin / total : 1.0) * usable;
    return _NextLineOverlay(
      painter: PartitionPainter(
        document: notation.document!,
        systems: [next],
        selectedHands: data.selectedHands,
      ),
      fullWidth: width,
      boxWidth: boxWidth,
    );
  }

  @override
  Widget build(BuildContext context) {
    final notation = ref.watch(notationProvider);
    final data = ref.watch(playerProvider);

    if (notation.error != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Text(
            'Could not load this score:\n${notation.error}',
            textAlign: TextAlign.center,
            style: const TextStyle(color: CymbraColors.error),
          ),
        ),
      );
    }
    if (!notation.hasDocument) {
      return const Center(
        child: Text(
          'No partition loaded — pick a score from the library.',
          style: TextStyle(color: CymbraColors.onSurfaceVariant),
        ),
      );
    }

    return Container(
      color: CymbraColors.surfaceContainerLow,
      child: LayoutBuilder(
        builder: (context, constraints) {
          final width = constraints.maxWidth;
          WidgetsBinding.instance.addPostFrameCallback((_) {
            ref.read(notationProvider.notifier).setAvailableWidth(width);
          });
          final painter = PartitionPainter(
            document: notation.document!,
            systems: notation.systems,
            elapsedMs: data.elapsedMs,
            measureStartMs: data.measureStartMs,
            songEndMs: data.songEndMs,
            activeNotes: data.activeNotes,
            selectedHands: data.selectedHands,
          );
          _followCursor(data, notation.systems, painter);
          final overlay = _buildNextLineOverlay(data, notation, width, painter);
          return Stack(
            children: [
              SingleChildScrollView(
                controller: _scroll,
                child: CustomPaint(
                  key: const Key('partition-canvas'),
                  painter: painter,
                  size: Size(width, painter.heightFor(width)),
                ),
              ),
              if (overlay != null) Positioned(left: 8, top: 8, child: overlay),
            ],
          );
        },
      ),
    );
  }
}

/// "Next up" peek: the first measures of the upcoming line, scaled down into a
/// small framed box (pinned top-left over the already-played start of the line).
class _NextLineOverlay extends StatelessWidget {
  final PartitionPainter painter;

  /// Width the system is engraved at — the same as the main view, so the notes
  /// are rendered at identical size (no scaling).
  final double fullWidth;

  /// Visible width of the peek (clips to roughly the first two measures).
  final double boxWidth;

  const _NextLineOverlay({
    required this.painter,
    required this.fullWidth,
    required this.boxWidth,
  });

  @override
  Widget build(BuildContext context) {
    final height = painter.heightFor(fullWidth);
    return Container(
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh.withValues(alpha: 0.94),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: CymbraColors.outlineVariant),
      ),
      padding: const EdgeInsets.fromLTRB(8, 4, 8, 6),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          const Text(
            'NEXT',
            style: TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 9,
              fontWeight: FontWeight.w700,
              letterSpacing: 1.5,
            ),
          ),
          const SizedBox(height: 2),
          // The system is painted at [fullWidth] (full size) but only [boxWidth]
          // is shown; OverflowBox lets the wider canvas extend under the clip.
          SizedBox(
            width: boxWidth,
            height: height,
            child: ClipRect(
              child: OverflowBox(
                alignment: Alignment.topLeft,
                minWidth: 0,
                maxWidth: fullWidth,
                minHeight: 0,
                maxHeight: height,
                child: SizedBox(
                  width: fullWidth,
                  height: height,
                  child: CustomPaint(
                    painter: painter,
                    size: Size(fullWidth, height),
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
