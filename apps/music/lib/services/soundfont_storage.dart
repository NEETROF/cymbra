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

import 'dart:io' show Directory;

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'soundfont_storage.g.dart';

/// Durable, app-private directory that holds imported and download-cached
/// SoundFonts, so a user's chosen `.sf2` survives relaunches (unlike the OS temp
/// dir, which can be purged). Behind a provider so tests override it with a
/// throwaway directory and never touch the real application-support directory.
@Riverpod(keepAlive: true)
Future<Directory> soundFontStorageDir(Ref ref) async {
  final base = await getApplicationSupportDirectory();
  final dir = Directory('${base.path}/soundfonts');
  if (!await dir.exists()) await dir.create(recursive: true);
  return dir;
}
