import 'package:flutter_test/flutter_test.dart';
import '_shared.dart';

Future<void> vfoIsActive(WidgetTester tester, String vfo) async {
  expect(scenarioRadio.activeVfo, vfo, reason: 'active VFO');
}
