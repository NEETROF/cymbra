import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/services/content_report_service.dart';
import 'package:music/widgets/app_snackbar.dart';
import 'package:music/widgets/report_content_sheet.dart';

import '../support/localized.dart';
import 'report_content_sheet_test.mocks.dart';

@GenerateNiceMocks([MockSpec<ContentReportService>()])
/// Pump a screen whose single button opens the report sheet, so the test drives
/// the real entry point rather than the private sheet widget.
Future<void> _pumpOpener(
  WidgetTester tester,
  ContentReportService service,
) async {
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: () {
        final c = ProviderContainer(
          overrides: [contentReportServiceProvider.overrideWithValue(service)],
        );
        addTearDown(c.dispose);
        return c;
      }(),
      child: localizedApp(
        Builder(
          builder: (context) => Scaffold(
            body: Center(
              child: ElevatedButton(
                onPressed: () => showReportContentSheet(
                  context,
                  target: ReportTarget.catalogScore,
                  targetId: 'score-1',
                ),
                child: const Text('open'),
              ),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('open'));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('sends the picked reason and the typed note, then closes', (
    tester,
  ) async {
    final service = MockContentReportService();
    await _pumpOpener(tester, service);

    // The sheet defaults to the rights-infringement reason; pick another one.
    await tester.tap(find.byKey(const Key('report-reason-inappropriate')));
    await tester.pump();
    await tester.enterText(find.byKey(const Key('report-note')), '  rude  ');
    await tester.tap(find.byKey(const Key('report-submit')));
    await tester.pumpAndSettle();

    verify(
      service.report(
        target: ReportTarget.catalogScore,
        targetId: 'score-1',
        reason: ReportReason.inappropriate,
        note: '  rude  ',
      ),
    ).called(1);
    // The listener pops the sheet on success…
    expect(find.byKey(const Key('report-submit')), findsNothing);
    // …and acknowledges over the route that replaced it.
    final l10n = await AppLocalizations.delegate.load(const Locale('en'));
    expect(find.text(l10n.reportSent), findsOneWidget);

    // The toast holds a dismissal Timer; the binding verifies no timer is pending
    // at the END of the body, so tearDown would be too late.
    dismissAppToast();
    await tester.pump();
  });

  testWidgets('a failure keeps the sheet open and shows a localized message', (
    tester,
  ) async {
    final service = MockContentReportService();
    when(
      service.report(
        target: anyNamed('target'),
        targetId: anyNamed('targetId'),
        reason: anyNamed('reason'),
        note: anyNamed('note'),
      ),
    ).thenThrow(Exception('grpc: UNAVAILABLE deadline exceeded'));
    await _pumpOpener(tester, service);

    await tester.tap(find.byKey(const Key('report-submit')));
    await tester.pumpAndSettle();

    // Still open, so the reporter can retry without re-typing.
    expect(find.byKey(const Key('report-submit')), findsOneWidget);
    final l10n = await AppLocalizations.delegate.load(const Locale('en'));
    expect(find.text(l10n.reportFailed), findsOneWidget);
    // The raw transport string never reaches the UI.
    expect(find.textContaining('UNAVAILABLE'), findsNothing);

    dismissAppToast();
    await tester.pump();
  });
}
