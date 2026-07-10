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

import 'dart:io' show Platform;

import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'platform_info.g.dart';

/// Whether the app is running on Android.
///
/// Behind a provider (an injectable seam) rather than calling `Platform.isAndroid`
/// inline, so widget tests can drive both Android and non-Android states
/// deterministically by overriding it — the VM the tests run on always reports
/// the host OS otherwise. The `kIsWeb` guard keeps `dart:io`'s `Platform` off the
/// web path, where it throws.
@riverpod
bool isAndroid(Ref ref) => !kIsWeb && Platform.isAndroid;
