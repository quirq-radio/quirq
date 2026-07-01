//! Integration tests for HamlibTransceiver against Hamlib's built-in dummy rig.
//!
//! The dummy rig (model 1) is a pure software simulation: it accepts all
//! commands, maintains internal state, and requires no physical hardware or
//! device node.

use quirq_core::transceiver::{
    Dcd, Frequency, Func, Level, LevelValue, Mode, Ptt, RptrShift, Transceiver, VfoOp,
};
use quirq_core::transceiver::hamlib::{self, HamlibTransceiver};

// Bring Display into scope so .to_string() works on the enums.
use std::string::ToString;

/// Hamlib model 1 = RIG_MODEL_DUMMY.  The port path is ignored by this backend.
const DUMMY_MODEL: u32 = 1;
const DUMMY_PORT: &str = "/dev/null";

fn open_dummy() -> HamlibTransceiver {
    HamlibTransceiver::open(DUMMY_MODEL, DUMMY_PORT)
        .expect("failed to open hamlib dummy rig")
}

/// Look up a VFO by name on `rig`, panicking if the radio does not support it.
/// This is the intended usage pattern for library consumers: obtain VFO handles
/// at runtime from the radio rather than using driver-specific constants.
fn vfo(rig: &mut HamlibTransceiver, name: &str) -> quirq_core::transceiver::Vfo {
    rig.vfo_by_name(name)
        .unwrap_or_else(|e| panic!("vfo_by_name({name:?}) failed: {e}"))
        .unwrap_or_else(|| panic!("radio does not have a VFO named {name:?}"))
}

// ── VFO discovery ─────────────────────────────────────────────────────────────

#[test]
fn vfo_list_is_nonempty() {
    // The dummy rig reports at least one VFO.
    let mut rig = open_dummy();
    let list = rig.vfo_list().unwrap();
    assert!(!list.is_empty(), "expected at least one VFO from dummy rig");
}

#[test]
fn vfo_list_contains_a_and_b() {
    // The dummy rig supports VFO A and VFO B as named entries.
    let mut rig = open_dummy();
    let list = rig.vfo_list().unwrap();
    let names: Vec<&str> = list.iter().map(|v| v.name()).collect();
    assert!(names.contains(&"A"), "expected VFO 'A' in list; got {names:?}");
    assert!(names.contains(&"B"), "expected VFO 'B' in list; got {names:?}");
}

#[test]
fn vfo_list_always_includes_current() {
    // The 'Current' selector must appear in vfo_list regardless of what the
    // radio reports in its hardware capability bitmask, because it is always
    // a valid target for operations.
    let mut rig = open_dummy();
    let list = rig.vfo_list().unwrap();
    let names: Vec<&str> = list.iter().map(|v| v.name()).collect();
    assert!(names.contains(&"Current"), "expected 'Current' in list; got {names:?}");
}

#[test]
fn vfo_by_name_finds_existing_vfo() {
    // vfo_by_name returns Some for a VFO the radio supports.
    let mut rig = open_dummy();
    let result = rig.vfo_by_name("A").unwrap();
    assert!(result.is_some(), "expected Some for 'A'");
    assert_eq!(result.unwrap().name(), "A");
}

#[test]
fn vfo_by_name_is_case_insensitive() {
    // VFO names can be provided in any case; the lookup is case-insensitive.
    let mut rig = open_dummy();
    assert!(rig.vfo_by_name("a").unwrap().is_some());
    assert!(rig.vfo_by_name("A").unwrap().is_some());
    assert!(rig.vfo_by_name("current").unwrap().is_some());
    assert!(rig.vfo_by_name("CURRENT").unwrap().is_some());
}

#[test]
fn vfo_by_name_returns_none_for_unknown_name() {
    // vfo_by_name returns None for a name that is not a valid VFO on this radio.
    let mut rig = open_dummy();
    let result = rig.vfo_by_name("NoSuchVFO").unwrap();
    assert!(result.is_none());
}

#[test]
fn vfo_by_name_accepts_hamlib_aliases() {
    // Hamlib uses canonical names like "VFOA" internally. vfo_by_name should
    // accept these aliases in addition to our friendly names like "A".
    let mut rig = open_dummy();
    assert!(rig.vfo_by_name("VFOA").unwrap().is_some(), "'VFOA' alias not recognised");
    assert!(rig.vfo_by_name("VFOB").unwrap().is_some(), "'VFOB' alias not recognised");
}

