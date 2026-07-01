pub mod hamlib;

pub use hamlib::HamlibTransceiver;

use crate::error::{Error, Result};

// ── VFO ───────────────────────────────────────────────────────────────────────

/// An opaque VFO descriptor.
///
/// Instances are obtained from [`Transceiver::vfo_list`] or
/// [`Transceiver::vfo_by_name`] — never constructed directly by the caller.
/// The `id` field is a driver-internal token whose meaning varies by
/// implementation; callers should treat `Vfo` as an opaque handle identified
/// only by its human-readable name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vfo {
    name: String,
    /// Driver-internal identifier. For hamlib: vfo_t bitmask. For other
    /// drivers: whatever integer the implementation chooses.
    id: u64,
}

impl Vfo {
    /// The canonical name of this VFO (e.g. `"A"`, `"B"`, `"Main"`, `"Sub"`).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Construct a Vfo. Only driver code inside this crate should call this.
    pub(crate) fn new(name: impl Into<String>, id: u64) -> Self {
        Self { name: name.into(), id }
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }
}

impl std::fmt::Display for Vfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

// ── Basic types ───────────────────────────────────────────────────────────────

/// Frequency in integer Hz. Avoids floating-point rounding across the API boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frequency(pub u64);

impl Frequency {
    pub fn from_mhz(mhz: f64) -> Self {
        Self((mhz * 1_000_000.0).round() as u64)
    }

    pub fn as_mhz(self) -> f64 {
        self.0 as f64 / 1_000_000.0
    }

    pub fn hz(self) -> u64 {
        self.0
    }
}

/// Operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Am,
    Cw,
    CwR,
    Usb,
    Lsb,
    Fm,
    Wfm,
    Rtty,
    RttyR,
    PktUsb,
    PktLsb,
    PktFm,
    /// Any mode not covered above, identified by its raw hamlib bitmask value.
    Other(u64),
}

/// Passband (filter) width in Hz.
/// `None` asks the radio to use its default width for the selected mode.
pub type Passband = Option<u32>;

/// PTT state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ptt {
    Rx,
    Tx,
}

// ── Level controls ────────────────────────────────────────────────────────────

/// Identifies which level to get or set.
///
/// Float levels accept/return values in `[0.0, 1.0]` unless noted otherwise.
/// Int levels use the unit stated in each variant's documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    // Float [0.0 – 1.0]
    AfGain,
    RfGain,
    Squelch,
    AudioPeakFilter,
    NoiseReduction,
    PbtIn,
    PbtOut,
    RfPower,
    MicGain,
    Comp,
    VoxGain,
    AntiVox,
    Balance,
    MonitorGain,
    NoiseBlanker,
    NotchFreqRaw,

    // Int (unit in comment)
    Preamp,        // dB
    Attenuation,   // dB
    VoxDelay,      // tenths of a second
    IfShift,       // Hz (signed)
    CwPitch,       // Hz
    KeySpeed,      // WPM
    NotchFreq,     // Hz
    Agc,           // see agc_level_e
    BkInDelay,     // tenths of a dot
    BkInDelayMs,   // milliseconds
    SlopeLow,      // Hz
    SlopeHigh,     // Hz

    // Read-only meters
    Strength,          // dB relative to S9 (S9 = 0, S1 ≈ −48)
    Swr,               // float
    Alc,               // float
    RfPowerMeter,      // float [0.0 – 1.0]
    RfPowerWatts,      // float, watts
    CompMeter,         // float, dB
    VdMeter,           // float, volts
    IdMeter,           // float, amperes
    TempMeter,         // float, °C

    /// Raw hamlib level bitmask for any level not listed above.
    Other(u64),
}

/// Value returned by or supplied to level get/set operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LevelValue {
    Float(f32),
    Int(i32),
}

// ── Functions (boolean toggles) ───────────────────────────────────────────────

