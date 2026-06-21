import 'package:flutter_test/flutter_test.dart';
import 'package:music/main.dart';
import 'package:music/src/rust/frb_generated.dart';
import 'package:integration_test/integration_test.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await RustLib.init());
  testWidgets('App boots and shows its title', (WidgetTester tester) async {
    await tester.pumpWidget(const CymbraApp());
    await tester.pump();
    expect(find.text('Cymbra Music'), findsWidgets);
  });
}
