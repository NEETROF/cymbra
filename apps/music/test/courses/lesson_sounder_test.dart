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

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/courses/lesson_sounder.dart';

import '../support/fakes.dart';

void main() {
  test('tap sounds the pitch and releases it after the duration', () {
    fakeAsync((async) {
      final audio = RecordingAudioService();
      final sounder = LessonSounder(audio);
      sounder.tap(60);
      expect(audio.noteOns.map((n) => n.pitch), [60]);
      expect(audio.noteOffs, isEmpty);
      async.elapse(const Duration(milliseconds: 350));
      expect(audio.noteOffs, [60]);
      sounder.dispose();
    });
  });

  test('playSequence spaces melodic notes and reports completion', () {
    fakeAsync((async) {
      final audio = RecordingAudioService();
      final sounder = LessonSounder(audio);
      var done = false;
      sounder.playSequence(
        [60, 64],
        noteMs: 500,
        gapMs: 700,
        onDone: () => done = true,
      );
      async.elapse(const Duration(milliseconds: 10));
      expect(audio.noteOns.map((n) => n.pitch), [60]);
      async.elapse(const Duration(milliseconds: 700));
      expect(audio.noteOns.map((n) => n.pitch), [60, 64]);
      expect(audio.noteOffs, [60]);
      async.elapse(const Duration(milliseconds: 600));
      expect(audio.noteOffs, [60, 64]);
      expect(done, isTrue);
      sounder.dispose();
    });
  });

  test('harmonic sequence starts every pitch together', () {
    fakeAsync((async) {
      final audio = RecordingAudioService();
      final sounder = LessonSounder(audio);
      sounder.playSequence([60, 64, 67], harmonic: true, noteMs: 400);
      async.elapse(const Duration(milliseconds: 10));
      expect(audio.noteOns.map((n) => n.pitch), [60, 64, 67]);
      async.elapse(const Duration(milliseconds: 500));
      expect(audio.noteOffs, [60, 64, 67]);
      sounder.dispose();
    });
  });

  test('dispose cancels scheduled playback and silences held voices', () {
    fakeAsync((async) {
      final audio = RecordingAudioService();
      final sounder = LessonSounder(audio);
      sounder.playSequence([60, 64, 67], gapMs: 500);
      async.elapse(const Duration(milliseconds: 10));
      expect(audio.noteOns, hasLength(1));
      sounder.dispose();
      // The sounding voice was released; nothing else ever fires.
      expect(audio.noteOffs, [60]);
      async.elapse(const Duration(seconds: 5));
      expect(audio.noteOns, hasLength(1));
      expect(audio.noteOffs, [60]);
    });
  });

  test('chime plays a rising three-note flourish', () {
    fakeAsync((async) {
      final audio = RecordingAudioService();
      final sounder = LessonSounder(audio);
      sounder.chime();
      async.elapse(const Duration(seconds: 1));
      expect(audio.noteOns.map((n) => n.pitch), [84, 88, 91]);
      expect(audio.noteOffs, hasLength(3));
      sounder.dispose();
    });
  });
}
