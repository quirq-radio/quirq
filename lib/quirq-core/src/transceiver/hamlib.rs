#![allow(non_upper_case_globals)] // bindgen-generated constants don't follow Rust naming

use std::ffi::{CStr, CString};
use std::sync::Mutex;

use hamlib_sys::*;

use super::{
    Dcd, Frequency, Func, Level, LevelValue, Mode, Parm, Passband, PowerState, Ptt, RptrShift,
    ScanOp, Transceiver, Vfo, VfoOp,
};
use crate::error::{Error, Result};

// rig_init writes to hamlib's global rig-model registry on its first call.
// Concurrent calls race on that write, causing the "Hash collision" abort.
// Per-rig operations after open() touch only each rig's private RIG* state
// and need no synchronisation beyond &mut self.
static RIG_INIT: Mutex<()> = Mutex::new(());

// ── Model discovery ───────────────────────────────────────────────────────────

/// Describes a single radio model registered in this build of libhamlib.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Hamlib model number — opaque to callers; pass back to `open()` via the
    /// `"model"` param.
    pub model_id: u32,
    pub manufacturer: String,
    pub model_name: String,
    pub version: String,
    /// Human-readable status: `"Stable"`, `"Beta"`, `"Alpha"`, `"Untested"`, or `"Buggy"`.
    pub status: String,
}

