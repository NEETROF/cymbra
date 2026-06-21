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

import '../painters/piano_keyboard_painter.dart';
import '../painters/piano_layout.dart';
import '../painters/staff_painter.dart';
import '../painters/synthesia_painter.dart';
import '../state/player_state.dart';
import '../theme/cymbra_theme.dart';

/// Main screen of the Cymbra player: top bar, rendering area
/// (Synthesia or Staff), keyboard, and transport bar.
class PlayerScreen extends StatefulWidget {
  const PlayerScreen({super.key});

  @override
  State<PlayerScreen> createState() => _PlayerScreenState();
}

class _PlayerScreenState extends State<PlayerScreen>
    with SingleTickerProviderStateMixin {
  final PlayerState _state = PlayerState();
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
    _state.init();
    _ticker = createTicker(_onTick)..start();
  }

  void _onTick(Duration elapsed) {
    final dt = (elapsed - _lastTick).inMicroseconds / 1000.0; // ms
    _lastTick = elapsed;
    if (dt > 0 && dt < 100) {
      _state.advance(dt * _state.speed);
    }
  }

  KeyEventResult _onKey(FocusNode node, KeyEvent event) {
    final pitch = _keyToPitch[event.logicalKey];
    if (pitch == null) return KeyEventResult.ignored;
    if (event is KeyDownEvent) {
      _state.noteOn(pitch);
      return KeyEventResult.handled;
    } else if (event is KeyUpEvent) {
      _state.noteOff(pitch);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored; // ignore repeats
  }

  @override
  void dispose() {
    _ticker.dispose();
    _focusNode.dispose();
    _state.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Focus(
      focusNode: _focusNode,
      autofocus: true,
      onKeyEvent: _onKey,
      child: Scaffold(
        body: SafeArea(
          child: ListenableBuilder(
            listenable: _state,
            builder: (context, _) {
              return Column(
                children: [
                  _TopBar(state: _state),
                  Expanded(
                    child: LayoutBuilder(
                      builder: (context, constraints) {
                        final layout = PianoLayout(width: constraints.maxWidth);
                        return Column(
                          children: [
                            Expanded(child: _buildRenderArea(layout)),
                            SizedBox(
                              height: _keyboardHeight,
                              child: CustomPaint(
                                size: Size(
                                  constraints.maxWidth,
                                  _keyboardHeight,
                                ),
                                painter: PianoKeyboardPainter(
                                  layout: layout,
                                  activeNotes: _state.activeNotes,
                                ),
                              ),
                            ),
                          ],
                        );
                      },
                    ),
                  ),
                  _TransportBar(state: _state),
                ],
              );
            },
          ),
        ),
      ),
    );
  }

  Widget _buildRenderArea(PianoLayout layout) {
    if (_state.mode == RenderMode.synthesia) {
      return Stack(
        children: [
          Positioned.fill(
            child: CustomPaint(
              painter: SynthesiaPainter(
                layout: layout,
                notes: _state.notes,
                elapsedMs: _state.elapsedMs,
                activeNotes: _state.activeNotes,
              ),
            ),
          ),
          if (_state.blocked) const _WaitOverlay(),
        ],
      );
    }
    // Standard staff mode (synchronized, horizontal scrolling).
    return Container(
      color: CymbraColors.surfaceContainerLow,
      child: CustomPaint(
        painter: StaffPainter(
          notes: _state.notes,
          elapsedMs: _state.elapsedMs,
          activeNotes: _state.activeNotes,
          bpm: _state.score?.bpm ?? 80,
          songEndMs: _state.songEndMs,
        ),
        size: Size.infinite,
      ),
    );
  }
}

/// Top bar: title, indicators and mode toggle.
class _TopBar extends StatelessWidget {
  final PlayerState state;
  const _TopBar({required this.state});

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
          const Icon(Icons.arrow_back, color: CymbraColors.onSurface),
          const SizedBox(width: 16),
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Cymbra Music',
                style: TextStyle(
                  color: CymbraColors.primary,
                  fontSize: 18,
                  fontWeight: FontWeight.w800,
                ),
              ),
              const Text(
                'Now Playing: Demo — C Major Scale',
                style: TextStyle(
                  color: CymbraColors.onSurfaceVariant,
                  fontSize: 12,
                ),
              ),
            ],
          ),
          const Spacer(),
          _MidiStatusIndicator(state: state),
          const SizedBox(width: 8),
          _Chip(icon: Icons.speed, label: 'Tempo: ${state.score?.bpm ?? '--'}'),
          const SizedBox(width: 8),
          // Rendering mode toggle.
          _ModeToggle(state: state),
        ],
      ),
    );
  }
}

