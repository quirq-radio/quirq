import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

Future<void> iTapVfo(WidgetTester tester, String vfo) async {
  await tester.tap(find.byKey(ValueKey('vfo_$vfo')));
  await tester.pump();
}