/// Identifies a boolean radio function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Func {
    FastAgc,
    NoiseBlanker,
    Comp,
    Vox,
    CtcssTone,
    CtcssSql,
    SemiBreakIn,
    FullBreakIn,
    AutoNotch,
    NoiseReduction,
    Aip,
    AutoPeakFilter,
    Monitor,
    ManualNotch,
    RttyFilter,
    AutoRptrOffset,
    Lock,
    Mute,
    VoiceScan,
    Reverse,
    Squelch,
    AutoBandMode,
    BeatCancel,
    ManualBeatCancel,
    Rit,
    Afc,
    SatMode,
    Scope,
    ScanResume,
    ToneBurst,
    Tuner,
    Xit,
    NoiseBlanker2,
    DcsSql,
    AfFilter,
    NoiseLimiter,
    BeatCancel2,
    DualWatch,
    Diversity,
    /// Raw hamlib func bitmask for any function not listed above.
    Other(u64),
}

// ── Radio parameters (not VFO-specific) ──────────────────────────────────────

/// Identifies a radio-wide parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parm {
    Announce,
    AutoPowerOff, // int, minutes
    Backlight,    // float [0.0 – 1.0]
    Beep,         // int (0/1)
    Time,         // int, seconds since 00:00:00
    Battery,      // float [0.0 – 1.0], read-only
    KeyLight,     // float [0.0 – 1.0]
    ScreenSaver,  // int
    AfIf,         // int (0 = AF, 1 = IF audio output)
    /// Raw hamlib parm bitmask for any parameter not listed above.
    Other(u64),
}

// ── Repeater ─────────────────────────────────────────────────────────────────

/// Repeater duplex shift direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RptrShift {
    None,
    Minus,
    Plus,
}

// ── VFO operations ────────────────────────────────────────────────────────────

/// Discrete VFO operations (actions, not queries).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfoOp {
    CopyAToB,
    Exchange,
    ToMemory,
    FromMemory,
    MemClear,
    StepUp,
    StepDown,
    BandUp,
    BandDown,
    TuneLeft,
    TuneRight,
    AutoTune,
    Toggle,
    /// Raw hamlib vfo_op_t value for any operation not listed above.
    Other(u32),
}

// ── Scan ─────────────────────────────────────────────────────────────────────

/// Scan mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanOp {
    Stop,
    Memory,
    Selected,
    Priority,
    Program,
    Delta,
    Vfo,
    Plt,
    /// Raw hamlib scan_t value.
    Other(u32),
}

// ── Power state ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    Off,
    On,
    Standby,
    Operate,
    /// Raw hamlib powerstat_t value.
    Other(u32),
}

// ── Data carrier detect ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dcd {
    /// No carrier / squelch closed.
    Off,
    /// Carrier detected / squelch open.
    On,
}

// ── Trait ─────────────────────────────────────────────────────────────────────

/// Core operations common to all transceiver implementations.
///
/// All methods take `&mut self` because even read operations send commands
/// over a serial or network link, which requires exclusive access to the
/// underlying I/O resource.
///
/// Implementations must be `Send` so they can be moved into worker threads
/// or across the FFI boundary.
///
/// Optional methods return `Err(Error::NotSupported)` by default. Override
/// them in implementations that have the capability.
pub trait Transceiver: Send {
    // ── VFO discovery (required) ──────────────────────────────────────────────

    /// Returns the list of VFOs this radio supports.
    ///
    /// Each entry is an opaque `Vfo` handle with a human-readable name.
    /// Callers should obtain VFO handles from this method or from
    /// `vfo_by_name`, never by constructing them directly.
    fn vfo_list(&mut self) -> Result<Vec<Vfo>>;

    /// Returns the VFO whose name matches `name` (case-insensitive), or
    /// `None` if this radio does not have a VFO by that name.
    ///
    /// The default implementation searches `vfo_list()`; drivers may override
    /// this to support additional aliases.
    fn vfo_by_name(&mut self, name: &str) -> Result<Option<Vfo>> {
        Ok(self
            .vfo_list()?
            .into_iter()
            .find(|v| v.name().eq_ignore_ascii_case(name)))
    }

    // ── Required ─────────────────────────────────────────────────────────────

    fn get_freq(&mut self, vfo: &Vfo) -> Result<Frequency>;
    fn set_freq(&mut self, vfo: &Vfo, freq: Frequency) -> Result<()>;