/// Returns every radio model registered in this build of libhamlib.
///
/// Calls `rig_load_all_backends()` first so that the list is complete even if
/// no rig has been opened yet.
pub fn list_models() -> Vec<ModelInfo> {
    // rig_load_all_backends touches the same global registry as rig_init, so
    // we hold the init lock for the duration.
    let _guard = RIG_INIT.lock().unwrap();

    unsafe {
        rig_load_all_backends();
    }

    let mut result: Vec<ModelInfo> = Vec::new();

    unsafe extern "C" fn collect(
        caps: *const rig_caps,
        data: *mut std::os::raw::c_void,
    ) -> std::os::raw::c_int {
        let list = &mut *(data as *mut Vec<ModelInfo>);
        let c = &*caps;

        let cstr = |ptr: *const std::os::raw::c_char| {
            if ptr.is_null() {
                String::new()
            } else {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        };

        let status = match c.status {
            rig_status_e_RIG_STATUS_STABLE => "Stable",
            rig_status_e_RIG_STATUS_BETA => "Beta",
            rig_status_e_RIG_STATUS_ALPHA => "Alpha",
            rig_status_e_RIG_STATUS_UNTESTED => "Untested",
            rig_status_e_RIG_STATUS_BUGGY => "Buggy",
            _ => "Unknown",
        };

        list.push(ModelInfo {
            model_id: c.rig_model,
            manufacturer: cstr(c.mfg_name),
            model_name: cstr(c.model_name),
            version: cstr(c.version),
            status: status.into(),
        });

        1 // non-zero = continue
    }

    unsafe {
        rig_list_foreach(
            Some(collect),
            &mut result as *mut Vec<ModelInfo> as *mut std::os::raw::c_void,
        );
    }

    result
}

// ── VFO name table ────────────────────────────────────────────────────────────
// Maps hamlib vfo_t bitmasks to human-readable names used in Vfo::name().
// Ordered from most specific to most general so that vfo_from_id() picks the
// best name when a bitmask could match multiple entries.

type VfoEntry = (vfo_t, &'static str);

const VFO_TABLE: &[VfoEntry] = &[
    (1 << 0,  "A"),
    (1 << 1,  "B"),
    (1 << 2,  "C"),
    (1 << 3,  "SubC"),
    (1 << 4,  "MainC"),
    (1 << 21, "SubA"),
    (1 << 22, "SubB"),
    (1 << 23, "MainA"),
    (1 << 24, "MainB"),
    (1 << 25, "Sub"),
    (1 << 26, "Main"),
    (1 << 28, "Memory"),
    (1 << 29, "Current"),
];

// ── Mode constants ────────────────────────────────────────────────────────────
// CONSTANT_64BIT_FLAG(n) = 1ul << n.

const MODE_AM: rmode_t = 1 << 0;
const MODE_CW: rmode_t = 1 << 1;
const MODE_USB: rmode_t = 1 << 2;
const MODE_LSB: rmode_t = 1 << 3;
const MODE_RTTY: rmode_t = 1 << 4;
const MODE_FM: rmode_t = 1 << 5;
const MODE_WFM: rmode_t = 1 << 6;
const MODE_CWR: rmode_t = 1 << 7;
const MODE_RTTYR: rmode_t = 1 << 8;
const MODE_PKTLSB: rmode_t = 1 << 10;
const MODE_PKTUSB: rmode_t = 1 << 11;
const MODE_PKTFM: rmode_t = 1 << 12;

// ── Level constants ───────────────────────────────────────────────────────────
// CONSTANT_64BIT_FLAG(n) = 1ul << n.  Bit positions from hamlib/rig.h.

const LVL_PREAMP: setting_t = 1 << 0;
const LVL_ATT: setting_t = 1 << 1;
const LVL_VOXDELAY: setting_t = 1 << 2;
const LVL_AF: setting_t = 1 << 3;
const LVL_RF: setting_t = 1 << 4;
const LVL_SQL: setting_t = 1 << 5;
const LVL_IF: setting_t = 1 << 6;
const LVL_APF: setting_t = 1 << 7;
const LVL_NR: setting_t = 1 << 8;
const LVL_PBT_IN: setting_t = 1 << 9;
const LVL_PBT_OUT: setting_t = 1 << 10;
const LVL_CWPITCH: setting_t = 1 << 11;
const LVL_RFPOWER: setting_t = 1 << 12;
const LVL_MICGAIN: setting_t = 1 << 13;
const LVL_KEYSPD: setting_t = 1 << 14;
const LVL_NOTCHF: setting_t = 1 << 15;
const LVL_COMP: setting_t = 1 << 16;
const LVL_AGC: setting_t = 1 << 17;
const LVL_BKINDL: setting_t = 1 << 18;
const LVL_BALANCE: setting_t = 1 << 19;
const LVL_VOXGAIN: setting_t = 1 << 21;
const LVL_ANTIVOX: setting_t = 1 << 22;
const LVL_SLOPE_LOW: setting_t = 1 << 23;
const LVL_SLOPE_HIGH: setting_t = 1 << 24;
const LVL_BKIN_DLYMS: setting_t = 1 << 25;
const LVL_SWR: setting_t = 1 << 28;
const LVL_ALC: setting_t = 1 << 29;
const LVL_STRENGTH: setting_t = 1 << 30;
const LVL_RFPOWER_METER: setting_t = 1 << 32;
const LVL_COMP_METER: setting_t = 1 << 33;
const LVL_VD_METER: setting_t = 1 << 34;
const LVL_ID_METER: setting_t = 1 << 35;
const LVL_NOTCHF_RAW: setting_t = 1 << 36;
const LVL_MONITOR_GAIN: setting_t = 1 << 37;
const LVL_NB: setting_t = 1 << 38;
const LVL_RFPOWER_METER_WATTS: setting_t = 1 << 39;
const LVL_TEMP_METER: setting_t = 1 << 48;

// ── Function constants ────────────────────────────────────────────────────────

const FUNC_FAGC: setting_t = 1 << 0;
const FUNC_NB: setting_t = 1 << 1;
const FUNC_COMP: setting_t = 1 << 2;
const FUNC_VOX: setting_t = 1 << 3;
const FUNC_TONE: setting_t = 1 << 4;
const FUNC_TSQL: setting_t = 1 << 5;
const FUNC_SBKIN: setting_t = 1 << 6;
const FUNC_FBKIN: setting_t = 1 << 7;
const FUNC_ANF: setting_t = 1 << 8;
const FUNC_NR: setting_t = 1 << 9;
const FUNC_AIP: setting_t = 1 << 10;
const FUNC_APF: setting_t = 1 << 11;
const FUNC_MON: setting_t = 1 << 12;
const FUNC_MN: setting_t = 1 << 13;
const FUNC_RF: setting_t = 1 << 14;
const FUNC_ARO: setting_t = 1 << 15;
const FUNC_LOCK: setting_t = 1 << 16;
const FUNC_MUTE: setting_t = 1 << 17;
const FUNC_VSC: setting_t = 1 << 18;
const FUNC_REV: setting_t = 1 << 19;
const FUNC_SQL: setting_t = 1 << 20;
const FUNC_ABM: setting_t = 1 << 21;
const FUNC_BC: setting_t = 1 << 22;
const FUNC_MBC: setting_t = 1 << 23;
const FUNC_RIT: setting_t = 1 << 24;
const FUNC_AFC: setting_t = 1 << 25;
const FUNC_SATMODE: setting_t = 1 << 26;
const FUNC_SCOPE: setting_t = 1 << 27;
const FUNC_RESUME: setting_t = 1 << 28;
const FUNC_TBURST: setting_t = 1 << 29;
const FUNC_TUNER: setting_t = 1 << 30;
const FUNC_XIT: setting_t = 1 << 31;
const FUNC_NB2: setting_t = 1 << 32;
const FUNC_CSQL: setting_t = 1 << 33;
const FUNC_AFLT: setting_t = 1 << 34;
const FUNC_ANL: setting_t = 1 << 35;
const FUNC_BC2: setting_t = 1 << 36;
const FUNC_DUAL_WATCH: setting_t = 1 << 37;
const FUNC_DIVERSITY: setting_t = 1 << 38;

// ── Antenna constants ─────────────────────────────────────────────────────────

const ANT_CURR: ant_t = 1 << 31; // RIG_ANT_CURR — query which antenna is active

// ── HamlibTransceiver ─────────────────────────────────────────────────────────

/// A transceiver driven by libhamlib.
///
/// Wraps hamlib's `RIG *` handle. The rig is closed and cleaned up when
/// this value is dropped.
pub struct HamlibTransceiver {
    rig: *mut RIG,
}

// SAFETY: All access goes through `&mut self`, preventing concurrent use.
// The caller is responsible for not sharing the handle across threads
// without external synchronisation (e.g. a Mutex).
unsafe impl Send for HamlibTransceiver {}

impl HamlibTransceiver {
    /// Open a rig by hamlib model number on the given port path.
    ///
    /// `model` is a hamlib rig model constant (e.g. `3085` for IC-705).
    /// `port` is the device path or hostname (e.g. `"/dev/ttyUSB0"`).
    pub fn open(model: u32, port: &str) -> Result<Self> {
        let port_cstr = CString::new(port)
            .map_err(|_| Error::InvalidArgument("port path contains a null byte".into()))?;
        // On Android the struct layout is opaque so we skip direct port setup;
        // port_cstr is kept for future rig_set_conf integration.
        #[cfg(target_os = "android")]
        let _ = &port_cstr;

        let rig = {
            let _guard = RIG_INIT.lock().unwrap();
            unsafe { rig_init(model) }
        };
        if rig.is_null() {
            return Err(Error::Hamlib {
                code: -1,
                description: "rig_init returned null — unknown model?".into(),
            });
        }

        unsafe {
            // The older hamlib API (system package) exposes rig_state fields
            // directly; the newer API (submodule) makes them opaque and
            // provides pointer accessors. The dummy rig ignores the port path,
            // so on Android we skip the direct-struct setup and rely on
            // rig_open's own defaults.
            #[cfg(not(target_os = "android"))]
            {
                let dest = (*rig).state.rigport.pathname.as_mut_ptr();
                let cap = (*rig).state.rigport.pathname.len();
                let src = port_cstr.as_bytes_with_nul();
                std::ptr::copy_nonoverlapping(
                    src.as_ptr() as *const i8,
                    dest,
                    src.len().min(cap),
                );
                // Default PTT to CAT control. Hamlib gates get/set_ptt on this
                // being non-NONE, so leaving it at the default (NONE) makes all
                // PTT calls fail with "Invalid parameter".
                (*rig).state.pttport.type_.ptt = ptt_type_t_RIG_PTT_RIG;
            }

            Self::hamlib_check(rig_open(rig))?;
        }

        Ok(Self { rig })
    }

    fn hamlib_check(code: i32) -> Result<()> {
        if code == 0 {
            return Ok(());
        }
        let description = unsafe {
            let ptr = rigerror(code);
            if ptr.is_null() {
                "unknown error".into()
            } else {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        };
        Err(Error::Hamlib { code, description })
    }

    /// Build a Vfo from a raw hamlib vfo_t, looking up the friendly name in
    /// VFO_TABLE.  Falls back to a hex string for any unlisted bitmask.
    fn vfo_from_id(id: vfo_t) -> Vfo {
        VFO_TABLE
            .iter()
            .find(|(mask, _)| *mask == id)
            .map(|(mask, name)| Vfo::new(*name, *mask as u64))
            .unwrap_or_else(|| Vfo::new(format!("VFO#{id:08x}"), id as u64))
    }

    fn mode_to_hamlib(mode: Mode) -> rmode_t {
        match mode {
            Mode::Am => MODE_AM,
            Mode::Cw => MODE_CW,
            Mode::CwR => MODE_CWR,
            Mode::Usb => MODE_USB,
            Mode::Lsb => MODE_LSB,
            Mode::Fm => MODE_FM,
            Mode::Wfm => MODE_WFM,
            Mode::Rtty => MODE_RTTY,
            Mode::RttyR => MODE_RTTYR,
            Mode::PktUsb => MODE_PKTUSB,
            Mode::PktLsb => MODE_PKTLSB,
            Mode::PktFm => MODE_PKTFM,
            Mode::Other(v) => v,
        }
    }

    fn mode_from_hamlib(mode: rmode_t) -> Mode {
        match mode {
            MODE_AM => Mode::Am,
            MODE_CW => Mode::Cw,
            MODE_CWR => Mode::CwR,
            MODE_USB => Mode::Usb,
            MODE_LSB => Mode::Lsb,
            MODE_FM => Mode::Fm,
            MODE_WFM => Mode::Wfm,
            MODE_RTTY => Mode::Rtty,
            MODE_RTTYR => Mode::RttyR,
            MODE_PKTUSB => Mode::PktUsb,
            MODE_PKTLSB => Mode::PktLsb,
            MODE_PKTFM => Mode::PktFm,
            v => Mode::Other(v),
        }
    }

    fn level_to_setting(level: &Level) -> setting_t {
        match level {
            Level::Preamp => LVL_PREAMP,
            Level::Attenuation => LVL_ATT,
            Level::VoxDelay => LVL_VOXDELAY,
            Level::AfGain => LVL_AF,
            Level::RfGain => LVL_RF,
            Level::Squelch => LVL_SQL,
            Level::IfShift => LVL_IF,
            Level::AudioPeakFilter => LVL_APF,
            Level::NoiseReduction => LVL_NR,
            Level::PbtIn => LVL_PBT_IN,
            Level::PbtOut => LVL_PBT_OUT,
            Level::CwPitch => LVL_CWPITCH,
            Level::RfPower => LVL_RFPOWER,
            Level::MicGain => LVL_MICGAIN,
            Level::KeySpeed => LVL_KEYSPD,
            Level::NotchFreq => LVL_NOTCHF,
            Level::Comp => LVL_COMP,
            Level::Agc => LVL_AGC,
            Level::BkInDelay => LVL_BKINDL,
            Level::Balance => LVL_BALANCE,
            Level::VoxGain => LVL_VOXGAIN,
            Level::AntiVox => LVL_ANTIVOX,
            Level::SlopeLow => LVL_SLOPE_LOW,
            Level::SlopeHigh => LVL_SLOPE_HIGH,
            Level::BkInDelayMs => LVL_BKIN_DLYMS,
            Level::Swr => LVL_SWR,
            Level::Alc => LVL_ALC,
            Level::Strength => LVL_STRENGTH,
            Level::RfPowerMeter => LVL_RFPOWER_METER,
            Level::CompMeter => LVL_COMP_METER,
            Level::VdMeter => LVL_VD_METER,
            Level::IdMeter => LVL_ID_METER,
            Level::NotchFreqRaw => LVL_NOTCHF_RAW,
            Level::MonitorGain => LVL_MONITOR_GAIN,
            Level::NoiseBlanker => LVL_NB,
            Level::RfPowerWatts => LVL_RFPOWER_METER_WATTS,
            Level::TempMeter => LVL_TEMP_METER,
            Level::Other(v) => *v,
        }
    }

    fn level_is_float(level: &Level) -> bool {
        matches!(
            level,
            Level::AfGain
                | Level::RfGain
                | Level::Squelch
                | Level::AudioPeakFilter
                | Level::NoiseReduction
                | Level::PbtIn
                | Level::PbtOut
                | Level::RfPower
                | Level::MicGain
                | Level::Comp
                | Level::VoxGain
                | Level::AntiVox
                | Level::Balance
                | Level::MonitorGain
                | Level::NoiseBlanker
                | Level::NotchFreqRaw
                | Level::Swr
                | Level::Alc
                | Level::RfPowerMeter
                | Level::RfPowerWatts
                | Level::CompMeter
                | Level::VdMeter
                | Level::IdMeter
                | Level::TempMeter
        )
    }

    fn func_to_setting(func: &Func) -> setting_t {
        match func {
            Func::FastAgc => FUNC_FAGC,
            Func::NoiseBlanker => FUNC_NB,
            Func::Comp => FUNC_COMP,
            Func::Vox => FUNC_VOX,
            Func::CtcssTone => FUNC_TONE,
            Func::CtcssSql => FUNC_TSQL,
            Func::SemiBreakIn => FUNC_SBKIN,
            Func::FullBreakIn => FUNC_FBKIN,
            Func::AutoNotch => FUNC_ANF,
            Func::NoiseReduction => FUNC_NR,
            Func::Aip => FUNC_AIP,
            Func::AutoPeakFilter => FUNC_APF,
            Func::Monitor => FUNC_MON,
            Func::ManualNotch => FUNC_MN,
            Func::RttyFilter => FUNC_RF,
            Func::AutoRptrOffset => FUNC_ARO,
            Func::Lock => FUNC_LOCK,
            Func::Mute => FUNC_MUTE,
            Func::VoiceScan => FUNC_VSC,
            Func::Reverse => FUNC_REV,
            Func::Squelch => FUNC_SQL,
            Func::AutoBandMode => FUNC_ABM,
            Func::BeatCancel => FUNC_BC,
            Func::ManualBeatCancel => FUNC_MBC,
            Func::Rit => FUNC_RIT,
            Func::Afc => FUNC_AFC,
            Func::SatMode => FUNC_SATMODE,
            Func::Scope => FUNC_SCOPE,
            Func::ScanResume => FUNC_RESUME,
            Func::ToneBurst => FUNC_TBURST,
            Func::Tuner => FUNC_TUNER,
            Func::Xit => FUNC_XIT,
            Func::NoiseBlanker2 => FUNC_NB2,
            Func::DcsSql => FUNC_CSQL,
            Func::AfFilter => FUNC_AFLT,
            Func::NoiseLimiter => FUNC_ANL,
            Func::BeatCancel2 => FUNC_BC2,
            Func::DualWatch => FUNC_DUAL_WATCH,
            Func::Diversity => FUNC_DIVERSITY,
            Func::Other(v) => *v,
        }
    }

    fn parm_to_setting(parm: &Parm) -> setting_t {
        match parm {
            Parm::Announce => rig_parm_e_RIG_PARM_ANN as setting_t,
            Parm::AutoPowerOff => rig_parm_e_RIG_PARM_APO as setting_t,
            Parm::Backlight => rig_parm_e_RIG_PARM_BACKLIGHT as setting_t,
            Parm::Beep => rig_parm_e_RIG_PARM_BEEP as setting_t,
            Parm::Time => rig_parm_e_RIG_PARM_TIME as setting_t,
            Parm::Battery => rig_parm_e_RIG_PARM_BAT as setting_t,
            Parm::KeyLight => rig_parm_e_RIG_PARM_KEYLIGHT as setting_t,
            Parm::ScreenSaver => rig_parm_e_RIG_PARM_SCREENSAVER as setting_t,
            Parm::AfIf => rig_parm_e_RIG_PARM_AFIF as setting_t,
            Parm::Other(v) => *v,
        }
    }

    fn vfoop_to_hamlib(op: VfoOp) -> vfo_op_t {
        match op {
            VfoOp::CopyAToB => vfo_op_t_RIG_OP_CPY,
            VfoOp::Exchange => vfo_op_t_RIG_OP_XCHG,
            VfoOp::ToMemory => vfo_op_t_RIG_OP_FROM_VFO,
            VfoOp::FromMemory => vfo_op_t_RIG_OP_TO_VFO,
            VfoOp::MemClear => vfo_op_t_RIG_OP_MCL,
            VfoOp::StepUp => vfo_op_t_RIG_OP_UP,
            VfoOp::StepDown => vfo_op_t_RIG_OP_DOWN,
            VfoOp::BandUp => vfo_op_t_RIG_OP_BAND_UP,
            VfoOp::BandDown => vfo_op_t_RIG_OP_BAND_DOWN,
            VfoOp::TuneLeft => vfo_op_t_RIG_OP_LEFT,
            VfoOp::TuneRight => vfo_op_t_RIG_OP_RIGHT,
            VfoOp::AutoTune => vfo_op_t_RIG_OP_TUNE,
            VfoOp::Toggle => vfo_op_t_RIG_OP_TOGGLE,
            VfoOp::Other(v) => v,
        }
    }

    fn scanop_to_hamlib(op: ScanOp) -> scan_t {
        match op {
            ScanOp::Stop => scan_t_RIG_SCAN_STOP,
            ScanOp::Memory => scan_t_RIG_SCAN_MEM,
            ScanOp::Selected => scan_t_RIG_SCAN_SLCT,
            ScanOp::Priority => scan_t_RIG_SCAN_PRIO,
            ScanOp::Program => scan_t_RIG_SCAN_PROG,
            ScanOp::Delta => scan_t_RIG_SCAN_DELTA,
            ScanOp::Vfo => scan_t_RIG_SCAN_VFO,
            ScanOp::Plt => scan_t_RIG_SCAN_PLT,
            ScanOp::Other(v) => v,
        }
    }
}

// ── Capability tables ─────────────────────────────────────────────────────────
// Parallel to VFO_TABLE: maps bitmask → enum variant for each category.

const LEVEL_TABLE: &[(setting_t, Level)] = &[
    (LVL_PREAMP, Level::Preamp),
    (LVL_ATT, Level::Attenuation),
    (LVL_VOXDELAY, Level::VoxDelay),
    (LVL_AF, Level::AfGain),
    (LVL_RF, Level::RfGain),
    (LVL_SQL, Level::Squelch),
    (LVL_IF, Level::IfShift),
    (LVL_APF, Level::AudioPeakFilter),
    (LVL_NR, Level::NoiseReduction),
    (LVL_PBT_IN, Level::PbtIn),
    (LVL_PBT_OUT, Level::PbtOut),
    (LVL_CWPITCH, Level::CwPitch),
    (LVL_RFPOWER, Level::RfPower),
    (LVL_MICGAIN, Level::MicGain),
    (LVL_KEYSPD, Level::KeySpeed),
    (LVL_NOTCHF, Level::NotchFreq),
    (LVL_COMP, Level::Comp),
    (LVL_AGC, Level::Agc),
    (LVL_BKINDL, Level::BkInDelay),
    (LVL_BALANCE, Level::Balance),
    (LVL_VOXGAIN, Level::VoxGain),
    (LVL_ANTIVOX, Level::AntiVox),
    (LVL_SLOPE_LOW, Level::SlopeLow),
    (LVL_SLOPE_HIGH, Level::SlopeHigh),
    (LVL_BKIN_DLYMS, Level::BkInDelayMs),
    (LVL_SWR, Level::Swr),
    (LVL_ALC, Level::Alc),
    (LVL_STRENGTH, Level::Strength),
    (LVL_RFPOWER_METER, Level::RfPowerMeter),
    (LVL_COMP_METER, Level::CompMeter),
    (LVL_VD_METER, Level::VdMeter),
    (LVL_ID_METER, Level::IdMeter),
    (LVL_NOTCHF_RAW, Level::NotchFreqRaw),
    (LVL_MONITOR_GAIN, Level::MonitorGain),
    (LVL_NB, Level::NoiseBlanker),
    (LVL_RFPOWER_METER_WATTS, Level::RfPowerWatts),
    (LVL_TEMP_METER, Level::TempMeter),
];

const FUNC_TABLE: &[(setting_t, Func)] = &[
    (FUNC_FAGC, Func::FastAgc),
    (FUNC_NB, Func::NoiseBlanker),
    (FUNC_COMP, Func::Comp),
    (FUNC_VOX, Func::Vox),
    (FUNC_TONE, Func::CtcssTone),
    (FUNC_TSQL, Func::CtcssSql),
    (FUNC_SBKIN, Func::SemiBreakIn),
    (FUNC_FBKIN, Func::FullBreakIn),
    (FUNC_ANF, Func::AutoNotch),
    (FUNC_NR, Func::NoiseReduction),
    (FUNC_AIP, Func::Aip),
    (FUNC_APF, Func::AutoPeakFilter),
    (FUNC_MON, Func::Monitor),
    (FUNC_MN, Func::ManualNotch),
    (FUNC_RF, Func::RttyFilter),
    (FUNC_ARO, Func::AutoRptrOffset),
    (FUNC_LOCK, Func::Lock),
    (FUNC_MUTE, Func::Mute),
    (FUNC_VSC, Func::VoiceScan),
    (FUNC_REV, Func::Reverse),
    (FUNC_SQL, Func::Squelch),
    (FUNC_ABM, Func::AutoBandMode),
    (FUNC_BC, Func::BeatCancel),
    (FUNC_MBC, Func::ManualBeatCancel),
    (FUNC_RIT, Func::Rit),
    (FUNC_AFC, Func::Afc),
    (FUNC_SATMODE, Func::SatMode),
    (FUNC_SCOPE, Func::Scope),
    (FUNC_RESUME, Func::ScanResume),
    (FUNC_TBURST, Func::ToneBurst),
    (FUNC_TUNER, Func::Tuner),
    (FUNC_XIT, Func::Xit),
    (FUNC_NB2, Func::NoiseBlanker2),
    (FUNC_CSQL, Func::DcsSql),
    (FUNC_AFLT, Func::AfFilter),
    (FUNC_ANL, Func::NoiseLimiter),
    (FUNC_BC2, Func::BeatCancel2),
    (FUNC_DUAL_WATCH, Func::DualWatch),
    (FUNC_DIVERSITY, Func::Diversity),
];

const MODE_TABLE: &[(rmode_t, Mode)] = &[
    (MODE_AM, Mode::Am),
    (MODE_CW, Mode::Cw),
    (MODE_CWR, Mode::CwR),
    (MODE_USB, Mode::Usb),
    (MODE_LSB, Mode::Lsb),
    (MODE_FM, Mode::Fm),
    (MODE_WFM, Mode::Wfm),
    (MODE_RTTY, Mode::Rtty),
    (MODE_RTTYR, Mode::RttyR),
    (MODE_PKTUSB, Mode::PktUsb),
    (MODE_PKTLSB, Mode::PktLsb),
    (MODE_PKTFM, Mode::PktFm),
];

const VFOOP_TABLE: &[(vfo_op_t, VfoOp)] = &[
    (vfo_op_t_RIG_OP_CPY, VfoOp::CopyAToB),
    (vfo_op_t_RIG_OP_XCHG, VfoOp::Exchange),
    (vfo_op_t_RIG_OP_FROM_VFO, VfoOp::ToMemory),
    (vfo_op_t_RIG_OP_TO_VFO, VfoOp::FromMemory),
    (vfo_op_t_RIG_OP_MCL, VfoOp::MemClear),
    (vfo_op_t_RIG_OP_UP, VfoOp::StepUp),
    (vfo_op_t_RIG_OP_DOWN, VfoOp::StepDown),
    (vfo_op_t_RIG_OP_BAND_UP, VfoOp::BandUp),
    (vfo_op_t_RIG_OP_BAND_DOWN, VfoOp::BandDown),
    (vfo_op_t_RIG_OP_LEFT, VfoOp::TuneLeft),
    (vfo_op_t_RIG_OP_RIGHT, VfoOp::TuneRight),
    (vfo_op_t_RIG_OP_TUNE, VfoOp::AutoTune),
    (vfo_op_t_RIG_OP_TOGGLE, VfoOp::Toggle),
];

const SCANOP_TABLE: &[(scan_t, ScanOp)] = &[
    (scan_t_RIG_SCAN_STOP, ScanOp::Stop),
    (scan_t_RIG_SCAN_MEM, ScanOp::Memory),
    (scan_t_RIG_SCAN_SLCT, ScanOp::Selected),
    (scan_t_RIG_SCAN_PRIO, ScanOp::Priority),
    (scan_t_RIG_SCAN_PROG, ScanOp::Program),
    (scan_t_RIG_SCAN_DELTA, ScanOp::Delta),
    (scan_t_RIG_SCAN_VFO, ScanOp::Vfo),
    (scan_t_RIG_SCAN_PLT, ScanOp::Plt),
];

impl Drop for HamlibTransceiver {
    fn drop(&mut self) {
        unsafe {
            rig_close(self.rig);
            rig_cleanup(self.rig);
        }
    }
}

impl Transceiver for HamlibTransceiver {
    // ── VFO discovery ─────────────────────────────────────────────────────────

    fn vfo_list(&mut self) -> Result<Vec<Vfo>> {
        #[cfg(not(target_os = "android"))]
        {
            // vfo_list is a bitmask of supported VFOs stored in rig state.
            // Cast to u32 so high bits (like VFO_CURR at bit 29) are unsigned.
            let supported = unsafe { (*self.rig).state.vfo_list as u32 };
            let mut list: Vec<Vfo> = VFO_TABLE
                .iter()
                .filter(|(mask, _)| supported & mask != 0)
                .map(|(mask, name)| Vfo::new(*name, *mask as u64))
                .collect();
            let curr_mask: u32 = 1 << 29;
            if supported & curr_mask == 0 {
                list.push(Vfo::new("Current", curr_mask as u64));
            }
            Ok(list)
        }
        #[cfg(target_os = "android")]
        {
            // Newer hamlib API: rig_state is opaque; use rig_get_vfo_list
            // which returns a space-separated string of hamlib VFO names
            // (e.g. "VFOA VFOB VFOCurrent"). Parse each name through
            // rig_parse_vfo to get the bitmask, then map to our canonical Vfo.
            // Use c_char so the type matches on both Linux (i8) and Android (u8).
            let mut buf = vec![0 as std::os::raw::c_char; 256];
            unsafe { rig_get_vfo_list(self.rig, buf.as_mut_ptr(), buf.len() as i32) };
            let s = unsafe { CStr::from_ptr(buf.as_ptr()) }
                .to_string_lossy()
                .into_owned();
            let mut list: Vec<Vfo> = s
                .split_whitespace()
                .filter_map(|name| {
                    CString::new(name).ok().and_then(|cn| {
                        let id = unsafe { rig_parse_vfo(cn.as_ptr()) };
                        if id != 0 { Some(Self::vfo_from_id(id)) } else { None }
                    })
                })
                .collect();
            if !list.iter().any(|v| v.name() == "Current") {
                list.push(Vfo::new("Current", (1u64 << 29)));
            }
            Ok(list)
        }
    }

    /// Searches the vfo_list by name (case-insensitive) and also tries
    /// hamlib's `rig_parse_vfo` to handle hamlib-specific aliases such as
    /// `"VFOA"` in addition to the canonical `"A"`.
    fn vfo_by_name(&mut self, name: &str) -> Result<Option<Vfo>> {
        // Try our canonical name table first.
        if let Some(vfo) = self
            .vfo_list()?
            .into_iter()
            .find(|v| v.name().eq_ignore_ascii_case(name))
        {
            return Ok(Some(vfo));
        }

        // Fall back to hamlib's own alias parser (handles "VFOA", "currVFO", etc.)
        if let Ok(cname) = CString::new(name) {
            let id = unsafe { rig_parse_vfo(cname.as_ptr()) };
            if id != 0 {
                return Ok(Some(Self::vfo_from_id(id)));
            }
        }

        Ok(None)
    }

    // ── Capability discovery ──────────────────────────────────────────────────

    fn supported_modes(&mut self) -> Result<Vec<Mode>> {
        // Union the mode bitmasks across all RX frequency range entries.
        // Older API: ranges are in rig_state; newer API: in rig_caps.
        let combined: rmode_t = unsafe {
            #[cfg(not(target_os = "android"))]
            { (*self.rig).state.rx_range_list.iter().take_while(|r| r.startf != 0.0).fold(0, |acc, r| acc | r.modes) }
            #[cfg(target_os = "android")]
            { (*(*self.rig).caps).rx_range_list1.iter().take_while(|r| r.startf != 0.0).fold(0, |acc, r| acc | r.modes) }
        };
        Ok(MODE_TABLE
            .iter()
            .filter(|(mask, _)| combined & mask != 0)
            .map(|(_, mode)| *mode)
            .collect())
    }

    fn supported_levels(&mut self) -> Result<Vec<Level>> {
        let combined = unsafe {
            #[cfg(not(target_os = "android"))]
            { (*self.rig).state.has_get_level | (*self.rig).state.has_set_level }
            #[cfg(target_os = "android")]
            { (*(*self.rig).caps).has_get_level | (*(*self.rig).caps).has_set_level }
        };
        Ok(LEVEL_TABLE
            .iter()
            .filter(|(mask, _)| combined & mask != 0)
            .map(|(_, level)| *level)
            .collect())
    }

    fn supported_funcs(&mut self) -> Result<Vec<Func>> {
        let combined = unsafe {
            #[cfg(not(target_os = "android"))]
            { (*self.rig).state.has_get_func | (*self.rig).state.has_set_func }
            #[cfg(target_os = "android")]
            { (*(*self.rig).caps).has_get_func | (*(*self.rig).caps).has_set_func }
        };
        Ok(FUNC_TABLE
            .iter()
            .filter(|(mask, _)| combined & mask != 0)
            .map(|(_, func)| *func)
            .collect())
    }

    fn supported_parms(&mut self) -> Result<Vec<Parm>> {
        let combined = unsafe {
            #[cfg(not(target_os = "android"))]
            { ((*self.rig).state.has_get_parm | (*self.rig).state.has_set_parm) as u64 }
            #[cfg(target_os = "android")]
            { ((*(*self.rig).caps).has_get_parm | (*(*self.rig).caps).has_set_parm) as u64 }
        };
        let parm_table: &[(u64, Parm)] = &[
            (rig_parm_e_RIG_PARM_ANN as u64, Parm::Announce),
            (rig_parm_e_RIG_PARM_APO as u64, Parm::AutoPowerOff),
            (rig_parm_e_RIG_PARM_BACKLIGHT as u64, Parm::Backlight),
            (rig_parm_e_RIG_PARM_BEEP as u64, Parm::Beep),
            (rig_parm_e_RIG_PARM_TIME as u64, Parm::Time),
            (rig_parm_e_RIG_PARM_BAT as u64, Parm::Battery),
            (rig_parm_e_RIG_PARM_KEYLIGHT as u64, Parm::KeyLight),
            (rig_parm_e_RIG_PARM_SCREENSAVER as u64, Parm::ScreenSaver),
            (rig_parm_e_RIG_PARM_AFIF as u64, Parm::AfIf),
        ];
        Ok(parm_table
            .iter()
            .filter(|(mask, _)| combined & mask != 0)
            .map(|(_, parm)| *parm)
            .collect())
    }

    fn supported_vfo_ops(&mut self) -> Result<Vec<VfoOp>> {
        let caps_ops = unsafe { (*(*self.rig).caps).vfo_ops };
        Ok(VFOOP_TABLE
            .iter()
            .filter(|(mask, _)| caps_ops & mask != 0)
            .map(|(_, op)| *op)
            .collect())
    }

    fn supported_scan_ops(&mut self) -> Result<Vec<ScanOp>> {
        let caps_scan = unsafe { (*(*self.rig).caps).scan_ops };
        Ok(SCANOP_TABLE
            .iter()
            .filter(|(mask, _)| caps_scan & mask != 0)
            .map(|(_, op)| *op)
            .collect())
    }

    // ── Core ──────────────────────────────────────────────────────────────────

    fn get_freq(&mut self, vfo: &Vfo) -> Result<Frequency> {
        let mut freq: freq_t = 0.0;
        unsafe {
            Self::hamlib_check(rig_get_freq(self.rig, vfo.id() as vfo_t, &mut freq))?;
        }
        Ok(Frequency(freq as u64))
    }

    fn set_freq(&mut self, vfo: &Vfo, freq: Frequency) -> Result<()> {
        unsafe {
            Self::hamlib_check(rig_set_freq(
                self.rig,
                vfo.id() as vfo_t,
                freq.hz() as freq_t,
            ))
        }
    }

    fn get_mode(&mut self, vfo: &Vfo) -> Result<(Mode, Passband)> {
        let mut mode: rmode_t = 0;
        let mut width: pbwidth_t = 0;
        unsafe {
            Self::hamlib_check(rig_get_mode(
                self.rig,
                vfo.id() as vfo_t,
                &mut mode,
                &mut width,
            ))?;
        }
        let passband = if width > 0 { Some(width as u32) } else { None };
        Ok((Self::mode_from_hamlib(mode), passband))
    }

    fn set_mode(&mut self, vfo: &Vfo, mode: Mode, passband: Passband) -> Result<()> {
        let width: pbwidth_t = passband.map(|w| w as pbwidth_t).unwrap_or(0);
        unsafe {
            Self::hamlib_check(rig_set_mode(
                self.rig,
                vfo.id() as vfo_t,
                Self::mode_to_hamlib(mode),
                width,
            ))
        }
    }

    fn get_ptt(&mut self) -> Result<Ptt> {
        let curr_vfo: vfo_t = 1 << 29; // VFO_CURR
        let mut ptt: ptt_t = 0;
        unsafe {
            Self::hamlib_check(rig_get_ptt(self.rig, curr_vfo, &mut ptt))?;
        }
        Ok(if ptt == ptt_t_RIG_PTT_OFF { Ptt::Rx } else { Ptt::Tx })
    }

    fn set_ptt(&mut self, ptt: Ptt) -> Result<()> {
        let curr_vfo: vfo_t = 1 << 29;
        let val: ptt_t = match ptt {
            Ptt::Rx => ptt_t_RIG_PTT_OFF,
            Ptt::Tx => ptt_t_RIG_PTT_ON,
        };
        unsafe { Self::hamlib_check(rig_set_ptt(self.rig, curr_vfo, val)) }
    }

    // ── Levels ────────────────────────────────────────────────────────────────

    fn get_level(&mut self, vfo: &Vfo, level: Level) -> Result<LevelValue> {
        let mut val: value_t = unsafe { std::mem::zeroed() };
        unsafe {
            Self::hamlib_check(rig_get_level(
                self.rig,
                vfo.id() as vfo_t,
                Self::level_to_setting(&level),
                &mut val,
            ))?;
            if Self::level_is_float(&level) {
                Ok(LevelValue::Float(val.f))
            } else {
                Ok(LevelValue::Int(val.i))
            }
        }
    }

    fn set_level(&mut self, vfo: &Vfo, level: Level, value: LevelValue) -> Result<()> {
        let val: value_t = match value {
            LevelValue::Float(f) => value_t { f },
            LevelValue::Int(i) => value_t { i },
        };
        unsafe {
            Self::hamlib_check(rig_set_level(
                self.rig,
                vfo.id() as vfo_t,
                Self::level_to_setting(&level),
                val,
            ))
        }
    }

    // ── Functions ─────────────────────────────────────────────────────────────

    fn get_func(&mut self, vfo: &Vfo, func: Func) -> Result<bool> {
        let mut status: std::os::raw::c_int = 0;
        unsafe {
            Self::hamlib_check(rig_get_func(
                self.rig,
                vfo.id() as vfo_t,
                Self::func_to_setting(&func),
                &mut status,
            ))?;
        }
        Ok(status != 0)
    }

    fn set_func(&mut self, vfo: &Vfo, func: Func, enabled: bool) -> Result<()> {
        unsafe {
            Self::hamlib_check(rig_set_func(
                self.rig,
                vfo.id() as vfo_t,
                Self::func_to_setting(&func),
                enabled as std::os::raw::c_int,
            ))
        }
    }

    // ── Parameters ────────────────────────────────────────────────────────────

    fn get_parm(&mut self, parm: Parm) -> Result<LevelValue> {
        let mut val: value_t = unsafe { std::mem::zeroed() };
        unsafe {
            Self::hamlib_check(rig_get_parm(
                self.rig,
                Self::parm_to_setting(&parm),
                &mut val,
            ))?;
        }
        let is_float = matches!(parm, Parm::Backlight | Parm::Battery | Parm::KeyLight);
        Ok(if is_float {
            LevelValue::Float(unsafe { val.f })
        } else {
            LevelValue::Int(unsafe { val.i })
        })
    }

    fn set_parm(&mut self, parm: Parm, value: LevelValue) -> Result<()> {
        let val: value_t = match value {
            LevelValue::Float(f) => value_t { f },
            LevelValue::Int(i) => value_t { i },
        };
        unsafe {
            Self::hamlib_check(rig_set_parm(self.rig, Self::parm_to_setting(&parm), val))
        }
    }

    // ── RIT / XIT ─────────────────────────────────────────────────────────────

    fn get_rit(&mut self, vfo: &Vfo) -> Result<i64> {
        let mut rit: shortfreq_t = 0;
        unsafe {
            Self::hamlib_check(rig_get_rit(self.rig, vfo.id() as vfo_t, &mut rit))?;
        }
        Ok(rit as i64)
    }

    fn set_rit(&mut self, vfo: &Vfo, hz: i64) -> Result<()> {
        unsafe {
            Self::hamlib_check(rig_set_rit(self.rig, vfo.id() as vfo_t, hz as shortfreq_t))
        }
    }

    fn get_xit(&mut self, vfo: &Vfo) -> Result<i64> {
        let mut xit: shortfreq_t = 0;
        unsafe {
            Self::hamlib_check(rig_get_xit(self.rig, vfo.id() as vfo_t, &mut xit))?;
        }
        Ok(xit as i64)
    }

    fn set_xit(&mut self, vfo: &Vfo, hz: i64) -> Result<()> {
        unsafe {
            Self::hamlib_check(rig_set_xit(self.rig, vfo.id() as vfo_t, hz as shortfreq_t))
        }
    }

    // ── Split ─────────────────────────────────────────────────────────────────

    fn get_split_vfo(&mut self, rx_vfo: &Vfo) -> Result<(bool, Vfo)> {
        let mut split: split_t = 0;
        let mut tx_vfo_id: vfo_t = 0;
        unsafe {
            Self::hamlib_check(rig_get_split_vfo(
                self.rig,
                rx_vfo.id() as vfo_t,
                &mut split,
                &mut tx_vfo_id,
            ))?;
        }
        Ok((split != 0, Self::vfo_from_id(tx_vfo_id)))
    }

    fn set_split_vfo(&mut self, rx_vfo: &Vfo, split: bool, tx_vfo: &Vfo) -> Result<()> {
        unsafe {
            Self::hamlib_check(rig_set_split_vfo(
                self.rig,
                rx_vfo.id() as vfo_t,
                split as split_t,
                tx_vfo.id() as vfo_t,
            ))
        }
    }

    fn get_split_freq(&mut self, tx_vfo: &Vfo) -> Result<Frequency> {
        let mut freq: freq_t = 0.0;
        unsafe {
            Self::hamlib_check(rig_get_split_freq(
                self.rig,
                tx_vfo.id() as vfo_t,
                &mut freq,
            ))?;
        }
        Ok(Frequency(freq as u64))
    }

    fn set_split_freq(&mut self, tx_vfo: &Vfo, freq: Frequency) -> Result<()> {
        unsafe {
            Self::hamlib_check(rig_set_split_freq(
                self.rig,
                tx_vfo.id() as vfo_t,
                freq.hz() as freq_t,
            ))
        }
    }

    fn get_split_mode(&mut self, tx_vfo: &Vfo) -> Result<(Mode, Passband)> {
        let mut mode: rmode_t = 0;
        let mut width: pbwidth_t = 0;
        unsafe {
            Self::hamlib_check(rig_get_split_mode(
                self.rig,
                tx_vfo.id() as vfo_t,
                &mut mode,
                &mut width,
            ))?;
        }
        let passband = if width > 0 { Some(width as u32) } else { None };
        Ok((Self::mode_from_hamlib(mode), passband))
    }

    fn set_split_mode(&mut self, tx_vfo: &Vfo, mode: Mode, passband: Passband) -> Result<()> {
        let width: pbwidth_t = passband.map(|w| w as pbwidth_t).unwrap_or(0);
        unsafe {
            Self::hamlib_check(rig_set_split_mode(
                self.rig,
                tx_vfo.id() as vfo_t,
                Self::mode_to_hamlib(mode),
                width,
            ))
        }
    }

    // ── CTCSS / DCS ───────────────────────────────────────────────────────────

    fn get_ctcss_tone(&mut self, vfo: &Vfo) -> Result<u32> {
        let mut tone: tone_t = 0;
        unsafe {
            Self::hamlib_check(rig_get_ctcss_tone(self.rig, vfo.id() as vfo_t, &mut tone))?;
        }
        Ok(tone)
    }

    fn set_ctcss_tone(&mut self, vfo: &Vfo, tone: u32) -> Result<()> {
        unsafe {
            Self::hamlib_check(rig_set_ctcss_tone(self.rig, vfo.id() as vfo_t, tone as tone_t))
        }
    }

    fn get_ctcss_sql(&mut self, vfo: &Vfo) -> Result<u32> {
        let mut tone: tone_t = 0;
        unsafe {
            Self::hamlib_check(rig_get_ctcss_sql(self.rig, vfo.id() as vfo_t, &mut tone))?;
        }
        Ok(tone)
    }

    fn set_ctcss_sql(&mut self, vfo: &Vfo, tone: u32) -> Result<()> {
        unsafe {
            Self::hamlib_check(rig_set_ctcss_sql(self.rig, vfo.id() as vfo_t, tone as tone_t))
        }
    }

    fn get_dcs_code(&mut self, vfo: &Vfo) -> Result<u32> {
        let mut code: tone_t = 0;
        unsafe {
            Self::hamlib_check(rig_get_dcs_code(self.rig, vfo.id() as vfo_t, &mut code))?;
        }
        Ok(code)
    }

    fn set_dcs_code(&mut self, vfo: &Vfo, code: u32) -> Result<()> {
        unsafe {
            Self::hamlib_check(rig_set_dcs_code(self.rig, vfo.id() as vfo_t, code as tone_t))
        }
    }

    fn get_dcs_sql(&mut self, vfo: &Vfo) -> Result<u32> {
        let mut code: tone_t = 0;
        unsafe {
            Self::hamlib_check(rig_get_dcs_sql(self.rig, vfo.id() as vfo_t, &mut code))?;
        }
        Ok(code)
    }

    fn set_dcs_sql(&mut self, vfo: &Vfo, code: u32) -> Result<()> {
        unsafe {
            Self::hamlib_check(rig_set_dcs_sql(self.rig, vfo.id() as vfo_t, code as tone_t))
        }
    }

    // ── Repeater ──────────────────────────────────────────────────────────────

    fn get_rptr_shift(&mut self, vfo: &Vfo) -> Result<RptrShift> {
        let mut shift: rptr_shift_t = 0;
        unsafe {
            Self::hamlib_check(rig_get_rptr_shift(self.rig, vfo.id() as vfo_t, &mut shift))?;
        }
        Ok(match shift {
            rptr_shift_t_RIG_RPT_SHIFT_MINUS => RptrShift::Minus,
            rptr_shift_t_RIG_RPT_SHIFT_PLUS => RptrShift::Plus,
            _ => RptrShift::None,
        })
    }

    fn set_rptr_shift(&mut self, vfo: &Vfo, shift: RptrShift) -> Result<()> {
        let s = match shift {
            RptrShift::None => rptr_shift_t_RIG_RPT_SHIFT_NONE,
            RptrShift::Minus => rptr_shift_t_RIG_RPT_SHIFT_MINUS,
            RptrShift::Plus => rptr_shift_t_RIG_RPT_SHIFT_PLUS,
        };
        unsafe { Self::hamlib_check(rig_set_rptr_shift(self.rig, vfo.id() as vfo_t, s)) }
    }

    fn get_rptr_offs(&mut self, vfo: &Vfo) -> Result<i64> {
        let mut offs: shortfreq_t = 0;
        unsafe {
            Self::hamlib_check(rig_get_rptr_offs(self.rig, vfo.id() as vfo_t, &mut offs))?;
        }
        Ok(offs as i64)
    }

    fn set_rptr_offs(&mut self, vfo: &Vfo, offset: i64) -> Result<()> {
        unsafe {
            Self::hamlib_check(rig_set_rptr_offs(
                self.rig,
                vfo.id() as vfo_t,
                offset as shortfreq_t,
            ))
        }
    }

    // ── Tuning step ───────────────────────────────────────────────────────────

    fn get_ts(&mut self, vfo: &Vfo) -> Result<u64> {
        let mut ts: shortfreq_t = 0;
        unsafe {
            Self::hamlib_check(rig_get_ts(self.rig, vfo.id() as vfo_t, &mut ts))?;
        }
        Ok(ts as u64)
    }

    fn set_ts(&mut self, vfo: &Vfo, step: u64) -> Result<()> {
        unsafe {
            Self::hamlib_check(rig_set_ts(self.rig, vfo.id() as vfo_t, step as shortfreq_t))
        }
    }

    // ── Antenna ───────────────────────────────────────────────────────────────

    fn get_ant(&mut self, vfo: &Vfo) -> Result<u32> {
        let mut option: value_t = unsafe { std::mem::zeroed() };
        let mut ant_curr: ant_t = 0;
        let mut ant_tx: ant_t = 0;
        let mut ant_rx: ant_t = 0;
        unsafe {
            Self::hamlib_check(rig_get_ant(
                self.rig,
                vfo.id() as vfo_t,
                ANT_CURR,
                &mut option,
                &mut ant_curr,
                &mut ant_tx,
                &mut ant_rx,
            ))?;
        }
        Ok(ant_curr)
    }

    fn set_ant(&mut self, vfo: &Vfo, ant: u32) -> Result<()> {
        let option: value_t = value_t { i: 0 };
        unsafe {
            Self::hamlib_check(rig_set_ant(
                self.rig,
                vfo.id() as vfo_t,
                ant as ant_t,
                option,
            ))
        }
    }

    // ── VFO operations ────────────────────────────────────────────────────────

    fn vfo_op(&mut self, vfo: &Vfo, op: VfoOp) -> Result<()> {
        unsafe {
            Self::hamlib_check(rig_vfo_op(
                self.rig,
                vfo.id() as vfo_t,
                Self::vfoop_to_hamlib(op),
            ))
        }
    }

    // ── Scan ─────────────────────────────────────────────────────────────────

    fn scan(&mut self, vfo: &Vfo, op: ScanOp, ch: i32) -> Result<()> {
        unsafe {
            Self::hamlib_check(rig_scan(
                self.rig,
                vfo.id() as vfo_t,
                Self::scanop_to_hamlib(op),
                ch,
            ))
        }
    }

    // ── Power ─────────────────────────────────────────────────────────────────

    fn get_powerstat(&mut self) -> Result<PowerState> {
        let mut status: powerstat_t = 0;
        unsafe {
            Self::hamlib_check(rig_get_powerstat(self.rig, &mut status))?;
        }
        Ok(match status {
            powerstat_t_RIG_POWER_OFF => PowerState::Off,
            powerstat_t_RIG_POWER_ON => PowerState::On,
            powerstat_t_RIG_POWER_STANDBY => PowerState::Standby,
            powerstat_t_RIG_POWER_OPERATE => PowerState::Operate,
            v => PowerState::Other(v),
        })
    }

    fn set_powerstat(&mut self, state: PowerState) -> Result<()> {
        let s = match state {
            PowerState::Off => powerstat_t_RIG_POWER_OFF,
            PowerState::On => powerstat_t_RIG_POWER_ON,
            PowerState::Standby => powerstat_t_RIG_POWER_STANDBY,
            PowerState::Operate => powerstat_t_RIG_POWER_OPERATE,
            PowerState::Other(v) => v,
        };
        unsafe { Self::hamlib_check(rig_set_powerstat(self.rig, s)) }
    }

    // ── Miscellaneous ─────────────────────────────────────────────────────────

    fn get_dcd(&mut self, vfo: &Vfo) -> Result<Dcd> {
        let mut dcd: dcd_t = dcd_e_RIG_DCD_OFF;
        unsafe {
            Self::hamlib_check(rig_get_dcd(self.rig, vfo.id() as vfo_t, &mut dcd))?;
        }
        Ok(if dcd == dcd_e_RIG_DCD_ON { Dcd::On } else { Dcd::Off })
    }

    fn get_info(&mut self) -> Result<String> {
        let ptr = unsafe { rig_get_info(self.rig) };
        if ptr.is_null() {
            return Ok(String::new());
        }
        Ok(unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned())
    }

    fn start_spectrum(
        &mut self,
        config: crate::spectrum::SpectrumConfig,
    ) -> Result<crate::spectrum::SpectrumStream> {
        use crate::spectrum::SpectrumSource;
        SpectrumSource::start(self, config)
    }
}

// ── SpectrumSource for HamlibTransceiver ──────────────────────────────────────

use crate::spectrum::{SpectrumConfig, SpectrumSource, SpectrumStream};

impl SpectrumSource for HamlibTransceiver {
    /// Start a spectrum stream via the hamlib 4.x scope API.
    ///
    /// Currently returns [`Error::NotSupported`] on all radios; the hamlib
    /// spectrum callback wiring (`rig_get_spectrum_line` / `spectrum_event`)
    /// will be implemented here for supported models (IC-7300, IC-7610,
    /// IC-9700, IC-705, …) once the CI-V spectrum integration is complete.
    fn start(&mut self, _config: SpectrumConfig) -> Result<SpectrumStream> {
        Err(Error::NotSupported)
    }
}

