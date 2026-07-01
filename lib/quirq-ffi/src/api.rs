//! Public API surface exposed to Flutter via flutter_rust_bridge.
//!
//! All VFO parameters are `String` names (e.g. `"A"`, `"B"`, `"Current"`),
//! resolved at call time via `vfo_by_name`.  Mode, Level, and Func values are
//! also `String`; use the corresponding `supported_*` discovery methods to
//! obtain the set of valid names for the connected radio.

use flutter_rust_bridge::frb;
use std::sync::{Mutex, MutexGuard};

use anyhow::{anyhow, Context};
use quirq_core::transceiver::{
    self as core, Frequency, Func, Level, Mode, Parm, PowerState, Ptt, RptrShift,
    Transceiver, VfoOp,
};
use quirq_core::transceiver::hamlib::{self, HamlibTransceiver};

// ── KeyValue ──────────────────────────────────────────────────────────────────

/// Initialise flutter_rust_bridge. Called automatically by generated Dart code.
#[frb(init)]
pub fn frb_init() {
    flutter_rust_bridge::setup_default_user_utils();
}

/// A key-value pair used for driver configuration in `RigHandle::open`.
/// FRB v2 does not support Rust tuples across the FFI boundary; this struct
/// is the equivalent.
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

// ── Model discovery ───────────────────────────────────────────────────────────

/// Describes a single radio model available through a particular driver.
///
/// `driver` and `driver_model_id` together identify the model in a way that
/// can be passed directly back to `RigHandle::open()` — Dart never needs to
/// parse or understand either value.
pub struct RadioModel {
    /// The driver that supports this model (e.g. `"hamlib"`).
    pub driver: String,
    /// Opaque driver-internal model identifier.  Pass this as the `"model"`
    /// entry in the `params` argument to `RigHandle::open()`.
    pub driver_model_id: String,
    pub manufacturer: String,
    pub model: String,
    pub version: String,
    /// Human-readable status: `"Stable"`, `"Beta"`, `"Alpha"`, `"Untested"`, `"Buggy"`.
    pub status: String,
}

/// Returns all radio models available through `driver`.
///
/// The returned list can be presented directly in a radio-selection UI.
/// When the user picks a model, pass `model.driver` and
/// `[("model", model.driver_model_id), ("port", ...)]` to `RigHandle::open()`.
pub fn list_models(driver: String) -> anyhow::Result<Vec<RadioModel>> {
    match driver.as_str() {
        "hamlib" => Ok(hamlib::list_models()
            .into_iter()
            .map(|m| RadioModel {
                driver: "hamlib".into(),
                driver_model_id: m.model_id.to_string(),
                manufacturer: m.manufacturer,
                model: m.model_name,
                version: m.version,
                status: m.status,
            })
            .collect()),
        d => Err(anyhow!("unknown driver {d:?}")),
    }
}

// ── LevelValue ────────────────────────────────────────────────────────────────

/// Value for a radio level — either a normalised float or a signed integer.
/// FRB generates a Dart sealed class with two subclasses from this enum.
pub enum LevelValue {
    Float(f32),
    Int(i32),
}

impl From<core::LevelValue> for LevelValue {
    fn from(v: core::LevelValue) -> Self {
        match v {
            core::LevelValue::Float(f) => LevelValue::Float(f),
            core::LevelValue::Int(i) => LevelValue::Int(i),
        }
    }
}

impl From<LevelValue> for core::LevelValue {
    fn from(v: LevelValue) -> Self {
        match v {
            LevelValue::Float(f) => core::LevelValue::Float(f),
            LevelValue::Int(i) => core::LevelValue::Int(i),
        }
    }
}

// ── Parse helpers ─────────────────────────────────────────────────────────────
// Each function matches the string against the Display output of every known
// variant, so the round-trip `parse_mode(mode.to_string())` always succeeds.

