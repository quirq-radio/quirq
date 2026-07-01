import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

Future<void> iTapTheSwapButton(WidgetTester tester) async {
  await tester.tap(find.byKey(const ValueKey('btn_swap')));
  await tester.pump();
}
