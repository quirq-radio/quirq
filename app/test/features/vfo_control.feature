Feature: VFO control

  Background:
    Given the VFO screen is shown

  Scenario: VFO A is active by default
    Then VFO 'A' is active

  Scenario: Tapping VFO B switches the active VFO
    Given VFO 'A' is selected
    When I tap VFO 'B'
    Then VFO 'B' is active

  Scenario: Swapping VFOs exchanges their frequencies
    Given VFO 'A' frequency is set to 7189500
    And VFO 'B' frequency is set to 14220000
    When I tap the swap button
    Then VFO 'A' frequency should be 14220000
    And VFO 'B' frequency should be 7189500

  Scenario: Copying VFO copies the frequency
    Given VFO 'A' frequency is set to 7189500
    And VFO 'A' is selected
    When I tap the copy button
    Then VFO 'B' frequency should be 7189500
