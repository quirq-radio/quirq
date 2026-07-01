import 'package:flutter_test/flutter_test.dart';
import '_shared.dart';

Future<void> vfoIsSelected(WidgetTester tester, String vfo) async {
  scenarioRadio.selectVfo(vfo);
  await tester.pump();
}
