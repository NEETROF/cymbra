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

import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/rust/api/audio.dart' as audio_api;

part 'sound_clip_player.g.dart';

/// Plays a short in-memory audio clip (a WAV byte buffer) — the SoundFont preview
/// audition path (change: add-soundfont-entitlement-previews). A seam over the audio
/// engine so [SoundFontPreviewService](soundfont_preview_service.dart) can be driven
/// by a fake in unit/widget tests (which run on the Dart VM with no native library).
abstract class SoundClipPlayer {
  /// Play [bytes] (a WAV clip), looping until [stop]; replaces any current clip.
  Future<void> play(Uint8List bytes);

  /// Stop any current clip.
  Future<void> stop();
}

/// Production [SoundClipPlayer]: plays the WAV clip through the **native Rust audio
/// engine** (the same cross-platform cpal stack as the piano synth), so a locked
/// font's preview auditions without any third-party audio plugin. The engine loops
/// the clip until [stop]. A silent no-op if the engine has not started.
class NativeClipPlayer implements SoundClipPlayer {
  @override
  Future<void> play(Uint8List bytes) async =>
      audio_api.playPreviewClip(wavBytes: bytes);

  @override
  Future<void> stop() async => audio_api.stopPreviewClip();
}

/// Production clip-player provider. Override in tests with a recording fake.
@riverpod
SoundClipPlayer soundClipPlayer(Ref ref) => NativeClipPlayer();