fn parse_mode(s: &str) -> anyhow::Result<Mode> {
    const ALL: &[Mode] = &[
        Mode::Am, Mode::Cw, Mode::CwR, Mode::Usb, Mode::Lsb,
        Mode::Fm, Mode::Wfm, Mode::Rtty, Mode::RttyR,
        Mode::PktUsb, Mode::PktLsb, Mode::PktFm,
    ];
    ALL.iter()
        .find(|m| m.to_string().eq_ignore_ascii_case(s))
        .copied()
        .ok_or_else(|| anyhow!("unknown mode {s:?}"))
}

fn parse_level(s: &str) -> anyhow::Result<Level> {
    const ALL: &[Level] = &[
        Level::AfGain, Level::RfGain, Level::Squelch, Level::AudioPeakFilter,
        Level::NoiseReduction, Level::PbtIn, Level::PbtOut, Level::RfPower,
        Level::MicGain, Level::Comp, Level::VoxGain, Level::AntiVox,
        Level::Balance, Level::MonitorGain, Level::NoiseBlanker, Level::NotchFreqRaw,
        Level::Preamp, Level::Attenuation, Level::VoxDelay, Level::IfShift,
        Level::CwPitch, Level::KeySpeed, Level::NotchFreq, Level::Agc,
        Level::BkInDelay, Level::BkInDelayMs, Level::SlopeLow, Level::SlopeHigh,
        Level::Strength, Level::Swr, Level::Alc, Level::RfPowerMeter,
        Level::RfPowerWatts, Level::CompMeter, Level::VdMeter, Level::IdMeter,
        Level::TempMeter,
    ];
    ALL.iter()
        .find(|l| l.to_string().eq_ignore_ascii_case(s))
        .copied()
        .ok_or_else(|| anyhow!("unknown level {s:?}"))
}

fn parse_func(s: &str) -> anyhow::Result<Func> {
    const ALL: &[Func] = &[
        Func::FastAgc, Func::NoiseBlanker, Func::Comp, Func::Vox,
        Func::CtcssTone, Func::CtcssSql, Func::SemiBreakIn, Func::FullBreakIn,
        Func::AutoNotch, Func::NoiseReduction, Func::Aip, Func::AutoPeakFilter,
        Func::Monitor, Func::ManualNotch, Func::RttyFilter, Func::AutoRptrOffset,
        Func::Lock, Func::Mute, Func::VoiceScan, Func::Reverse, Func::Squelch,
        Func::AutoBandMode, Func::BeatCancel, Func::ManualBeatCancel, Func::Rit,
        Func::Afc, Func::SatMode, Func::Scope, Func::ScanResume, Func::ToneBurst,
        Func::Tuner, Func::Xit, Func::NoiseBlanker2, Func::DcsSql, Func::AfFilter,
        Func::NoiseLimiter, Func::BeatCancel2, Func::DualWatch, Func::Diversity,
    ];
    ALL.iter()
        .find(|f| f.to_string().eq_ignore_ascii_case(s))
        .copied()
        .ok_or_else(|| anyhow!("unknown function {s:?}"))
}

fn parse_vfo_op(s: &str) -> anyhow::Result<VfoOp> {
    const ALL: &[VfoOp] = &[
        VfoOp::CopyAToB, VfoOp::Exchange, VfoOp::ToMemory, VfoOp::FromMemory,
        VfoOp::MemClear, VfoOp::StepUp, VfoOp::StepDown, VfoOp::BandUp,
        VfoOp::BandDown, VfoOp::TuneLeft, VfoOp::TuneRight, VfoOp::AutoTune,
        VfoOp::Toggle,
    ];
    ALL.iter()
        .find(|o| o.to_string().eq_ignore_ascii_case(s))
        .copied()
        .ok_or_else(|| anyhow!("unknown VFO op {s:?}"))
}

fn parse_rptr_shift(s: &str) -> anyhow::Result<RptrShift> {
    match s.to_ascii_lowercase().as_str() {
        "simplex" | "none" => Ok(RptrShift::None),
        "minus" | "-" => Ok(RptrShift::Minus),
        "plus" | "+" => Ok(RptrShift::Plus),
        _ => Err(anyhow!("unknown repeater shift {s:?}")),
    }
}

