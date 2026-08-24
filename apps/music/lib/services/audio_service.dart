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

import 'package:flutter/foundation.dart' show debugPrint, kDebugMode, kIsWeb;
import 'package:flutter/services.dart' show MethodChannel, rootBundle;
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

  /// Swaps the synthesizer's active SoundFont at runtime from the `.sf2` file at
  /// [sf2Path], so later notes (on-screen, computer keyboard, MIDI, playback)
  /// sound with the newly chosen piano — without restarting the audio output.
  ///
  /// Non-throwing and degradable: a no-op when audio never started, and a
  /// missing/invalid file leaves the current piano in place (the engine keeps
  /// the working synth). An all-notes-off is applied across the swap so a held
  /// voice does not hang.
  Future<void> loadSoundFont(String sf2Path);

  /// Swaps the SoundFont like [loadSoundFont] but resolves only once the
  /// outcome is known (change: add-drum-audio-channel): `true` when the
  /// incoming font is installed and sounding, `false` when the swap failed and
  /// the previous font was kept (or audio is unavailable). The player's
  /// percussion-readiness gate awaits this so a drum score never sounds
  /// through the still-loaded piano font. Non-throwing, like every entry here.
  Future<bool> loadSoundFontAwaited(String sf2Path);

  /// Sounds a piano voice for [pitch] (7-bit MIDI) at [velocity].
  void noteOn(int pitch, {int velocity = defaultVelocity});

  /// Releases the voice for [pitch].
  void noteOff(int pitch);

  /// Sounds a percussion stroke for General MIDI [key] at [velocity] on the
  /// drum channel, where the active kit font's bank-128 presets resolve
  /// (change: add-drum-audio-channel). The melodic pair above is untouched.
  void drumOn(int key, {int velocity = defaultVelocity});

  /// Releases the drum voice for [key]. Kit voices mostly self-terminate; the
  /// paired release keeps the engine's voice bookkeeping exact.
  void drumOff(int key);

  /// Releases every sounding voice on every channel — melodic and drum alike
  /// (stop / restart / seek / loop).
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

  Future<void>? _init;
  bool _failed = false;

  /// Memoized: every caller gets the **in-flight** future, so awaiting init is
  /// a real completion barrier. A boolean started-guard here let the second
  /// caller return immediately and act (select an output at startup) before the
  /// engine had published its command channels — the call was silently dropped.
  @override
  Future<void> init() => _init ??= _doInit();

  Future<void> _doInit() async {
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
      // Android's output is owned by the platform, not by the engine: `cpal`'s
      // AAudio path there cannot enumerate the device's outputs and does not
      // deliver usable audio to a USB-audio instrument, so `EngineOutput.kt` runs
      // an `AudioTrack` that pulls rendered samples from Rust. Starting it is what
      // makes any sound come out at all on that platform.
      if (!kIsWeb && Platform.isAndroid) await _startAndroidOutput();
      if (sw != null) {
        debugPrint('[audio] soundfont ready: ${sw.elapsedMilliseconds}ms');
      }
    } catch (_) {
      // No audio device, or the SoundFont could not be extracted/parsed: remain
      // a silent no-op for the rest of the session.
      _failed = true;
    }
  }

  /// Starts the platform-owned output on Android (see [init]). Non-throwing: a
  /// failure leaves the app silent rather than broken, like every other path here.
  Future<void> _startAndroidOutput() async {
    try {
      await const MethodChannel(
        'org.cymbra.music/audio_routing',
      ).invokeMethod<bool>('startOutput', {'deviceId': -1});
    } catch (_) {}
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
  Future<void> loadSoundFont(String sf2Path) async {
    // Even if init never ran, forwarding is harmless: the engine drops the swap
    // when no audio thread is running. Guarding on [_failed] avoids the bridge
    // call once we know audio is unavailable this session.
    if (_failed) return;
    try {
      // Tiny sync call: the engine reads/parses the file off the UI isolate and
      // hands the parsed instrument to the audio thread. The path (not the
      // bytes) crosses the bridge, so a large SoundFont never freezes the UI.
      audio_api.audioLoadSoundfont(sf2Path: sf2Path);
    } catch (_) {}
  }

  @override
  Future<bool> loadSoundFontAwaited(String sf2Path) async {
    // Same guard as [loadSoundFont]; a known-dead audio session resolves false
    // immediately — the honest "the kit is not sounding" answer.
    if (_failed) return false;
    try {
      return await audio_api.audioLoadSoundfontAwaited(sf2Path: sf2Path);
    } catch (_) {
      return false;
    }
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
  void drumOn(int key, {int velocity = AudioService.defaultVelocity}) {
    if (_failed) return;
    try {
      audio_api.drumOn(key: key, velocity: velocity);
    } catch (_) {}
  }

  @override
  void drumOff(int key) {
    if (_failed) return;
    try {
      audio_api.drumOff(key: key);
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