    fn get_mode(&mut self, vfo: &Vfo) -> Result<(Mode, Passband)>;
    fn set_mode(&mut self, vfo: &Vfo, mode: Mode, passband: Passband) -> Result<()>;

    fn get_ptt(&mut self) -> Result<Ptt>;
    fn set_ptt(&mut self, ptt: Ptt) -> Result<()>;

    // ── Capability discovery (optional) ──────────────────────────────────────

    /// Returns the operating modes this radio can receive and/or transmit.
    fn supported_modes(&mut self) -> Result<Vec<Mode>> {
        Err(Error::NotSupported)
    }

    /// Returns the levels (gain controls, meters, etc.) this radio supports.
    /// Includes both readable and writable levels.
    fn supported_levels(&mut self) -> Result<Vec<Level>> {
        Err(Error::NotSupported)
    }

    /// Returns the boolean functions (NB, NR, VOX, etc.) this radio supports.
    fn supported_funcs(&mut self) -> Result<Vec<Func>> {
        Err(Error::NotSupported)
    }

    /// Returns the radio-wide parameters this radio supports.
    fn supported_parms(&mut self) -> Result<Vec<Parm>> {
        Err(Error::NotSupported)
    }

    /// Returns the VFO operations this radio supports.
    fn supported_vfo_ops(&mut self) -> Result<Vec<VfoOp>> {
        Err(Error::NotSupported)
    }

    /// Returns the scan modes this radio supports.
    fn supported_scan_ops(&mut self) -> Result<Vec<ScanOp>> {
        Err(Error::NotSupported)
    }

    // ── Levels ───────────────────────────────────────────────────────────────

    fn get_level(&mut self, _vfo: &Vfo, _level: Level) -> Result<LevelValue> {
        Err(Error::NotSupported)
    }
    fn set_level(&mut self, _vfo: &Vfo, _level: Level, _value: LevelValue) -> Result<()> {
        Err(Error::NotSupported)
    }

    // ── Functions ─────────────────────────────────────────────────────────────

    fn get_func(&mut self, _vfo: &Vfo, _func: Func) -> Result<bool> {
        Err(Error::NotSupported)
    }
    fn set_func(&mut self, _vfo: &Vfo, _func: Func, _enabled: bool) -> Result<()> {
        Err(Error::NotSupported)
    }

    // ── Parameters ───────────────────────────────────────────────────────────

    fn get_parm(&mut self, _parm: Parm) -> Result<LevelValue> {
        Err(Error::NotSupported)
    }
    fn set_parm(&mut self, _parm: Parm, _value: LevelValue) -> Result<()> {
        Err(Error::NotSupported)
    }

    // ── RIT / XIT ─────────────────────────────────────────────────────────────

    /// Receive Incremental Tuning offset in Hz.
    fn get_rit(&mut self, _vfo: &Vfo) -> Result<i64> {
        Err(Error::NotSupported)
    }
    fn set_rit(&mut self, _vfo: &Vfo, _hz: i64) -> Result<()> {
        Err(Error::NotSupported)
    }

    /// Transmit Incremental Tuning offset in Hz.
    fn get_xit(&mut self, _vfo: &Vfo) -> Result<i64> {
        Err(Error::NotSupported)
    }
    fn set_xit(&mut self, _vfo: &Vfo, _hz: i64) -> Result<()> {
        Err(Error::NotSupported)
    }

    // ── Split operation ───────────────────────────────────────────────────────

    /// Returns `(split_active, tx_vfo)`.
    fn get_split_vfo(&mut self, _rx_vfo: &Vfo) -> Result<(bool, Vfo)> {
        Err(Error::NotSupported)
    }
    fn set_split_vfo(&mut self, _rx_vfo: &Vfo, _split: bool, _tx_vfo: &Vfo) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn get_split_freq(&mut self, _tx_vfo: &Vfo) -> Result<Frequency> {
        Err(Error::NotSupported)
    }
    fn set_split_freq(&mut self, _tx_vfo: &Vfo, _freq: Frequency) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn get_split_mode(&mut self, _tx_vfo: &Vfo) -> Result<(Mode, Passband)> {
        Err(Error::NotSupported)
    }
    fn set_split_mode(&mut self, _tx_vfo: &Vfo, _mode: Mode, _passband: Passband) -> Result<()> {
        Err(Error::NotSupported)
    }

