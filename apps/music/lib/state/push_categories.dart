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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../l10n/gen/app_localizations.dart';

part 'push_categories.g.dart';

/// One notification category the user can switch on or off (change:
/// add-push-notifications).
///
/// A *category* is declared by the feature that owns the notification type; the
/// push platform itself ships none. Its [id] must match the category string the
/// backend send job uses and the three feature-flag keys built by the backend's
/// `registry::category_enabled_key` / `category_hour_key` /
/// `category_foreground_key`.
///
/// Deliberately **no** presentation field here (add-foreground-notifications):
/// whether a category surfaces in the foreground is a hot-reloadable back-office
/// flag that travels on each message — a compiled-in constant would turn a
/// product decision into an app release. This class carries only what the
/// settings toggle needs.
class PushCategory {
  const PushCategory({
    required this.id,
    required this.label,
    this.defaultEnabled = true,
  });

  /// Stable identifier, e.g. `practice_streak`.
  final String id;

  /// Localized user-facing label for the settings switch.
  final String Function(AppLocalizations l10n) label;

  /// What an *absent* stored preference means — must match the `default_pref` the
  /// feature's dispatch job sends, so the switch shows the truth before the user
  /// has ever touched it.
  final bool defaultEnabled;
}

/// The notification categories this build knows about.
///
/// **Empty by design**: the push platform ships no concrete notification type
/// (design D6). A feature that adds one appends its [PushCategory] here — and
/// nothing else in the settings UI, the preference notifier, or the backend
/// platform changes. With an empty list the settings menu renders no notification
/// switches at all.
@Riverpod(keepAlive: true)
List<PushCategory> pushCategories(Ref ref) => const <PushCategory>[];
