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
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../services/legal_links.dart';
import '../theme/cymbra_theme.dart';

/// Fine print, sized and coloured the same wherever it appears: the sign-in
/// consent line and the subscription flow, which App Store guideline 3.1.2
/// requires to carry the Terms and privacy links itself.
const legalBodyStyle = TextStyle(
  color: CymbraColors.onSurfaceVariant,
  fontSize: 12,
  height: 1.4,
);

/// A tappable legal link.
///
/// Goes through [legalLinkLauncherProvider] rather than opening a URL directly,
/// so a test asserts on the [Uri] instead of on a browser. Sized to sit inline
/// in a [Text.rich] span as well as on its own.
class LegalLink extends ConsumerWidget {
  const LegalLink({required this.label, required this.url, super.key});

  final String label;
  final Uri url;

  static const _style = TextStyle(
    color: CymbraColors.primary,
    fontSize: 12,
    height: 1.4,
    decoration: TextDecoration.underline,
    decorationColor: CymbraColors.primary,
  );

  @override
  Widget build(BuildContext context, WidgetRef ref) => GestureDetector(
    onTap: () => ref.read(legalLinkLauncherProvider).open(url),
    child: Text(label, style: _style),
  );
}