    // ── CTCSS / DCS ───────────────────────────────────────────────────────────

    /// CTCSS TX tone in units of 0.1 Hz (e.g. 1318 = 131.8 Hz).
    fn get_ctcss_tone(&mut self, _vfo: &Vfo) -> Result<u32> {
        Err(Error::NotSupported)
    }
    fn set_ctcss_tone(&mut self, _vfo: &Vfo, _tone: u32) -> Result<()> {
        Err(Error::NotSupported)
    }

    /// CTCSS squelch tone in units of 0.1 Hz.
    fn get_ctcss_sql(&mut self, _vfo: &Vfo) -> Result<u32> {
        Err(Error::NotSupported)
    }
    fn set_ctcss_sql(&mut self, _vfo: &Vfo, _tone: u32) -> Result<()> {
        Err(Error::NotSupported)
    }

    /// DCS TX code.
    fn get_dcs_code(&mut self, _vfo: &Vfo) -> Result<u32> {
        Err(Error::NotSupported)
    }
    fn set_dcs_code(&mut self, _vfo: &Vfo, _code: u32) -> Result<()> {
        Err(Error::NotSupported)
    }

    /// DCS squelch code.
    fn get_dcs_sql(&mut self, _vfo: &Vfo) -> Result<u32> {
        Err(Error::NotSupported)
    }
    fn set_dcs_sql(&mut self, _vfo: &Vfo, _code: u32) -> Result<()> {
        Err(Error::NotSupported)
    }

    // ── Repeater ──────────────────────────────────────────────────────────────

    fn get_rptr_shift(&mut self, _vfo: &Vfo) -> Result<RptrShift> {
        Err(Error::NotSupported)
    }
    fn set_rptr_shift(&mut self, _vfo: &Vfo, _shift: RptrShift) -> Result<()> {
        Err(Error::NotSupported)
    }

    /// Repeater offset in Hz (signed; negative for minus shift).
    fn get_rptr_offs(&mut self, _vfo: &Vfo) -> Result<i64> {
        Err(Error::NotSupported)
    }
    fn set_rptr_offs(&mut self, _vfo: &Vfo, _offset: i64) -> Result<()> {
        Err(Error::NotSupported)
    }

    // ── Tuning step ───────────────────────────────────────────────────────────

    /// Tuning step in Hz.
    fn get_ts(&mut self, _vfo: &Vfo) -> Result<u64> {
        Err(Error::NotSupported)
    }
    fn set_ts(&mut self, _vfo: &Vfo, _step: u64) -> Result<()> {
        Err(Error::NotSupported)
    }

    // ── Antenna ───────────────────────────────────────────────────────────────

    /// Returns the currently selected antenna number (1-based).
    fn get_ant(&mut self, _vfo: &Vfo) -> Result<u32> {
        Err(Error::NotSupported)
    }
    /// Selects antenna by number (1-based).
    fn set_ant(&mut self, _vfo: &Vfo, _ant: u32) -> Result<()> {
        Err(Error::NotSupported)
    }

    // ── VFO operations ────────────────────────────────────────────────────────

    fn vfo_op(&mut self, _vfo: &Vfo, _op: VfoOp) -> Result<()> {
        Err(Error::NotSupported)
    }

    // ── Scan ─────────────────────────────────────────────────────────────────

    /// `ch` is used by some scan modes (e.g. program channel for `ScanOp::Program`).
    fn scan(&mut self, _vfo: &Vfo, _op: ScanOp, _ch: i32) -> Result<()> {
        Err(Error::NotSupported)
    }

    // ── Power ─────────────────────────────────────────────────────────────────

    fn get_powerstat(&mut self) -> Result<PowerState> {
        Err(Error::NotSupported)
    }
    fn set_powerstat(&mut self, _state: PowerState) -> Result<()> {
        Err(Error::NotSupported)
    }

    // ── Miscellaneous ─────────────────────────────────────────────────────────

    /// Data carrier detect: whether the squelch is open.
    fn get_dcd(&mut self, _vfo: &Vfo) -> Result<Dcd> {
        Err(Error::NotSupported)
    }

