import 'package:flutter_test/flutter_test.dart';
import '_shared.dart';

Future<void> theActiveModeShouldBe(WidgetTester tester, String mode) async {
  expect(
    scenarioRadio.mode.toUpperCase(),
    mode.toUpperCase(),
    reason: 'active mode (VFO ${scenarioRadio.activeVfo})',
  );
}
