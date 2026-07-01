import 'package:flutter_test/flutter_test.dart';
import '_shared.dart';

Future<void> theActiveModeIsSetTo(WidgetTester tester, String mode) async {
  scenarioRadio.setVfoModeForTesting(scenarioRadio.activeVfo, mode);
  await tester.pump();
}
