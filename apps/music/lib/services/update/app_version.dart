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

/// The app's own version format, ordered (change: add-desktop-auto-update,
/// design D6).
///
/// `major.minor.patch+build`, e.g. `1.24.0+32` — exactly what `pubspec.yaml`
/// declares and what `package_info_plus` reports back at runtime. Hand-rolled
/// rather than a semver package on purpose: strict semver treats build metadata
/// as **not** participating in precedence, whereas here the build number is a
/// real tiebreaker (two releases can share a triple). Borrowing semver's rule
/// would silently make `1.24.0+33` "not newer than" `1.24.0+32`.
///
/// The ordering must agree with the backend's `version_order` projection
/// (`backend/updates/src/core.rs`): a disagreement would have the feed offer an
/// update the client then refuses as non-newer, forever and silently.
class AppVersion implements Comparable<AppVersion> {
  const AppVersion(this.major, this.minor, this.patch, this.build);

  final int major;
  final int minor;
  final int patch;

  /// The `+build` component. Absent is treated as `0`, so `1.2.3` and `1.2.3+0`
  /// compare equal.
  final int build;

  /// Parses `major.minor.patch` with an optional `+build`.
  ///
  /// Returns `null` for anything else — a leading `v`, a missing component, a
  /// pre-release suffix, a negative number. `null` is not an error to report to
  /// the user: it means the manifest (or the platform's own version string) is
  /// not something this build can reason about, and the correct response is to
  /// offer nothing.
  static AppVersion? tryParse(String raw) {
    final input = raw.trim();
    if (input.isEmpty) return null;
    final plus = input.indexOf('+');
    final triple = plus == -1 ? input : input.substring(0, plus);
    final buildPart = plus == -1 ? '0' : input.substring(plus + 1);
    if (buildPart.isEmpty) return null;
    final parts = triple.split('.');
    if (parts.length != 3) return null;
    final numbers = <int>[];
    for (final part in [...parts, buildPart]) {
      // `int.tryParse` accepts a leading `+`/`-`; only plain digits are a
      // version component.
      if (part.isEmpty || !RegExp(r'^\d+$').hasMatch(part)) return null;
      final value = int.tryParse(part);
      if (value == null) return null;
      numbers.add(value);
    }
    return AppVersion(numbers[0], numbers[1], numbers[2], numbers[3]);
  }

  @override
  int compareTo(AppVersion other) {
    final byMajor = major.compareTo(other.major);
    if (byMajor != 0) return byMajor;
    final byMinor = minor.compareTo(other.minor);
    if (byMinor != 0) return byMinor;
    final byPatch = patch.compareTo(other.patch);
    if (byPatch != 0) return byPatch;
    return build.compareTo(other.build);
  }

  bool operator >(AppVersion other) => compareTo(other) > 0;
  bool operator <(AppVersion other) => compareTo(other) < 0;
  bool operator >=(AppVersion other) => compareTo(other) >= 0;
  bool operator <=(AppVersion other) => compareTo(other) <= 0;

  @override
  bool operator ==(Object other) =>
      other is AppVersion && compareTo(other) == 0;

  @override
  int get hashCode => Object.hash(major, minor, patch, build);

  /// `1.24.0+32` — the build is always rendered, so a round trip through
  /// [tryParse] is lossless.
  @override
  String toString() => '$major.$minor.$patch+$build';

  /// `1.24.0` — what a user should read. The build number is an internal
  /// counter and means nothing to them.
  String get display => '$major.$minor.$patch';
}
