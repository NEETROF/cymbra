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

import 'dart:io';

import 'package:flutter/foundation.dart' show debugPrint, kDebugMode;
import 'package:flutter/services.dart' show rootBundle;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/rust/api/audio.dart' as audio_api;

part 'audio_service.g.dart';

/// Asset path of the bundled CC0 piano SoundFont (see
/// `assets/soundfonts/CREDITS.md`). The filename is version-stamped, so it
/// doubles as the cache key when extracted to disk.
const String _soundFontAsset = 'assets/soundfonts/UprightPianoKW-20220221.sf2';

/// Production audio engine provider. Override in tests with a recording fake.
@riverpod
AudioService audioService(Ref ref) => FrbAudioService();

/// Seam over the Rust piano synthesizer.
///
/// [PlayerState] depends on this interface instead of the generated
/// flutter_rust_bridge functions directly, so it can be driven by a fake in
/// unit/widget tests (which run on the Dart VM with no native library loaded).
/// The production wiring is [FrbAudioService], which loads the SoundFont and
/// forwards to the bridge.
abstract class AudioService {
  /// Velocity used for sources that carry no pressure (on-screen keyboard,
  /// computer-keyboard fallback).
  static const int defaultVelocity = 100;

  /// Loads the SoundFont and starts the audio output. Idempotent and
  /// non-throwing: on any failure (no device, missing/invalid font) the service
  /// stays a silent no-op and the app keeps working.
  Future<void> init();

  /// Sounds a piano voice for [pitch] (7-bit MIDI) at [velocity].
  void noteOn(int pitch, {int velocity = defaultVelocity});

  /// Releases the voice for [pitch].
  void noteOff(int pitch);

  /// Releases every sounding voice (stop / restart / seek / loop).
  void allNotesOff();

  /// Sounds a one-shot metronome click, independent of the piano voices. When
  /// [accent] is true it marks the downbeat (higher and louder). Self-terminating
  /// — there is no matching release call.
  void metronomeClick({required bool accent});
}

/// Production [AudioService] backed by the generated flutter_rust_bridge API.
///
/// All entry points degrade gracefully: if [init] failed (or has not run yet),
/// note events are dropped rather than throwing, so audio never crashes the
/// player. Every bridge call is additionally guarded so a native hiccup cannot
/// propagate into the UI.
class FrbAudioService implements AudioService {
  FrbAudioService();

  bool _initStarted = false;
  bool _failed = false;

  @override
  Future<void> init() async {
    if (_initStarted) return;
    _initStarted = true;
    try {
      final sw = kDebugMode ? (Stopwatch()..start()) : null;
      // Hand the engine a *file path*, not the bytes: the ~50 MB SoundFont is
      // extracted from the bundle to a cached file once (off the UI thread) and
      // Rust reads it straight from disk. Pushing the bytes across the bridge
      // would serialize them on the UI isolate and freeze it for seconds.
      final path = await _ensureSoundFontFile();
      // Tiny sync call: just spawns the audio thread, which reads/parses the
      // file and opens the device.
      audio_api.audioInit(sf2Path: path);
      if (sw != null) {
        debugPrint('[audio] soundfont ready: ${sw.elapsedMilliseconds}ms');
      }
    } catch (_) {
      // No audio device, or the SoundFont could not be extracted/parsed: remain
      // a silent no-op for the rest of the session.
      _failed = true;
    }
  }

  /// Ensures the bundled SoundFont exists as a plain file and returns its path,
  /// extracting it from the asset bundle only on first run (the version-stamped
  /// filename is the cache key). The 54 MB write happens on dart:io's I/O
  /// threads, not the UI isolate.
  Future<String> _ensureSoundFontFile() async {
    final name = _soundFontAsset.split('/').last;
    final file = File('${Directory.systemTemp.path}/cymbra_soundfonts/$name');
    if (!await file.exists()) {
      await file.parent.create(recursive: true);
      final data = await rootBundle.load(_soundFontAsset);
      await file.writeAsBytes(
        data.buffer.asUint8List(data.offsetInBytes, data.lengthInBytes),
        flush: true,
      );
    }
    return file.path;
  }

  @override
  void noteOn(int pitch, {int velocity = AudioService.defaultVelocity}) {
    if (_failed) return;
    try {
      audio_api.noteOn(pitch: pitch, velocity: velocity);
    } catch (_) {}
  }

  @override
  void noteOff(int pitch) {
    if (_failed) return;
    try {
      audio_api.noteOff(pitch: pitch);
    } catch (_) {}
  }

  @override
  void allNotesOff() {
    if (_failed) return;
    try {
      audio_api.allNotesOff();
    } catch (_) {}
  }

  @override
  void metronomeClick({required bool accent}) {
    if (_failed) return;
    try {
      audio_api.metronomeClick(accent: accent);
    } catch (_) {}
  }
}
