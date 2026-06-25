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
import 'package:flutter/services.dart';
import 'package:flutter/scheduler.dart';
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
    final data = ref.watch(playerProvider);
    final notifier = ref.read(playerProvider.notifier);
    return Focus(
      focusNode: _focusNode,
      autofocus: true,
      onKeyEvent: _onKey,
      child: Scaffold(
        body: SafeArea(
          child: Column(
            children: [
              _TopBar(data: data, notifier: notifier),
              Expanded(
                child: LayoutBuilder(
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
                              size: Size(constraints.maxWidth, _keyboardHeight),
                              painter: PianoKeyboardPainter(
                                layout: layout,
                                activeNotes: data.activeNotes,
                                requiredNotes: data.expectedKeys,
                              ),
                            ),
                          ),
                        ),
                      ],
                    );
                  },
                ),
              ),
              _TransportBar(data: data, notifier: notifier),
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
                notes: data.notes,
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
          notes: data.notes,
          elapsedMs: data.elapsedMs,
          activeNotes: data.activeNotes,
          bpm: data.bpm,
          songEndMs: data.songEndMs,
          keyFifths: data.keyFifths,
          beats: data.beats,
          beatType: data.beatType,
        ),
        size: Size.infinite,
      ),
    );
  }
}

/// Top bar: title, indicators and mode toggle.
class _TopBar extends StatelessWidget {
  final PlayerData data;
  final Player notifier;
  const _TopBar({required this.data, required this.notifier});

  @override
  Widget build(BuildContext context) {
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
                  'Now Playing: ${data.title ?? '—'}',
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
          _MidiStatusIndicator(data: data, notifier: notifier),
          const SizedBox(width: 8),
          _Chip(icon: Icons.speed, label: 'Tempo: ${data.bpm}'),
          const SizedBox(width: 8),
          // On-screen keyboard size chooser.
          _RangeChooser(data: data, notifier: notifier),
          const SizedBox(width: 8),
          // Rendering mode toggle.
          _ModeToggle(data: data, notifier: notifier),
        ],
      ),
    );
  }
}

/// Chooses the on-screen keyboard range: auto-fit to the piece, or a fixed
/// key-count preset (25/37/49/61/76/88).
class _RangeChooser extends StatelessWidget {
  final PlayerData data;
  final Player notifier;
  const _RangeChooser({required this.data, required this.notifier});

  @override
  Widget build(BuildContext context) {
    final current = data.keyboardRange;
    return PopupMenuButton<KeyboardRangeMode>(
      tooltip: 'Keyboard size',
      color: CymbraColors.surfaceContainerHigh,
      onSelected: notifier.setKeyboardRange,
      itemBuilder: (_) => [
        for (final m in KeyboardRangeMode.values)
          PopupMenuItem<KeyboardRangeMode>(
            value: m,
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  m == current
                      ? Icons.check_circle
                      : Icons.radio_button_unchecked,
                  size: 16,
                  color: m == current
                      ? CymbraColors.tertiary
                      : CymbraColors.onSurfaceVariant,
                ),
                const SizedBox(width: 8),
                Text(
                  m == KeyboardRangeMode.auto
                      ? 'Auto (fit piece)'
                      : '${m.label} keys',
                ),
              ],
            ),
          ),
      ],
      child: _Chip(
        icon: Icons.piano,
        label: current == KeyboardRangeMode.auto ? 'Auto' : current.label,
      ),
    );
  }
}

/// Switch between the two rendering modes (Synthesia / Staff).
class _ModeToggle extends StatelessWidget {
  final PlayerData data;
  final Player notifier;
  const _ModeToggle({required this.data, required this.notifier});

  @override
  Widget build(BuildContext context) {
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
      selected: {data.mode},
      onSelectionChanged: (s) => notifier.setMode(s.first),
      showSelectedIcon: false,
    );
  }
}

