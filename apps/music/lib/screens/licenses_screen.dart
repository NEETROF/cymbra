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

import '../theme/cymbra_theme.dart';

/// The open-source licenses page, inset for the landscape sensor housing.
///
/// Flutter's [LicensePage] pads the **top** inset only. This app is
/// landscape-locked, so the housing sits on a *side*: the back button and the
/// package list ran underneath it. Same reasoning as `PlanScreen`, which pads
/// its own bar and body for exactly this reason.
///
/// [LicensePage] builds its own `Scaffold`, so the horizontal inset is taken
/// outside it — over a [ColoredBox] in the app background, otherwise the strip
/// the inset frees would show the route below rather than the app.
class LicensesScreen extends StatelessWidget {
  const LicensesScreen({super.key});

  @override
  Widget build(BuildContext context) => const ColoredBox(
    color: CymbraColors.background,
    child: SafeArea(
      top: false,
      bottom: false,
      child: LicensePage(applicationName: 'Cymbra'),
    ),
  );
}
