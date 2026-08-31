import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/content_report_service.dart';
import 'package:music/state/content_report_notifier.dart';

import 'content_report_notifier_test.mocks.dart';

@GenerateNiceMocks([MockSpec<ContentReportService>()])
ProviderContainer _container(ContentReportService service) {
  final c = ProviderContainer(
    overrides: [contentReportServiceProvider.overrideWithValue(service)],
  );
  addTearDown(c.dispose);
  return c;
}

void main() {
  group('ContentReport', () {
    test('forwards the report and bumps the sent sequence', () async {
      final service = MockContentReportService();
      final c = _container(service);
      final notifier = c.read(contentReportProvider.notifier);

      await notifier.submit(
        target: ReportTarget.catalogScore,
        targetId: 'score-1',
        reason: ReportReason.copyright,
        note: 'this is my arrangement',
      );

      verify(
        service.report(
          target: ReportTarget.catalogScore,
          targetId: 'score-1',
          reason: ReportReason.copyright,
          note: 'this is my arrangement',
        ),
      ).called(1);
      final state = c.read(contentReportProvider);
      expect(state.sentSeq, 1);
      expect(state.errorSeq, 0);
      expect(state.sending, isFalse);
    });

    test('a failure lands in the state and is never thrown', () async {
      final service = MockContentReportService();
      when(
        service.report(
          target: anyNamed('target'),
          targetId: anyNamed('targetId'),
          reason: anyNamed('reason'),
          note: anyNamed('note'),
        ),
      ).thenThrow(Exception('unavailable'));
      final c = _container(service);

      // The call site is a button, not an exception handler.
      await c
          .read(contentReportProvider.notifier)
          .submit(
            target: ReportTarget.soundFont,
            targetId: 'sf-1',
            reason: ReportReason.other,
          );

      final state = c.read(contentReportProvider);
      expect(state.errorSeq, 1);
      expect(state.sentSeq, 0);
      expect(state.sending, isFalse);
    });

    test(
      'two successes bump the sequence twice so a listener fires again',
      () async {
        final service = MockContentReportService();
        final c = _container(service);
        final notifier = c.read(contentReportProvider.notifier);

        await notifier.submit(
          target: ReportTarget.catalogScore,
          targetId: 'score-1',
          reason: ReportReason.other,
        );
        await notifier.submit(
          target: ReportTarget.catalogScore,
          targetId: 'score-2',
          reason: ReportReason.other,
        );

        expect(c.read(contentReportProvider).sentSeq, 2);
      },
    );
  });
}
