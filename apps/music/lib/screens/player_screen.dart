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
import 'package:flutter/services.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../painters/partition_painter.dart';
import '../painters/piano_keyboard_painter.dart';
import '../painters/piano_layout.dart';
import '../painters/staff_painter.dart';
import '../painters/synthesia_painter.dart';
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

  static const double _keyboardHeight = 150;

  /// Computer keyboard → MIDI pitch mapping (piano-style row, QWERTY).
  static final Map<LogicalKeyboardKey, int> _keyToPitch = {
    LogicalKeyboardKey.keyA: 60, // C4
    LogicalKeyboardKey.keyW: 61, // C#4
    LogicalKeyboardKey.keyS: 62, // D4
    LogicalKeyboardKey.keyE: 63, // D#4
    LogicalKeyboardKey.keyD: 64, // E4
    LogicalKeyboardKey.keyF: 65, // F4
    LogicalKeyboardKey.keyT: 66, // F#4
    LogicalKeyboardKey.keyG: 67, // G4
    LogicalKeyboardKey.keyY: 68, // G#4
    LogicalKeyboardKey.keyH: 69, // A4
    LogicalKeyboardKey.keyU: 70, // A#4
    LogicalKeyboardKey.keyJ: 71, // B4
    LogicalKeyboardKey.keyK: 72, // C5
    LogicalKeyboardKey.keyO: 73, // C#5
    LogicalKeyboardKey.keyL: 74, // D5
  };

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

  KeyEventResult _onKey(FocusNode node, KeyEvent event) {
    final pitch = _keyToPitch[event.logicalKey];
    if (pitch == null) return KeyEventResult.ignored;
    final notifier = ref.read(playerProvider.notifier);
    if (event is KeyDownEvent) {
      notifier.noteOn(pitch);
      return KeyEventResult.handled;
    } else if (event is KeyUpEvent) {
      notifier.noteOff(pitch);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored; // ignore repeats
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
                        Expanded(child: _buildRenderArea(layout, data)),
                        SizedBox(
                          height: _keyboardHeight,
                          child: CustomPaint(
                            size: Size(constraints.maxWidth, _keyboardHeight),
                            painter: PianoKeyboardPainter(
                              layout: layout,
                              activeNotes: data.activeNotes,
                              requiredNotes: data.requiredNotesAt(
                                data.elapsedMs,
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
class _PartitionView extends ConsumerWidget {
  const _PartitionView();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final notation = ref.watch(notationProvider);

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
          );
          return SingleChildScrollView(
            child: CustomPaint(
              painter: painter,
              size: Size(width, painter.heightFor(width)),
            ),
          );
        },
      ),
    );
  }
}
