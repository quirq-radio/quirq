// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import './step/the_vfo_screen_is_shown.dart';
import './step/the_active_mode_is_set_to_lsb.dart';
import './step/i_tap_mode_button_usb.dart';
import './step/the_active_mode_should_be_usb.dart';
import './step/the_active_mode_is_set_to_usb.dart';
import './step/i_tap_mode_button_lsb.dart';
import './step/the_active_mode_should_be_lsb.dart';
import './step/vfo_a_mode_is_set_to_lsb.dart';
import './step/vfo_b_mode_is_set_to_usb.dart';
import './step/vfo_a_is_selected.dart';
import './step/i_tap_vfo_b.dart';
import './step/i_tap_vfo_a.dart';
import './step/i_tap_the_swap_button.dart';
import './step/vfo_a_mode_should_be_usb.dart';
import './step/vfo_b_mode_should_be_lsb.dart';
import './step/vfo_a_mode_is_set_to_cw.dart';
import './step/i_tap_the_copy_button.dart';
import './step/vfo_b_mode_should_be_cw.dart';

void main() {
  group('''Mode switching''', () {
    Future<void> bddSetUp(WidgetTester tester) async {
      await theVfoScreenIsShown(tester);
    }

    testWidgets('''Selecting USB mode''', (tester) async {
      await bddSetUp(tester);
      await theActiveModeIsSetToLsb(tester);
      await iTapModeButtonUsb(tester);
      await theActiveModeShouldBeUsb(tester);
    });
    testWidgets('''Selecting LSB mode''', (tester) async {
      await bddSetUp(tester);
      await theActiveModeIsSetToUsb(tester);
      await iTapModeButtonLsb(tester);
      await theActiveModeShouldBeLsb(tester);
    });
    testWidgets('''Mode is remembered per VFO''', (tester) async {
      await bddSetUp(tester);
      await vfoAModeIsSetToLsb(tester);
      await vfoBModeIsSetToUsb(tester);
      await vfoAIsSelected(tester);
      await iTapVfoB(tester);
      await theActiveModeShouldBeUsb(tester);
      await iTapVfoA(tester);
      await theActiveModeShouldBeLsb(tester);
    });
    testWidgets('''Swapping VFOs swaps their modes''', (tester) async {
      await bddSetUp(tester);
      await vfoAModeIsSetToLsb(tester);
      await vfoBModeIsSetToUsb(tester);
      await vfoAIsSelected(tester);
      await iTapTheSwapButton(tester);
      await vfoAModeShouldBeUsb(tester);
      await vfoBModeShouldBeLsb(tester);
    });
    testWidgets('''Copying VFO copies mode''', (tester) async {
      await bddSetUp(tester);
      await vfoAModeIsSetToCw(tester);
      await vfoBModeIsSetToUsb(tester);
      await vfoAIsSelected(tester);
      await iTapTheCopyButton(tester);
      await vfoBModeShouldBeCw(tester);
    });
  });
}
