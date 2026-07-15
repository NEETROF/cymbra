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
import 'package:url_launcher/url_launcher.dart';

part 'legal_links.g.dart';

/// The Terms of Service and Privacy Policy URLs for a given UI language.
typedef LegalLinks = ({Uri terms, Uri privacy});

/// Resolves the legal-page URLs for [languageCode].
///
/// French uses the French pages; every other supported language falls back to
/// the English pages. Pure and host-testable — no provider or platform access.
LegalLinks legalLinksFor(String languageCode) {
  if (languageCode == 'fr') {
    return (
      terms: Uri.parse('https://cymbra.app/cgu/'),
      privacy: Uri.parse('https://cymbra.app/confidentialite/'),
    );
  }
  return (
    terms: Uri.parse('https://cymbra.app/en/terms/'),
    privacy: Uri.parse('https://cymbra.app/en/privacy/'),
  );
}

/// Seam over `url_launcher` so widgets can open a legal page without the native
/// plugin loaded. [PlayerScreen]/[EntryScreen] depend on this interface; tests
/// override [legalLinkLauncherProvider] with a fake that records the URL.
abstract class LegalLinkLauncher {
  /// Open [url] in an external browser. Best-effort: a legal link that fails to
  /// open must never crash the UI.
  Future<void> open(Uri url);
}

/// Production [LegalLinkLauncher] backed by `url_launcher`.
class UrlLauncherLegalLinkLauncher implements LegalLinkLauncher {
  const UrlLauncherLegalLinkLauncher();

  @override
  Future<void> open(Uri url) async {
    // Best-effort: swallow launch failures (no browser handler, etc.) so a
    // legal link can never bring down the entry or settings screen.
    try {
      await launchUrl(url, mode: LaunchMode.externalApplication);
    } catch (_) {}
  }
}

/// Production launcher provider. Override in tests with a fake.
@riverpod
LegalLinkLauncher legalLinkLauncher(Ref ref) =>
    const UrlLauncherLegalLinkLauncher();