    /// Human-readable identification string from the radio.
    fn get_info(&mut self) -> Result<String> {
        Err(Error::NotSupported)
    }

    /// Start a spectrum stream using this transceiver's built-in scope.
    ///
    /// Returns `Err(NotSupported)` for radios that have no accessible built-in
    /// spectrum display.  In that case, connect a standalone
    /// [`SpectrumSource`] (SDR dongle, audio FFT, …) independently.
    ///
    /// Because [`SpectrumStream`] runs on its own thread and does not borrow
    /// from `self`, the transceiver is available for normal control calls
    /// (frequency, mode, PTT, …) immediately after `start_spectrum` returns.
    ///
    /// [`SpectrumSource`]: crate::spectrum::SpectrumSource
    /// [`SpectrumStream`]: crate::spectrum::SpectrumStream
    fn start_spectrum(
        &mut self,
        _config: crate::spectrum::SpectrumConfig,
    ) -> Result<crate::spectrum::SpectrumStream> {
        Err(Error::NotSupported)
    }
}

// ── Display implementations ───────────────────────────────────────────────────

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Mode::Am => "AM",
            Mode::Cw => "CW",
            Mode::CwR => "CW-R",
            Mode::Usb => "USB",
            Mode::Lsb => "LSB",
            Mode::Fm => "FM",
            Mode::Wfm => "WFM",
            Mode::Rtty => "RTTY",
            Mode::RttyR => "RTTY-R",
            Mode::PktUsb => "Pkt-USB",
            Mode::PktLsb => "Pkt-LSB",
            Mode::PktFm => "Pkt-FM",
            Mode::Other(v) => return write!(f, "Mode(0x{v:x})"),
        })
    }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Level::AfGain => "AF Gain",
            Level::RfGain => "RF Gain",
            Level::Squelch => "Squelch",
            Level::AudioPeakFilter => "Audio Peak Filter",
            Level::NoiseReduction => "Noise Reduction",
            Level::PbtIn => "PBT (inner)",
            Level::PbtOut => "PBT (outer)",
            Level::RfPower => "RF Power",
            Level::MicGain => "Mic Gain",
            Level::Comp => "Speech Compression",
            Level::VoxGain => "VOX Gain",
            Level::AntiVox => "Anti-VOX",
            Level::Balance => "Balance",
            Level::MonitorGain => "Monitor Gain",
            Level::NoiseBlanker => "Noise Blanker Level",
            Level::NotchFreqRaw => "Notch Freq (raw)",
            Level::Preamp => "Preamp",
            Level::Attenuation => "Attenuation",
            Level::VoxDelay => "VOX Delay",
            Level::IfShift => "IF Shift",
            Level::CwPitch => "CW Pitch",
            Level::KeySpeed => "Key Speed",
            Level::NotchFreq => "Notch Freq",
            Level::Agc => "AGC",
            Level::BkInDelay => "Break-in Delay",
            Level::BkInDelayMs => "Break-in Delay (ms)",
            Level::SlopeLow => "Slope Low",
            Level::SlopeHigh => "Slope High",
            Level::Strength => "Signal Strength",
            Level::Swr => "SWR",
            Level::Alc => "ALC",
            Level::RfPowerMeter => "RF Power Meter",
            Level::RfPowerWatts => "RF Power (W)",
            Level::CompMeter => "Compression Meter",
            Level::VdMeter => "Voltage",
            Level::IdMeter => "Current",
            Level::TempMeter => "Temperature",
            Level::Other(v) => return write!(f, "Level(0x{v:x})"),
        })
    }
}