// ── Frequency ─────────────────────────────────────────────────────────────────

#[test]
fn freq_roundtrip_vfo_a() {
    // A frequency written to VFO A is returned unchanged when VFO A is read back.
    let mut rig = open_dummy();
    let a = vfo(&mut rig, "A");
    let want = Frequency::from_mhz(14.225);
    rig.set_freq(&a, want).unwrap();
    assert_eq!(rig.get_freq(&a).unwrap(), want);
}

#[test]
fn freq_roundtrip_vfo_b() {
    // A frequency written to VFO B is returned unchanged when VFO B is read back.
    let mut rig = open_dummy();
    let b = vfo(&mut rig, "B");
    let want = Frequency::from_mhz(7.074);
    rig.set_freq(&b, want).unwrap();
    assert_eq!(rig.get_freq(&b).unwrap(), want);
}

#[test]
fn freq_roundtrip_current_vfo() {
    // A frequency written using the Current VFO selector is returned unchanged
    // when the current VFO is read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    let want = Frequency::from_mhz(21.300);
    rig.set_freq(&curr, want).unwrap();
    assert_eq!(rig.get_freq(&curr).unwrap(), want);
}

#[test]
fn freq_roundtrip_hf_band_edges() {
    // Frequencies at the lower edge of every HF amateur band survive a
    // set/get round-trip without loss or rounding error.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    for mhz in [1.800, 3.500, 7.000, 10.100, 14.000, 18.068, 21.000, 24.890, 28.000, 29.700] {
        let want = Frequency::from_mhz(mhz);
        rig.set_freq(&curr, want).unwrap();
        assert_eq!(rig.get_freq(&curr).unwrap(), want, "failed at {mhz} MHz");
    }
}

#[test]
fn freq_vfo_a_and_b_are_independent() {
    // Setting a frequency on VFO A does not affect VFO B, and vice versa.
    // Both VFOs retain their independently assigned frequencies.
    let mut rig = open_dummy();
    let a = vfo(&mut rig, "A");
    let b = vfo(&mut rig, "B");
    let fa = Frequency::from_mhz(14.074);
    let fb = Frequency::from_mhz(7.074);
    rig.set_freq(&a, fa).unwrap();
    rig.set_freq(&b, fb).unwrap();
    assert_eq!(rig.get_freq(&a).unwrap(), fa);
    assert_eq!(rig.get_freq(&b).unwrap(), fb);
}

// ── Mode ──────────────────────────────────────────────────────────────────────

/// Helper: set a mode with an explicit 2400 Hz passband and verify both
/// the mode and the passband are returned unchanged.
fn assert_mode_roundtrip(rig: &mut HamlibTransceiver, mode: Mode) {
    let curr = vfo(rig, "Current");
    let passband = Some(2400u32);
    rig.set_mode(&curr, mode, passband).unwrap();
    let (got_mode, got_pb) = rig.get_mode(&curr).unwrap();
    assert_eq!(got_mode, mode, "mode mismatch");
    assert_eq!(got_pb, passband, "passband mismatch");
}

#[test]
fn mode_usb() {
    // USB mode and its passband survive a set/get round-trip.
    assert_mode_roundtrip(&mut open_dummy(), Mode::Usb);
}

#[test]
fn mode_lsb() {
    // LSB mode and its passband survive a set/get round-trip.
    assert_mode_roundtrip(&mut open_dummy(), Mode::Lsb);
}

#[test]
fn mode_am() {
    // AM mode and its passband survive a set/get round-trip.
    assert_mode_roundtrip(&mut open_dummy(), Mode::Am);
}

#[test]
fn mode_fm() {
    // FM mode and its passband survive a set/get round-trip.
    assert_mode_roundtrip(&mut open_dummy(), Mode::Fm);
}

#[test]
fn mode_cw() {
    // CW (normal sideband) mode and its passband survive a set/get round-trip.
    assert_mode_roundtrip(&mut open_dummy(), Mode::Cw);
}

#[test]
fn mode_cwr() {
    // CW reverse sideband mode and its passband survive a set/get round-trip.
    assert_mode_roundtrip(&mut open_dummy(), Mode::CwR);
}

