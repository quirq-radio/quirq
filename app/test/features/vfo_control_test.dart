// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import './step/the_vfo_screen_is_shown.dart';
import './step/vfo_a_is_active.dart';
import './step/vfo_a_is_selected.dart';
import './step/i_tap_vfo_b.dart';
import './step/vfo_b_is_active.dart';
import './step/vfo_a_frequency_is_set_to7189500.dart';
import './step/vfo_b_frequency_is_set_to14220000.dart';
import './step/i_tap_the_swap_button.dart';
import './step/vfo_a_frequency_should_be14220000.dart';
import './step/vfo_b_frequency_should_be7189500.dart';
import './step/i_tap_the_copy_button.dart';

void main() {
  group('''VFO control''', () {
    Future<void> bddSetUp(WidgetTester tester) async {
      await theVfoScreenIsShown(tester);
    }

    testWidgets('''VFO A is active by default''', (tester) async {
      await bddSetUp(tester);
      await vfoAIsActive(tester);
    });
    testWidgets('''Tapping VFO B switches the active VFO''', (tester) async {
      await bddSetUp(tester);
      await vfoAIsSelected(tester);
      await iTapVfoB(tester);
      await vfoBIsActive(tester);
    });
    testWidgets('''Swapping VFOs exchanges their frequencies''',
        (tester) async {
      await bddSetUp(tester);
      await vfoAFrequencyIsSetTo7189500(tester);
      await vfoBFrequencyIsSetTo14220000(tester);
      await iTapTheSwapButton(tester);
      await vfoAFrequencyShouldBe14220000(tester);
      await vfoBFrequencyShouldBe7189500(tester);
    });
    testWidgets('''Copying VFO copies the frequency''', (tester) async {
      await bddSetUp(tester);
      await vfoAFrequencyIsSetTo7189500(tester);
      await vfoAIsSelected(tester);
      await iTapTheCopyButton(tester);
      await vfoBFrequencyShouldBe7189500(tester);
    });
  });
}