class _Chip extends StatelessWidget {
  final IconData icon;
  final String label;
  const _Chip({required this.icon, required this.label});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Icon(icon, size: 16, color: CymbraColors.onSurfaceVariant),
          const SizedBox(width: 6),
          Text(
            label,
            style: const TextStyle(color: CymbraColors.onSurface, fontSize: 13),
          ),
        ],
      ),
    );
  }
}

/// MIDI connection indicator: green dot + name of the connected device,
/// amber if detected but not yet connected, gray if none.
class _MidiStatusIndicator extends StatelessWidget {
  final PlayerData data;
  final Player notifier;
  const _MidiStatusIndicator({required this.data, required this.notifier});

  @override
  Widget build(BuildContext context) {
    final Color color;
    final String label;
    final IconData icon;

    if (data.midiConnected) {
      color = CymbraColors.tertiary;
      icon = Icons.usb;
      label = data.connectedDevice!;
    } else if (data.midiPorts.isNotEmpty) {
      color = CymbraColors.secondary;
      icon = Icons.usb;
      label = '${data.midiPorts.first} (connecting…)';
    } else {
      color = CymbraColors.outline;
      icon = Icons.usb_off;
      label = 'No MIDI device';
    }

    const autoValue = '__auto__';
    return PopupMenuButton<String>(
      tooltip: 'Choose MIDI device',
      color: CymbraColors.surfaceContainerHigh,
      onSelected: (v) => notifier.selectMidiPort(v == autoValue ? null : v),
      itemBuilder: (_) => [
        PopupMenuItem<String>(
          value: autoValue,
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: const [
              Icon(
                Icons.autorenew,
                size: 16,
                color: CymbraColors.onSurfaceVariant,
              ),
              SizedBox(width: 8),
              Text('Auto (first real device)'),
            ],
          ),
        ),
        if (data.midiPorts.isNotEmpty) const PopupMenuDivider(),
        ...data.midiPorts.map(
          (p) => PopupMenuItem<String>(
            value: p,
            child: Row(
              children: [
                Icon(
                  p == data.connectedDevice
                      ? Icons.check_circle
                      : Icons.radio_button_unchecked,
                  size: 16,
                  color: p == data.connectedDevice
                      ? CymbraColors.tertiary
                      : CymbraColors.onSurfaceVariant,
                ),
                const SizedBox(width: 8),
                Flexible(child: Text(p)),
              ],
            ),
          ),
        ),
        if (data.midiPorts.isEmpty)
          const PopupMenuItem<String>(
            enabled: false,
            child: Text('No device detected'),
          ),
      ],
      child: Container(
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
            const SizedBox(width: 4),
            const Icon(
              Icons.arrow_drop_down,
              size: 18,
              color: CymbraColors.onSurfaceVariant,
            ),
          ],
        ),
      ),
    );
  }
}

/// Floating transport bar: restart, play/pause, speed, loop, Wait Mode.
class _TransportBar extends StatelessWidget {
  final PlayerData data;
  final Player notifier;
  const _TransportBar({required this.data, required this.notifier});

  @override
  Widget build(BuildContext context) {
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

    // Engrave the FULL next system at the same width as the main view (so the
    // notes are exactly the same size — no down-scaling) and clip the overlay to
    // the first ~2 measures. The clip width follows the painter's justification:
    // an approximate header plus the first measures' share of the system width.
    final next = systems[sysIndex + 1];
    final measures = notation.document!.measures;
    var total = 0.0;
    for (final m in next.measures) {
      total += measures[m].minWidth;
    }
    final take = next.measures.length < 2 ? next.measures.length : 2;
    var firstMin = 0.0;
    for (var i = 0; i < take; i++) {
      firstMin += measures[next.measures[i]].minWidth;
    }
    const headerApprox = 96.0; // clef + key + time, roughly
    final usable = (width - headerApprox).clamp(0.0, width);
    final boxWidth =
        headerApprox + (total > 0 ? firstMin / total : 1.0) * usable;
    return _NextLineOverlay(
      painter: PartitionPainter(document: notation.document!, systems: [next]),
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
          );
          _followCursor(data, notation.systems, painter);
          final overlay = _buildNextLineOverlay(data, notation, width);
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