impl std::fmt::Display for Func {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Func::FastAgc => "Fast AGC",
            Func::NoiseBlanker => "Noise Blanker",
            Func::Comp => "Speech Compression",
            Func::Vox => "VOX",
            Func::CtcssTone => "CTCSS Tone TX",
            Func::CtcssSql => "CTCSS Squelch",
            Func::SemiBreakIn => "Semi Break-in",
            Func::FullBreakIn => "Full Break-in",
            Func::AutoNotch => "Auto Notch",
            Func::NoiseReduction => "Noise Reduction",
            Func::Aip => "AIP",
            Func::AutoPeakFilter => "Auto Peak Filter",
            Func::Monitor => "Monitor",
            Func::ManualNotch => "Manual Notch",
            Func::RttyFilter => "RTTY Filter",
            Func::AutoRptrOffset => "Auto Repeater Offset",
            Func::Lock => "Lock",
            Func::Mute => "Mute",
            Func::VoiceScan => "Voice Scan",
            Func::Reverse => "Reverse",
            Func::Squelch => "Squelch",
            Func::AutoBandMode => "Auto Band Mode",
            Func::BeatCancel => "Beat Canceller",
            Func::ManualBeatCancel => "Manual Beat Canceller",
            Func::Rit => "RIT",
            Func::Afc => "AFC",
            Func::SatMode => "Satellite Mode",
            Func::Scope => "Scope",
            Func::ScanResume => "Scan Resume",
            Func::ToneBurst => "Tone Burst",
            Func::Tuner => "Tuner",
            Func::Xit => "XIT",
            Func::NoiseBlanker2 => "Noise Blanker 2",
            Func::DcsSql => "DCS Squelch",
            Func::AfFilter => "AF Filter",
            Func::NoiseLimiter => "Noise Limiter",
            Func::BeatCancel2 => "Beat Canceller 2",
            Func::DualWatch => "Dual Watch",
            Func::Diversity => "Diversity",
            Func::Other(v) => return write!(f, "Func(0x{v:x})"),
        })
    }
}

impl std::fmt::Display for Parm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Parm::Announce => "Announce",
            Parm::AutoPowerOff => "Auto Power-off",
            Parm::Backlight => "Backlight",
            Parm::Beep => "Beep",
            Parm::Time => "Clock",
            Parm::Battery => "Battery",
            Parm::KeyLight => "Key Light",
            Parm::ScreenSaver => "Screen Saver",
            Parm::AfIf => "AF/IF Output",
            Parm::Other(v) => return write!(f, "Parm(0x{v:x})"),
        })
    }
}

impl std::fmt::Display for VfoOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            VfoOp::CopyAToB => "Copy A→B",
            VfoOp::Exchange => "Exchange A/B",
            VfoOp::ToMemory => "VFO→Memory",
            VfoOp::FromMemory => "Memory→VFO",
            VfoOp::MemClear => "Memory Clear",
            VfoOp::StepUp => "Step Up",
            VfoOp::StepDown => "Step Down",
            VfoOp::BandUp => "Band Up",
            VfoOp::BandDown => "Band Down",
            VfoOp::TuneLeft => "Tune Left",
            VfoOp::TuneRight => "Tune Right",
            VfoOp::AutoTune => "Auto Tune",
            VfoOp::Toggle => "Toggle A/B",
            VfoOp::Other(v) => return write!(f, "VfoOp(0x{v:x})"),
        })
    }
}

impl std::fmt::Display for ScanOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ScanOp::Stop => "Stop",
            ScanOp::Memory => "Memory Scan",
            ScanOp::Selected => "Selected Scan",
            ScanOp::Priority => "Priority Scan",
            ScanOp::Program => "Program Scan",
            ScanOp::Delta => "Delta Scan",
            ScanOp::Vfo => "VFO Scan",
            ScanOp::Plt => "Pilot Tone Scan",
            ScanOp::Other(v) => return write!(f, "ScanOp(0x{v:x})"),
        })
    }
}

impl std::fmt::Display for RptrShift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RptrShift::None => "Simplex",
            RptrShift::Minus => "Minus",
            RptrShift::Plus => "Plus",
        })
    }
}

impl std::fmt::Display for PowerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            PowerState::Off => "Off",
            PowerState::On => "On",
            PowerState::Standby => "Standby",
            PowerState::Operate => "Operate",
            PowerState::Other(v) => return write!(f, "PowerState({v})"),
        })
    }
}

impl std::fmt::Display for Ptt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Ptt::Rx => "Receive",
            Ptt::Tx => "Transmit",
        })
    }
}

impl std::fmt::Display for Dcd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Dcd::Off => "Squelch Closed",
            Dcd::On => "Carrier Detected",
        })
    }
}
