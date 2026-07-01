import 'package:flutter_test/flutter_test.dart';
import '_shared.dart';

Future<void> vfoModeShouldBe(WidgetTester tester, String vfo, String mode) async {
  expect(
    scenarioRadio.modeOf(vfo).toUpperCase(),
    mode.toUpperCase(),
    reason: 'mode of VFO $vfo',
  );
}
