import 'package:flutter_test/flutter_test.dart';
import '_shared.dart';

Future<void> vfoFrequencyIsSetTo(WidgetTester tester, String vfo, int hz) async {
  await scenarioRadio.setFreq(vfo, hz);
  await tester.pump();
}