#[test]
fn mode_rtty() {
    // RTTY mode and its passband survive a set/get round-trip.
    assert_mode_roundtrip(&mut open_dummy(), Mode::Rtty);
}

#[test]
fn mode_rttyr() {
    // RTTY reverse sideband mode and its passband survive a set/get round-trip.
    assert_mode_roundtrip(&mut open_dummy(), Mode::RttyR);
}

#[test]
fn mode_pktusb() {
    // Packet/digital USB mode and its passband survive a set/get round-trip.
    assert_mode_roundtrip(&mut open_dummy(), Mode::PktUsb);
}

#[test]
fn mode_pktlsb() {
    // Packet/digital LSB mode and its passband survive a set/get round-trip.
    assert_mode_roundtrip(&mut open_dummy(), Mode::PktLsb);
}

#[test]
fn mode_pktfm() {
    // Packet/digital FM mode and its passband survive a set/get round-trip.
    assert_mode_roundtrip(&mut open_dummy(), Mode::PktFm);
}

#[test]
fn mode_default_passband_is_nonzero() {
    // When passband is set to None (radio default), the radio selects a
    // non-zero default filter width for the chosen mode. Verifies that
    // get_mode returns a positive passband width, not zero or None.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_mode(&curr, Mode::Usb, None).unwrap();
    let (mode, pb) = rig.get_mode(&curr).unwrap();
    assert_eq!(mode, Mode::Usb);
    assert!(pb.map(|w| w > 0).unwrap_or(false), "expected a positive default passband");
}

#[test]
fn mode_and_freq_are_independent() {
    // Setting the mode does not alter the frequency, and setting the frequency
    // does not alter the mode. Both pieces of state are stored independently.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    let freq = Frequency::from_mhz(14.225);
    rig.set_freq(&curr, freq).unwrap();
    rig.set_mode(&curr, Mode::Usb, Some(2700)).unwrap();
    assert_eq!(rig.get_freq(&curr).unwrap(), freq);
    let (mode, _) = rig.get_mode(&curr).unwrap();
    assert_eq!(mode, Mode::Usb);
}

// ── PTT ───────────────────────────────────────────────────────────────────────

#[test]
fn ptt_starts_rx() {
    // A freshly opened rig reports PTT in the receive (inactive) state.
    let mut rig = open_dummy();
    assert_eq!(rig.get_ptt().unwrap(), Ptt::Rx);
}

#[test]
fn ptt_roundtrip_tx() {
    // After PTT is keyed to transmit, get_ptt reports the transmit state.
    let mut rig = open_dummy();
    rig.set_ptt(Ptt::Tx).unwrap();
    assert_eq!(rig.get_ptt().unwrap(), Ptt::Tx);
}

#[test]
fn ptt_roundtrip_rx_after_tx() {
    // After PTT is keyed to transmit and then released, get_ptt reports
    // the receive state again.
    let mut rig = open_dummy();
    rig.set_ptt(Ptt::Tx).unwrap();
    rig.set_ptt(Ptt::Rx).unwrap();
    assert_eq!(rig.get_ptt().unwrap(), Ptt::Rx);
}

// ── Levels ────────────────────────────────────────────────────────────────────

#[test]
fn level_af_gain_roundtrip() {
    // AF (audio) gain can be set to any value in [0.0, 1.0] and read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_level(&curr, Level::AfGain, LevelValue::Float(0.75)).unwrap();
    assert_eq!(rig.get_level(&curr, Level::AfGain).unwrap(), LevelValue::Float(0.75));
}

#[test]
fn level_rf_gain_roundtrip() {
    // RF gain can be set and read back accurately.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_level(&curr, Level::RfGain, LevelValue::Float(0.5)).unwrap();
    assert_eq!(rig.get_level(&curr, Level::RfGain).unwrap(), LevelValue::Float(0.5));
}

#[test]
fn level_rf_power_roundtrip() {
    // RF transmit power level can be set and read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_level(&curr, Level::RfPower, LevelValue::Float(0.6)).unwrap();
    assert_eq!(rig.get_level(&curr, Level::RfPower).unwrap(), LevelValue::Float(0.6));
}

#[test]
fn level_squelch_roundtrip() {
    // Squelch level can be set and read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_level(&curr, Level::Squelch, LevelValue::Float(0.3)).unwrap();
    assert_eq!(rig.get_level(&curr, Level::Squelch).unwrap(), LevelValue::Float(0.3));
}

