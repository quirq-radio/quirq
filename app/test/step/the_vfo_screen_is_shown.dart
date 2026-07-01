import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';
import 'package:quirq/radio_connection.dart';
import 'package:quirq/vfo_screen.dart';
import '_shared.dart';

Future<void> theVfoScreenIsShown(WidgetTester tester) async {
  scenarioRadio = RadioConnection();
  await tester.pumpWidget(
    ChangeNotifierProvider.value(
      value: scenarioRadio,
      child: const MaterialApp(home: VfoScreen()),
    ),
  );
  await tester.pump();
}
