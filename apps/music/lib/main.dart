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
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'dart:async';

import 'screens/library_screen.dart';
import 'services/audio_service.dart';
import 'src/rust/frb_generated.dart';
import 'theme/cymbra_theme.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  // Lock to landscape: the on-screen keyboard (up to 88 keys) is only legible in
  // landscape. No-op on desktop/web.
  await SystemChrome.setPreferredOrientations(const [
    DeviceOrientation.landscapeLeft,
    DeviceOrientation.landscapeRight,
  ]);
  await RustLib.init();

  // Pre-warm the piano synth at launch (loads the ~50 MB SoundFont) so it is
  // ready before the user picks a piece — keeping the heavy one-time load off
  // the score-selection path. The container is shared with the app so the
  // player reuses this already-initialized AudioService instance.
  final container = ProviderContainer();
  unawaited(container.read(audioServiceProvider).init());

  // Silence the synth when the OS backgrounds/hides the app, so a held voice
  // (note pressed, no note-off yet) doesn't keep ringing while paused.
  WidgetsBinding.instance.addObserver(_AudioLifecycleObserver(container));

  runApp(
    UncontrolledProviderScope(container: container, child: const CymbraApp()),
  );
}

/// Cuts all audio when the app leaves the foreground. `paused`/`hidden` cover
/// mobile backgrounding (and desktop minimise); `inactive` is intentionally not
/// silenced so a brief focus change on desktop doesn't chop a sounding note.
class _AudioLifecycleObserver with WidgetsBindingObserver {
  _AudioLifecycleObserver(this._container);

  final ProviderContainer _container;

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.paused ||
        state == AppLifecycleState.hidden ||
        state == AppLifecycleState.detached) {
      _container.read(audioServiceProvider).allNotesOff();
    }
  }
}

class CymbraApp extends StatelessWidget {
  const CymbraApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Cymbra Music',
      debugShowCheckedModeBanner: false,
      theme: buildCymbraTheme(),
      home: const LibraryScreen(),
    );
  }
}