fn parse_parm(s: &str) -> anyhow::Result<Parm> {
    const ALL: &[Parm] = &[
        Parm::Announce, Parm::AutoPowerOff, Parm::Backlight, Parm::Beep,
        Parm::Time, Parm::Battery, Parm::KeyLight, Parm::ScreenSaver, Parm::AfIf,
    ];
    ALL.iter()
        .find(|p| p.to_string().eq_ignore_ascii_case(s))
        .copied()
        .ok_or_else(|| anyhow!("unknown parameter {s:?}"))
}

fn parse_power_state(s: &str) -> anyhow::Result<PowerState> {
    match s.to_ascii_lowercase().as_str() {
        "off" => Ok(PowerState::Off),
        "on" => Ok(PowerState::On),
        "standby" => Ok(PowerState::Standby),
        "operate" => Ok(PowerState::Operate),
        _ => Err(anyhow!("unknown power state {s:?}")),
    }
}

// ── RigHandle ────────────────────────────────────────────────────────────────

/// An open connection to a radio transceiver.
///
/// Obtain an instance via [`RigHandle::open`]. All methods are thread-safe;
/// concurrent calls serialise on an internal mutex.
#[frb(opaque)]
pub struct RigHandle {
    rig: Mutex<Box<dyn Transceiver>>,
}

// SAFETY: Every access goes through the Mutex, which enforces exclusive access.
// Box<dyn Transceiver> is Send (Transceiver: Send) but not Sync; the Mutex
// provides the necessary synchronisation for shared ownership.
unsafe impl Sync for RigHandle {}

