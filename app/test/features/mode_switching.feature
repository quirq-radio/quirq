Feature: Mode switching

  Background:
    Given the VFO screen is shown

  Scenario: Selecting USB mode
    Given the active mode is set to 'LSB'
    When I tap mode button 'USB'
    Then the active mode should be 'USB'

  Scenario: Selecting LSB mode
    Given the active mode is set to 'USB'
    When I tap mode button 'LSB'
    Then the active mode should be 'LSB'

  Scenario: Mode is remembered per VFO
    Given VFO 'A' mode is set to 'LSB'
    And VFO 'B' mode is set to 'USB'
    And VFO 'A' is selected
    When I tap VFO 'B'
    Then the active mode should be 'USB'
    When I tap VFO 'A'
    Then the active mode should be 'LSB'

  Scenario: Swapping VFOs swaps their modes
    Given VFO 'A' mode is set to 'LSB'
    And VFO 'B' mode is set to 'USB'
    And VFO 'A' is selected
    When I tap the swap button
    Then VFO 'A' mode should be 'USB'
    And VFO 'B' mode should be 'LSB'

  Scenario: Copying VFO copies mode
    Given VFO 'A' mode is set to 'CW'
    And VFO 'B' mode is set to 'USB'
    And VFO 'A' is selected
    When I tap the copy button
    Then VFO 'B' mode should be 'CW'
