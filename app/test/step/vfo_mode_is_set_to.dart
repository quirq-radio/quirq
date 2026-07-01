import 'package:flutter_test/flutter_test.dart';
import '_shared.dart';

Future<void> vfoModeIsSetTo(WidgetTester tester, String vfo, String mode) async {
  scenarioRadio.setVfoModeForTesting(vfo, mode);
  await tester.pump();
}