/// Switch between the two rendering modes (Synthesia / Staff).
class _ModeToggle extends StatelessWidget {
  final PlayerState state;
  const _ModeToggle({required this.state});

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
      segments: const [
        ButtonSegment(
          value: RenderMode.synthesia,
          label: Text('Synthesia'),
          icon: Icon(Icons.waterfall_chart),
        ),
        ButtonSegment(
          value: RenderMode.staff,
          label: Text('Staff'),
          icon: Icon(Icons.music_note),
        ),
      ],
      selected: {state.mode},
      onSelectionChanged: (s) => state.setMode(s.first),
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
  final PlayerState state;
  const _MidiStatusIndicator({required this.state});

  @override
  Widget build(BuildContext context) {
    final Color color;
    final String label;
    final IconData icon;

    if (state.midiConnected) {
      color = CymbraColors.tertiary;
      icon = Icons.usb;
      label = state.connectedDevice!;
    } else if (state.midiPorts.isNotEmpty) {
      color = CymbraColors.secondary;
      icon = Icons.usb;
      label = '${state.midiPorts.first} (connecting…)';
    } else {
      color = CymbraColors.outline;
      icon = Icons.usb_off;
      label = 'No MIDI device';
    }

    const autoValue = '__auto__';
    return PopupMenuButton<String>(
      tooltip: 'Choose MIDI device',
      color: CymbraColors.surfaceContainerHigh,
      onSelected: (v) => state.selectMidiPort(v == autoValue ? null : v),
      itemBuilder: (_) => [
        PopupMenuItem<String>(
          value: autoValue,
          child: Row(
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
        if (state.midiPorts.isNotEmpty) const PopupMenuDivider(),
        ...state.midiPorts.map(
          (p) => PopupMenuItem<String>(
            value: p,
            child: Row(
              children: [
                Icon(
                  p == state.connectedDevice
                      ? Icons.check_circle
                      : Icons.radio_button_unchecked,
                  size: 16,
                  color: p == state.connectedDevice
                      ? CymbraColors.tertiary
                      : CymbraColors.onSurfaceVariant,
                ),
                const SizedBox(width: 8),
                Flexible(child: Text(p)),
              ],
            ),
          ),
        ),
        if (state.midiPorts.isEmpty)
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
  final PlayerState state;
  const _TransportBar({required this.state});

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
            onPressed: state.restart,
            icon: const Icon(
              Icons.skip_previous,
              color: CymbraColors.onSurface,
            ),
          ),
          const SizedBox(width: 8),
          // Play / pause.
          GestureDetector(
            onTap: state.togglePlay,
            child: CircleAvatar(
              radius: 26,
              backgroundColor: CymbraColors.primaryContainer,
              child: Icon(
                state.isPlaying ? Icons.pause : Icons.play_arrow,
                color: Colors.white,
                size: 28,
              ),
            ),
          ),
          const SizedBox(width: 16),
          // Speed.
          IconButton(
            onPressed: () => state.setSpeed(state.speed - 0.25),
            icon: const Icon(
              Icons.remove,
              color: CymbraColors.onSurfaceVariant,
            ),
          ),
          Text(
            '${(state.speed * 100).round()}% SPD',
            style: const TextStyle(
              color: CymbraColors.onSurface,
              fontWeight: FontWeight.w600,
            ),
          ),
          IconButton(
            onPressed: () => state.setSpeed(state.speed + 0.25),
            icon: const Icon(Icons.add, color: CymbraColors.onSurfaceVariant),
          ),
          const SizedBox(width: 8),
          // Wait Mode.
          TextButton.icon(
            onPressed: state.toggleWaitMode,
            icon: Icon(
              state.waitMode ? Icons.hourglass_top : Icons.hourglass_disabled,
              color: state.waitMode
                  ? CymbraColors.secondary
                  : CymbraColors.onSurfaceVariant,
            ),
            label: Text(
              'Wait',
              style: TextStyle(
                color: state.waitMode
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
