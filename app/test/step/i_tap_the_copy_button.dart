import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

Future<void> iTapTheCopyButton(WidgetTester tester) async {
  await tester.tap(find.byKey(const ValueKey('btn_copy')));
  await tester.pump();
}