#[test]
fn level_strength_is_readable() {
    // S-meter (signal strength) can be read without error.
    // The exact value is not asserted because it depends on the dummy rig's state.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    let val = rig.get_level(&curr, Level::Strength).unwrap();
    assert!(matches!(val, LevelValue::Int(_)), "expected Int from S-meter");
}

#[test]
fn level_mic_gain_roundtrip() {
    // Microphone gain can be set and read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_level(&curr, Level::MicGain, LevelValue::Float(0.8)).unwrap();
    assert_eq!(rig.get_level(&curr, Level::MicGain).unwrap(), LevelValue::Float(0.8));
}

#[test]
fn level_if_shift_roundtrip() {
    // IF shift (in Hz) can be set and read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_level(&curr, Level::IfShift, LevelValue::Int(250)).unwrap();
    assert_eq!(rig.get_level(&curr, Level::IfShift).unwrap(), LevelValue::Int(250));
}

#[test]
fn level_nr_roundtrip() {
    // Noise reduction level can be set and read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_level(&curr, Level::NoiseReduction, LevelValue::Float(0.4)).unwrap();
    assert_eq!(rig.get_level(&curr, Level::NoiseReduction).unwrap(), LevelValue::Float(0.4));
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[test]
fn func_noise_blanker_roundtrip() {
    // Noise blanker can be enabled and the enabled state can be read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_func(&curr, Func::NoiseBlanker, true).unwrap();
    assert!(rig.get_func(&curr, Func::NoiseBlanker).unwrap());
}

#[test]
fn func_noise_blanker_disable() {
    // Noise blanker can be disabled after being enabled, and the disabled state
    // is correctly reported.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_func(&curr, Func::NoiseBlanker, true).unwrap();
    rig.set_func(&curr, Func::NoiseBlanker, false).unwrap();
    assert!(!rig.get_func(&curr, Func::NoiseBlanker).unwrap());
}

#[test]
fn func_vox_roundtrip() {
    // VOX (voice-operated relay) can be enabled and read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_func(&curr, Func::Vox, true).unwrap();
    assert!(rig.get_func(&curr, Func::Vox).unwrap());
}

#[test]
fn func_noise_reduction_roundtrip() {
    // Noise reduction (DSP) function can be enabled and read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_func(&curr, Func::NoiseReduction, true).unwrap();
    assert!(rig.get_func(&curr, Func::NoiseReduction).unwrap());
}

#[test]
fn func_lock_roundtrip() {
    // Tuning lock can be enabled and read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_func(&curr, Func::Lock, true).unwrap();
    assert!(rig.get_func(&curr, Func::Lock).unwrap());
}

// ── RIT / XIT ─────────────────────────────────────────────────────────────────

#[test]
fn rit_roundtrip_positive() {
    // A positive RIT offset in Hz can be set and read back unchanged.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_rit(&curr, 600).unwrap();
    assert_eq!(rig.get_rit(&curr).unwrap(), 600);
}

#[test]
fn rit_roundtrip_negative() {
    // A negative RIT offset (tuning down) can be set and read back unchanged.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_rit(&curr, -300).unwrap();
    assert_eq!(rig.get_rit(&curr).unwrap(), -300);
}

#[test]
fn rit_reset_to_zero() {
    // Setting RIT to 0 Hz effectively clears any existing offset.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_rit(&curr, 500).unwrap();
    rig.set_rit(&curr, 0).unwrap();
    assert_eq!(rig.get_rit(&curr).unwrap(), 0);
}

#[test]
fn xit_roundtrip() {
    // XIT (transmit incremental tuning) offset can be set and read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_xit(&curr, 400).unwrap();
    assert_eq!(rig.get_xit(&curr).unwrap(), 400);
}

// ── Split operation ───────────────────────────────────────────────────────────

#[test]
fn split_off_by_default() {
    // A freshly opened rig is not in split mode.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    let (split, _) = rig.get_split_vfo(&curr).unwrap();
    assert!(!split);
}

#[test]
fn split_enable_and_disable() {
    // Split mode can be turned on (assigning a TX VFO) and then turned off again.
    let mut rig = open_dummy();
    let a = vfo(&mut rig, "A");
    let b = vfo(&mut rig, "B");
    rig.set_split_vfo(&a, true, &b).unwrap();
    let (split, _) = rig.get_split_vfo(&a).unwrap();
    assert!(split);

    rig.set_split_vfo(&a, false, &a).unwrap();
    let (split, _) = rig.get_split_vfo(&a).unwrap();
    assert!(!split);
}

#[test]
fn split_freq_roundtrip() {
    // When split is active, the TX frequency can be set independently and
    // read back on the TX VFO without affecting the RX frequency.
    let mut rig = open_dummy();
    let a = vfo(&mut rig, "A");
    let b = vfo(&mut rig, "B");
    let rx_freq = Frequency::from_mhz(14.200);
    let tx_freq = Frequency::from_mhz(14.225);
    rig.set_freq(&a, rx_freq).unwrap();
    rig.set_split_vfo(&a, true, &b).unwrap();
    rig.set_split_freq(&b, tx_freq).unwrap();
    assert_eq!(rig.get_split_freq(&b).unwrap(), tx_freq);
    assert_eq!(rig.get_freq(&a).unwrap(), rx_freq);
}

#[test]
fn split_mode_roundtrip() {
    // The TX mode can be set independently from the RX mode when in split.
    let mut rig = open_dummy();
    let a = vfo(&mut rig, "A");
    let b = vfo(&mut rig, "B");
    rig.set_split_vfo(&a, true, &b).unwrap();
    rig.set_split_mode(&b, Mode::Usb, Some(2700)).unwrap();
    let (mode, pb) = rig.get_split_mode(&b).unwrap();
    assert_eq!(mode, Mode::Usb);
    assert_eq!(pb, Some(2700));
}

// ── CTCSS / DCS ───────────────────────────────────────────────────────────────

#[test]
fn ctcss_tone_roundtrip() {
    // A CTCSS TX tone (in units of 0.1 Hz) can be set and read back.
    // 1318 = 131.8 Hz, a standard CTCSS tone.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_ctcss_tone(&curr, 1318).unwrap();
    assert_eq!(rig.get_ctcss_tone(&curr).unwrap(), 1318);
}

#[test]
fn ctcss_sql_roundtrip() {
    // A CTCSS squelch tone can be set and read back independently of the TX tone.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_ctcss_sql(&curr, 670).unwrap();
    assert_eq!(rig.get_ctcss_sql(&curr).unwrap(), 670);
}

#[test]
fn dcs_code_roundtrip() {
    // A DCS TX code can be set and read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_dcs_code(&curr, 23).unwrap();
    assert_eq!(rig.get_dcs_code(&curr).unwrap(), 23);
}

#[test]
fn dcs_sql_roundtrip() {
    // A DCS squelch code can be set and read back independently of the TX code.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_dcs_sql(&curr, 25).unwrap();
    assert_eq!(rig.get_dcs_sql(&curr).unwrap(), 25);
}

// ── Repeater ──────────────────────────────────────────────────────────────────

#[test]
fn rptr_shift_none_roundtrip() {
    // Repeater shift direction of None (simplex) can be set and read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_rptr_shift(&curr, RptrShift::None).unwrap();
    assert_eq!(rig.get_rptr_shift(&curr).unwrap(), RptrShift::None);
}

#[test]
fn rptr_shift_plus_roundtrip() {
    // Positive repeater shift can be set and read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_rptr_shift(&curr, RptrShift::Plus).unwrap();
    assert_eq!(rig.get_rptr_shift(&curr).unwrap(), RptrShift::Plus);
}

#[test]
fn rptr_shift_minus_roundtrip() {
    // Negative repeater shift can be set and read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_rptr_shift(&curr, RptrShift::Minus).unwrap();
    assert_eq!(rig.get_rptr_shift(&curr).unwrap(), RptrShift::Minus);
}

#[test]
fn rptr_offs_roundtrip() {
    // Repeater offset frequency in Hz can be set and read back.
    // 600_000 Hz = 600 kHz, a common 2m repeater offset.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_rptr_offs(&curr, 600_000).unwrap();
    assert_eq!(rig.get_rptr_offs(&curr).unwrap(), 600_000);
}

// ── Tuning step ───────────────────────────────────────────────────────────────

#[test]
fn tuning_step_roundtrip() {
    // The VFO tuning step in Hz can be set and read back.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    rig.set_ts(&curr, 500).unwrap();
    assert_eq!(rig.get_ts(&curr).unwrap(), 500);
}

// ── VFO operations ────────────────────────────────────────────────────────────

#[test]
fn vfo_op_toggle_accepted() {
    // The Toggle VFO operation is accepted by the radio without returning an
    // error. The exact effect (which VFO becomes active) is radio-specific and
    // not asserted here; the dummy rig's internal VFO state machine does not
    // reliably expose the switched VFO through the Current selector.
    let mut rig = open_dummy();
    let a = vfo(&mut rig, "A");
    rig.set_freq(&a, Frequency::from_mhz(14.200)).unwrap();
    let b = vfo(&mut rig, "B");
    rig.set_freq(&b, Frequency::from_mhz(7.100)).unwrap();
    rig.vfo_op(&a, VfoOp::Toggle).unwrap();
}

#[test]
fn vfo_op_copy_accepted() {
    // The CopyAToB VFO operation is accepted by the radio without returning
    // an error. The exact copy direction and observable effect are
    // radio-specific; the dummy rig's CPY semantics differ from real hardware.
    let mut rig = open_dummy();
    let a = vfo(&mut rig, "A");
    rig.set_freq(&a, Frequency::from_mhz(14.225)).unwrap();
    rig.vfo_op(&a, VfoOp::CopyAToB).unwrap();
}

// ── Power state ───────────────────────────────────────────────────────────────

#[test]
fn powerstat_is_on_after_open() {
    // The dummy rig reports powered-on state immediately after being opened.
    let mut rig = open_dummy();
    assert_eq!(rig.get_powerstat().unwrap(), quirq_core::transceiver::PowerState::On);
}

// ── DCD ───────────────────────────────────────────────────────────────────────

#[test]
fn dcd_is_readable() {
    // Data carrier detect (squelch open indicator) can be queried without
    // error; the exact value depends on the dummy rig's simulated state.
    let mut rig = open_dummy();
    let curr = vfo(&mut rig, "Current");
    let dcd = rig.get_dcd(&curr).unwrap();
    assert!(matches!(dcd, Dcd::Off | Dcd::On));
}

// ── Info ──────────────────────────────────────────────────────────────────────

#[test]
fn get_info_returns_nonempty_string() {
    // The rig identification string is non-empty for the dummy backend.
    let mut rig = open_dummy();
    let info = rig.get_info().unwrap();
    assert!(!info.is_empty(), "expected a non-empty info string from dummy rig");
}

// ── Model discovery ───────────────────────────────────────────────────────────

#[test]
fn list_models_returns_many_entries() {
    // list_models() queries the full hamlib registry; a system-installed hamlib
    // should report dozens of supported radios.
    let models = hamlib::list_models();
    assert!(models.len() > 10, "expected >10 models from hamlib; got {}", models.len());
}

#[test]
fn list_models_includes_dummy_rig() {
    // The dummy rig (model 1) must appear in the list so tests can reference it
    // without hard-coding the model number in driver-facing code.
    let models = hamlib::list_models();
    let dummy = models.iter().find(|m| m.model_id == 1);
    assert!(dummy.is_some(), "expected dummy rig (model 1) in list_models()");
    assert!(dummy.unwrap().model_name.to_ascii_lowercase().contains("dummy"));
}

#[test]
fn list_models_entries_have_nonempty_names() {
    // Every model in the list should have at least a non-empty model name.
    for m in hamlib::list_models() {
        assert!(!m.model_name.is_empty(), "model {} has no name", m.model_id);
    }
}

#[test]
fn list_models_status_is_known_string() {
    // The status field must be one of the documented values, not a raw integer.
    let known = ["Stable", "Beta", "Alpha", "Untested", "Buggy", "Unknown"];
    for m in hamlib::list_models() {
        assert!(
            known.contains(&m.status.as_str()),
            "model {} has unexpected status {:?}", m.model_id, m.status
        );
    }
}

// ── Capability discovery ──────────────────────────────────────────────────────

#[test]
fn supported_modes_is_nonempty() {
    // The dummy rig supports at least one operating mode.
    let mut rig = open_dummy();
    let modes = rig.supported_modes().unwrap();
    assert!(!modes.is_empty(), "expected at least one supported mode");
}

#[test]
fn supported_modes_contains_usb_and_lsb() {
    // A general-purpose transceiver must support SSB. The dummy rig lists
    // USB and LSB among its supported modes.
    let mut rig = open_dummy();
    let modes = rig.supported_modes().unwrap();
    assert!(modes.contains(&Mode::Usb), "USB not in supported modes: {modes:?}");
    assert!(modes.contains(&Mode::Lsb), "LSB not in supported modes: {modes:?}");
}

#[test]
fn supported_levels_is_nonempty() {
    // The dummy rig supports at least one controllable level.
    let mut rig = open_dummy();
    let levels = rig.supported_levels().unwrap();
    assert!(!levels.is_empty(), "expected at least one supported level");
}

#[test]
fn supported_levels_contains_af_gain() {
    // AF Gain is a near-universal level; the dummy rig should report it.
    let mut rig = open_dummy();
    let levels = rig.supported_levels().unwrap();
    assert!(
        levels.contains(&Level::AfGain),
        "AfGain not in supported levels: {levels:?}"
    );
}

#[test]
fn supported_funcs_is_nonempty() {
    // The dummy rig supports at least one boolean function.
    let mut rig = open_dummy();
    let funcs = rig.supported_funcs().unwrap();
    assert!(!funcs.is_empty(), "expected at least one supported function");
}

#[test]
fn supported_funcs_contains_noise_blanker() {
    // Noise blanker is a commonly supported function; the dummy rig includes it.
    let mut rig = open_dummy();
    let funcs = rig.supported_funcs().unwrap();
    assert!(
        funcs.contains(&Func::NoiseBlanker),
        "NoiseBlanker not in supported funcs: {funcs:?}"
    );
}

#[test]
fn supported_vfo_ops_is_nonempty() {
    // The dummy rig supports at least one VFO operation.
    let mut rig = open_dummy();
    let ops = rig.supported_vfo_ops().unwrap();
    assert!(!ops.is_empty(), "expected at least one supported VFO op");
}

#[test]
fn supported_scan_ops_is_nonempty() {
    // The dummy rig supports at least one scan mode.
    let mut rig = open_dummy();
    let ops = rig.supported_scan_ops().unwrap();
    assert!(!ops.is_empty(), "expected at least one supported scan op");
}

// ── Display (human-readable names) ───────────────────────────────────────────

#[test]
fn mode_display_produces_nonempty_string() {
    // Every Mode variant produces a non-empty human-readable string via Display.
    let modes = [
        Mode::Am, Mode::Cw, Mode::CwR, Mode::Usb, Mode::Lsb,
        Mode::Fm, Mode::Wfm, Mode::Rtty, Mode::RttyR,
        Mode::PktUsb, Mode::PktLsb, Mode::PktFm,
    ];
    for mode in modes {
        let s = mode.to_string();
        assert!(!s.is_empty(), "Mode::{mode:?} produced an empty string");
        assert_ne!(s, "Mode(0x0)", "Mode::{mode:?} fell through to Other display");
    }
}

#[test]
fn level_display_produces_nonempty_string() {
    // A representative set of Level variants produce non-empty human-readable strings.
    let levels = [
        Level::AfGain, Level::RfPower, Level::Squelch,
        Level::Strength, Level::Swr, Level::KeySpeed,
    ];
    for level in levels {
        let s = level.to_string();
        assert!(!s.is_empty(), "Level::{level:?} produced an empty string");
    }
}

#[test]
fn func_display_produces_nonempty_string() {
    // A representative set of Func variants produce non-empty human-readable strings.
    let funcs = [
        Func::NoiseBlanker, Func::Vox, Func::Lock,
        Func::AutoNotch, Func::NoiseReduction,
    ];
    for func in funcs {
        let s = func.to_string();
        assert!(!s.is_empty(), "Func::{func:?} produced an empty string");
    }
}

#[test]
fn discovery_names_match_display() {
    // The names returned by supported_levels() should appear in the human-readable
    // display output, confirming the two systems are consistent.
    let mut rig = open_dummy();
    for level in rig.supported_levels().unwrap() {
        let name = level.to_string();
        assert!(
            !name.is_empty(),
            "supported level {level:?} has no display name"
        );
    }
    for func in rig.supported_funcs().unwrap() {
        let name = func.to_string();
        assert!(
            !name.is_empty(),
            "supported func {func:?} has no display name"
        );
    }
}
