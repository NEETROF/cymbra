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

/// The doubles the showcase video runs on (change: add-site-music-showcase).
///
/// The video is rendered frame by frame on a [VirtualClock] rather than filmed:
/// each frame is drawn at an exact instant, and every command the app sends to
/// its sound engine is logged at that same instant by [RecordingAudioService].
/// `tool/record_showcase_video.sh` then renders that log through the app's own
/// mixer (`showcase-render`), so the picture and the sound share one clock.
library;

import 'package:music/services/audio_service.dart';
import 'package:music/services/clock_service.dart';

/// A [Clock] the scenario moves by hand, one video frame at a time.
class VirtualClock implements Clock {
  /// The current instant, in milliseconds since the video started.
  double ms = 0;

  @override
  int nowMs() => ms.round();
}

/// The SoundFont the app installs at start-up ([FrbAudioService.init]).
const String kShowcaseDefaultFont = 'UprightPianoKW-20220221.sf2';

/// An [AudioService] that sounds nothing and logs every command, stamped with
/// [clock], in the format `showcase-render` reads (`<ms> <verb> [args]`).
class RecordingAudioService implements AudioService {
  RecordingAudioService(this.clock);

  final VirtualClock clock;
  final List<String> _lines = [];

  /// The log so far, one command per line.
  List<String> get lines => List.unmodifiable(_lines);

  void _log(String command) =>
      _lines.add('${clock.ms.toStringAsFixed(3)} $command');

  /// Ends the log at the current instant: the length of the video.
  void end() => _log('end');

  /// The file name the engine would load: the log is rendered on another
  /// machine, against the repository's `assets/soundfonts/`, so only the name
  /// travels.
  static String fontName(String path) => path.split(RegExp(r'[/\\]')).last;

  bool _initialised = false;

  /// Several callers await `init` (start-up, the player, audio routing); like
  /// the real service it happens once, so a late caller cannot log a font swap
  /// that would silence the notes sounding at that instant.
  @override
  Future<void> init() async {
    if (_initialised) return;
    _initialised = true;
    _log('font $kShowcaseDefaultFont');
  }

  @override
  Future<void> loadSoundFont(String sf2Path) async =>
      _log('font ${fontName(sf2Path)}');

  @override
  Future<bool> loadSoundFontAwaited(String sf2Path) async {
    _log('font ${fontName(sf2Path)}');
    return true;
  }

  @override
  void noteOn(int pitch, {int velocity = AudioService.defaultVelocity}) =>
      _log('on $pitch $velocity');

  @override
  void noteOff(int pitch) => _log('off $pitch');

  @override
  void drumOn(int key, {int velocity = AudioService.defaultVelocity}) =>
      _log('drum-on $key $velocity');

  @override
  void drumOff(int key) => _log('drum-off $key');

  @override
  void allNotesOff() => _log('all-off');

  @override
  void metronomeClick({required bool accent}) =>
      _log('click ${accent ? 1 : 0}');
}