impl RigHandle {
    fn lock(&self) -> MutexGuard<'_, Box<dyn Transceiver>> {
        self.rig.lock().expect("rig mutex poisoned")
    }

    // ── Construction ──────────────────────────────────────────────────────────

    /// Open a connection to a radio transceiver.
    ///
    /// `driver` selects the backend. `params` supplies driver-specific
    /// key-value configuration. Callers never need to import driver-specific
    /// types or constants.
    ///
    /// ## Supported drivers
    ///
    /// **`"hamlib"`** — any radio supported by libhamlib.
    /// - `"model"` — hamlib model number as a decimal string (e.g. `"3085"` for IC-705).
    ///   Run `rigctl --list` to find the number for a specific radio.
    /// - `"port"` — device path or hostname (e.g. `"/dev/ttyUSB0"`, `"192.168.1.100"`).
    ///
    /// Additional drivers (e.g. `"icom-lan"`) will be added here as they are
    /// implemented. The set of recognised param keys is documented per driver.
    pub fn open(driver: String, params: Vec<KeyValue>) -> anyhow::Result<Self> {
        let get = |key: &str| -> anyhow::Result<&str> {
            params
                .iter()
                .find(|kv| kv.key == key)
                .map(|kv| kv.value.as_str())
                .ok_or_else(|| anyhow!("driver {driver:?} requires param {key:?}"))
        };

        let rig: Box<dyn Transceiver> = match driver.as_str() {
            "hamlib" => {
                let model: u32 = get("model")?
                    .parse()
                    .context("param 'model' must be a decimal integer")?;
                let port = get("port").unwrap_or("");
                Box::new(HamlibTransceiver::open(model, port)?)
            }
            d => return Err(anyhow!("unknown driver {d:?}")),
        };

        Ok(RigHandle { rig: Mutex::new(rig) })
    }

    // Convenience used in tests — not exposed to Flutter.
    #[doc(hidden)]
    pub fn open_hamlib_raw(model: u32, port: &str) -> anyhow::Result<Self> {
        let rig = HamlibTransceiver::open(model, port)?;
        Ok(RigHandle { rig: Mutex::new(Box::new(rig)) })
    }

    // ── VFO discovery ─────────────────────────────────────────────────────────

    /// Returns the VFO names supported by this radio (e.g. `["A", "B", "Current"]`).
    pub fn vfo_list(&self) -> anyhow::Result<Vec<String>> {
        let mut g = self.lock();
        Ok(g.vfo_list()?.into_iter().map(|v| v.name().to_string()).collect())
    }

    /// Returns the canonical name for `name` if this radio has such a VFO,
    /// or `None` if it does not. Accepts driver aliases (e.g. `"VFOA"` for `"A"`).
    pub fn vfo_by_name(&self, name: String) -> anyhow::Result<Option<String>> {
        let mut g = self.lock();
        Ok(g.vfo_by_name(&name)?.map(|v| v.name().to_string()))
    }

    // ── Capability discovery ──────────────────────────────────────────────────

    /// Returns the mode names this radio supports (e.g. `["USB", "LSB", "CW"]`).
    pub fn supported_modes(&self) -> anyhow::Result<Vec<String>> {
        let mut g = self.lock();
        Ok(g.supported_modes()?.into_iter().map(|m| m.to_string()).collect())
    }

    /// Returns the level names this radio supports (e.g. `["AF Gain", "RF Power"]`).
    pub fn supported_levels(&self) -> anyhow::Result<Vec<String>> {
        let mut g = self.lock();
        Ok(g.supported_levels()?.into_iter().map(|l| l.to_string()).collect())
    }

    /// Returns the function names this radio supports (e.g. `["Noise Blanker", "VOX"]`).
    pub fn supported_funcs(&self) -> anyhow::Result<Vec<String>> {
        let mut g = self.lock();
        Ok(g.supported_funcs()?.into_iter().map(|f| f.to_string()).collect())
    }

    /// Returns the VFO operation names this radio supports.
    pub fn supported_vfo_ops(&self) -> anyhow::Result<Vec<String>> {
        let mut g = self.lock();
        Ok(g.supported_vfo_ops()?.into_iter().map(|o| o.to_string()).collect())
    }

    // ── Frequency ─────────────────────────────────────────────────────────────

    /// Returns the frequency of `vfo` in Hz.
    pub fn get_freq(&self, vfo: String) -> anyhow::Result<u64> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.get_freq(&v)?.hz())
    }

    /// Sets the frequency of `vfo` to `hz` Hz.
    pub fn set_freq(&self, vfo: String, hz: u64) -> anyhow::Result<()> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.set_freq(&v, Frequency(hz))?)
    }

    // ── Mode ──────────────────────────────────────────────────────────────────

    /// Returns `(mode_name, passband_hz)` for `vfo`.
    /// `passband_hz` is `None` when the radio reports its default bandwidth.
    pub fn get_mode(&self, vfo: String) -> anyhow::Result<(String, Option<u32>)> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        let (mode, pb) = g.get_mode(&v)?;
        Ok((mode.to_string(), pb))
    }

    /// Sets the mode and optional passband width (Hz) on `vfo`.
    /// Pass `None` for `passband_hz` to use the radio's default width for the mode.
    pub fn set_mode(
        &self,
        vfo: String,
        mode: String,
        passband_hz: Option<u32>,
    ) -> anyhow::Result<()> {
        let m = parse_mode(&mode)?;
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.set_mode(&v, m, passband_hz)?)
    }

    // ── PTT ───────────────────────────────────────────────────────────────────

    /// Returns `true` if the radio is transmitting.
    pub fn get_ptt(&self) -> anyhow::Result<bool> {
        Ok(self.lock().get_ptt()? == Ptt::Tx)
    }

    /// Keys the transmitter if `tx` is `true`, releases it if `false`.
    pub fn set_ptt(&self, tx: bool) -> anyhow::Result<()> {
        Ok(self.lock().set_ptt(if tx { Ptt::Tx } else { Ptt::Rx })?)
    }

    // ── Levels ────────────────────────────────────────────────────────────────

    /// Returns the value of `level` on `vfo`.
    /// Use `supported_levels()` to obtain valid level names.
    pub fn get_level(&self, vfo: String, level: String) -> anyhow::Result<LevelValue> {
        let l = parse_level(&level)?;
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.get_level(&v, l)?.into())
    }

    /// Sets `level` on `vfo` to `value`.
    pub fn set_level(&self, vfo: String, level: String, value: LevelValue) -> anyhow::Result<()> {
        let l = parse_level(&level)?;
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.set_level(&v, l, value.into())?)
    }

    // ── Functions ─────────────────────────────────────────────────────────────

    /// Returns `true` if `func` is enabled on `vfo`.
    /// Use `supported_funcs()` to obtain valid function names.
    pub fn get_func(&self, vfo: String, func: String) -> anyhow::Result<bool> {
        let f = parse_func(&func)?;
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.get_func(&v, f)?)
    }

    /// Enables or disables `func` on `vfo`.
    pub fn set_func(&self, vfo: String, func: String, enabled: bool) -> anyhow::Result<()> {
        let f = parse_func(&func)?;
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.set_func(&v, f, enabled)?)
    }

    // ── Parameters (radio-wide) ───────────────────────────────────────────────

    /// Returns the value of a radio-wide parameter.
    pub fn get_parm(&self, parm: String) -> anyhow::Result<LevelValue> {
        Ok(self.lock().get_parm(parse_parm(&parm)?)?.into())
    }

    /// Sets a radio-wide parameter.
    pub fn set_parm(&self, parm: String, value: LevelValue) -> anyhow::Result<()> {
        Ok(self.lock().set_parm(parse_parm(&parm)?, value.into())?)
    }

    // ── RIT / XIT ─────────────────────────────────────────────────────────────

    /// Returns the RIT (Receive Incremental Tuning) offset in Hz.
    pub fn get_rit(&self, vfo: String) -> anyhow::Result<i64> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.get_rit(&v)?)
    }

    /// Sets the RIT offset in Hz (negative to tune down).
    pub fn set_rit(&self, vfo: String, hz: i64) -> anyhow::Result<()> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.set_rit(&v, hz)?)
    }

    /// Returns the XIT (Transmit Incremental Tuning) offset in Hz.
    pub fn get_xit(&self, vfo: String) -> anyhow::Result<i64> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.get_xit(&v)?)
    }

    /// Sets the XIT offset in Hz.
    pub fn set_xit(&self, vfo: String, hz: i64) -> anyhow::Result<()> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.set_xit(&v, hz)?)
    }

    // ── Split ─────────────────────────────────────────────────────────────────

    /// Returns `(split_enabled, tx_vfo_name)` for `rx_vfo`.
    pub fn get_split_vfo(&self, rx_vfo: String) -> anyhow::Result<(bool, String)> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&rx_vfo)?.ok_or_else(|| anyhow!("VFO {rx_vfo:?} not found"))?;
        let (split, tx) = g.get_split_vfo(&v)?;
        Ok((split, tx.name().to_string()))
    }

    /// Enables or disables split, assigning `tx_vfo` as the transmit VFO.
    pub fn set_split_vfo(
        &self,
        rx_vfo: String,
        split: bool,
        tx_vfo: String,
    ) -> anyhow::Result<()> {
        let mut g = self.lock();
        let rx = g.vfo_by_name(&rx_vfo)?.ok_or_else(|| anyhow!("VFO {rx_vfo:?} not found"))?;
        let tx = g.vfo_by_name(&tx_vfo)?.ok_or_else(|| anyhow!("VFO {tx_vfo:?} not found"))?;
        Ok(g.set_split_vfo(&rx, split, &tx)?)
    }

    /// Returns the TX frequency in Hz when in split mode.
    pub fn get_split_freq(&self, tx_vfo: String) -> anyhow::Result<u64> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&tx_vfo)?.ok_or_else(|| anyhow!("VFO {tx_vfo:?} not found"))?;
        Ok(g.get_split_freq(&v)?.hz())
    }

    /// Sets the TX frequency in Hz when in split mode.
    pub fn set_split_freq(&self, tx_vfo: String, hz: u64) -> anyhow::Result<()> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&tx_vfo)?.ok_or_else(|| anyhow!("VFO {tx_vfo:?} not found"))?;
        Ok(g.set_split_freq(&v, Frequency(hz))?)
    }

    // ── CTCSS / DCS ───────────────────────────────────────────────────────────

    /// Returns the CTCSS TX tone in units of 0.1 Hz (e.g. 1318 = 131.8 Hz).
    pub fn get_ctcss_tone(&self, vfo: String) -> anyhow::Result<u32> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.get_ctcss_tone(&v)?)
    }

    /// Sets the CTCSS TX tone in units of 0.1 Hz.
    pub fn set_ctcss_tone(&self, vfo: String, tone: u32) -> anyhow::Result<()> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.set_ctcss_tone(&v, tone)?)
    }

    /// Returns the CTCSS squelch tone in units of 0.1 Hz.
    pub fn get_ctcss_sql(&self, vfo: String) -> anyhow::Result<u32> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.get_ctcss_sql(&v)?)
    }

    /// Sets the CTCSS squelch tone in units of 0.1 Hz.
    pub fn set_ctcss_sql(&self, vfo: String, tone: u32) -> anyhow::Result<()> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.set_ctcss_sql(&v, tone)?)
    }

    /// Returns the DCS TX code.
    pub fn get_dcs_code(&self, vfo: String) -> anyhow::Result<u32> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.get_dcs_code(&v)?)
    }

    /// Sets the DCS TX code.
    pub fn set_dcs_code(&self, vfo: String, code: u32) -> anyhow::Result<()> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.set_dcs_code(&v, code)?)
    }

    // ── Repeater ──────────────────────────────────────────────────────────────

    /// Returns the repeater shift direction: `"Simplex"`, `"Plus"`, or `"Minus"`.
    pub fn get_rptr_shift(&self, vfo: String) -> anyhow::Result<String> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.get_rptr_shift(&v)?.to_string())
    }

    /// Sets the repeater shift direction. Accepts `"Simplex"`, `"Plus"`, `"Minus"`,
    /// `"None"`, `"+"`, or `"-"` (case-insensitive).
    pub fn set_rptr_shift(&self, vfo: String, shift: String) -> anyhow::Result<()> {
        let s = parse_rptr_shift(&shift)?;
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.set_rptr_shift(&v, s)?)
    }

    /// Returns the repeater offset in Hz.
    pub fn get_rptr_offs(&self, vfo: String) -> anyhow::Result<i64> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.get_rptr_offs(&v)?)
    }

    /// Sets the repeater offset in Hz.
    pub fn set_rptr_offs(&self, vfo: String, hz: i64) -> anyhow::Result<()> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.set_rptr_offs(&v, hz)?)
    }

    // ── Tuning step ───────────────────────────────────────────────────────────

    /// Returns the VFO tuning step in Hz.
    pub fn get_ts(&self, vfo: String) -> anyhow::Result<u64> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.get_ts(&v)?)
    }

    /// Sets the VFO tuning step in Hz.
    pub fn set_ts(&self, vfo: String, hz: u64) -> anyhow::Result<()> {
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.set_ts(&v, hz)?)
    }

    // ── VFO operations ────────────────────────────────────────────────────────

    /// Executes a discrete VFO operation (e.g. `"Toggle A/B"`, `"Copy A→B"`).
    /// Use `supported_vfo_ops()` to obtain the list of valid operation names.
    pub fn vfo_op(&self, vfo: String, op: String) -> anyhow::Result<()> {
        let o = parse_vfo_op(&op)?;
        let mut g = self.lock();
        let v = g.vfo_by_name(&vfo)?.ok_or_else(|| anyhow!("VFO {vfo:?} not found"))?;
        Ok(g.vfo_op(&v, o)?)
    }

    // ── Power state ───────────────────────────────────────────────────────────

    /// Returns the radio power state: `"On"`, `"Off"`, `"Standby"`, or `"Operate"`.
    pub fn get_powerstat(&self) -> anyhow::Result<String> {
        Ok(self.lock().get_powerstat()?.to_string())
    }

    /// Sets the radio power state. Accepts `"On"`, `"Off"`, `"Standby"`, or `"Operate"`.
    pub fn set_powerstat(&self, state: String) -> anyhow::Result<()> {
        Ok(self.lock().set_powerstat(parse_power_state(&state)?)?)
    }

    // ── Miscellaneous ─────────────────────────────────────────────────────────

    /// Returns the radio's identification string.
    pub fn get_info(&self) -> anyhow::Result<String> {
        Ok(self.lock().get_info()?)
    }
}
