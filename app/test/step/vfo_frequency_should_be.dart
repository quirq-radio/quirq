import 'package:flutter_test/flutter_test.dart';
import '_shared.dart';

Future<void> vfoFrequencyShouldBe(WidgetTester tester, String vfo, int hz) async {
  expect(scenarioRadio.freqOf(vfo), hz, reason: 'frequency of VFO $vfo');
}
