import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

Future<void> iTapModeButton(WidgetTester tester, String mode) async {
  await tester.tap(find.byKey(ValueKey('mode_$mode')));
  await tester.pump();
}
