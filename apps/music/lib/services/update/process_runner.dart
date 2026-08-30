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

import 'dart:io' show Process, ProcessStartMode, exit;

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'process_runner.g.dart';

/// The one place the updater touches the OS process table (change:
/// add-desktop-auto-update, task 7.7).
///
/// A seam purely so **no test ever spawns a process or exits the test runner**.
/// The installer implementations are then plain logic over this interface, and
/// their tests assert on the exact command line — which is the part that is easy
/// to get subtly wrong and impossible to notice until an update silently does
/// nothing on a user's machine.
abstract class ProcessRunner {
  /// Starts [executable] detached: it must outlive this process, because this
  /// process is about to end.
  Future<void> startDetached(String executable, List<String> arguments);

  /// Runs [executable] to completion and returns its exit code. Used for the
  /// short auxiliary commands the swap depends on (`chmod`), where firing and
  /// forgetting would race the very next step.
  Future<int> run(String executable, List<String> arguments);

  /// Ends the running app. Called only after a successful [startDetached] —
  /// on Windows the installer cannot replace a running `.exe`, and on Linux the
  /// relaunched AppImage is the one that should own the window.
  void quit();
}

/// Production [ProcessRunner].
class OsProcessRunner implements ProcessRunner {
  const OsProcessRunner();

  @override
  Future<void> startDetached(String executable, List<String> arguments) async {
    await Process.start(
      executable,
      arguments,
      // `detached` and not `detachedWithStdio`: the child must survive this
      // process exiting, and nothing here reads its output.
      mode: ProcessStartMode.detached,
    );
  }

  @override
  Future<int> run(String executable, List<String> arguments) async {
    final result = await Process.run(executable, arguments);
    return result.exitCode;
  }

  @override
  void quit() => exit(0);
}

/// The process seam, behind a provider so tests override it with a mock.
@Riverpod(keepAlive: true)
ProcessRunner processRunner(Ref ref) => const OsProcessRunner();
