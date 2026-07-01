/// Spectrum / waterfall data abstraction.
///
/// A [`SpectrumSource`] produces a stream of [`SpectrumLine`] values —
/// horizontal slices of a waterfall, one per sweep.  The trait is intentionally
/// decoupled from [`Transceiver`]: the source might be a hamlib rig with a
/// built-in scope, an SDR dongle, an audio-FFT pipeline, or a test fixture.
///
/// # Producing a stream
///
/// ```no_run
/// # use quirq_core::spectrum::{SpectrumSource, SpectrumConfig};
/// # fn example(mut src: impl SpectrumSource) -> quirq_core::error::Result<()> {
/// let stream = src.start(SpectrumConfig::new(7_200_000, 200_000))?;
/// loop {
///     if let Some(line) = stream.try_next() {
///         println!("{} bins across {} Hz", line.amplitudes_dbfs.len(), line.span_hz);
///     }
/// }
/// # }
/// ```
///
/// [`Transceiver`]: crate::transceiver::Transceiver
use std::sync::mpsc;
use std::time::Instant;

use crate::error::{Error, Result};

// ── SpectrumLine ──────────────────────────────────────────────────────────────

/// A single horizontal sweep of spectrum data — one row of the waterfall.
///
/// The `amplitudes_dbfs` slice spans `[low_edge_hz(), high_edge_hz()]`
/// uniformly; index 0 is the lowest frequency.
#[derive(Debug, Clone)]
pub struct SpectrumLine {
    /// Center of the displayed span in Hz.
    pub center_hz: u64,
    /// Width of the displayed span in Hz.
    pub span_hz: u64,
    /// Power in each frequency bin, in dBFS.
    ///
    /// Values are typically in the range −160 dBFS (noise floor) to 0 dBFS
    /// (clipping).  Higher values indicate stronger signals.  The number of
    /// bins is determined by the implementation and may differ from the
    /// requested [`SpectrumConfig::bins`].
    pub amplitudes_dbfs: Vec<f32>,
    /// Monotonic timestamp at the moment this sweep was captured.
    pub captured_at: Instant,
}

impl SpectrumLine {
    /// Lowest frequency covered by this line, in Hz.
    pub fn low_edge_hz(&self) -> u64 {
        self.center_hz.saturating_sub(self.span_hz / 2)
    }

    /// Highest frequency covered by this line, in Hz.
    pub fn high_edge_hz(&self) -> u64 {
        self.center_hz.saturating_add(self.span_hz / 2)
    }

    /// Hertz per amplitude bin.
    pub fn hz_per_bin(&self) -> f64 {
        if self.amplitudes_dbfs.is_empty() {
            return 0.0;
        }
        self.span_hz as f64 / self.amplitudes_dbfs.len() as f64
    }

    /// Returns the frequency (Hz) at the centre of bin `idx`.
    pub fn bin_center_hz(&self, idx: usize) -> f64 {
        self.low_edge_hz() as f64 + (idx as f64 + 0.5) * self.hz_per_bin()
    }
}

// ── SpectrumStream ────────────────────────────────────────────────────────────

/// The receiving end of a live spectrum stream, produced by
/// [`SpectrumSource::start`].
///
/// When this value is dropped the producer thread detects the closed channel
/// and stops generating data.  Callers do not need to call any explicit
/// stop method.
pub struct SpectrumStream {
    rx: mpsc::Receiver<SpectrumLine>,
}

impl SpectrumStream {
    /// Called by [`SpectrumSource`] implementations only.
    pub fn from_receiver(rx: mpsc::Receiver<SpectrumLine>) -> Self {
        Self { rx }
    }

    /// Returns the next line immediately if one is ready, or `None` if the
    /// producer hasn't produced a new sweep yet.  Never blocks.
    pub fn try_next(&self) -> Option<SpectrumLine> {
        self.rx.try_recv().ok()
    }

    /// Blocks until the next line arrives or `timeout` elapses.
    ///
    /// Returns `None` on timeout; the stream remains open.
    pub fn next_timeout(&self, timeout: std::time::Duration) -> Option<SpectrumLine> {
        self.rx.recv_timeout(timeout).ok()
    }

    /// Drains all lines that are currently queued without blocking.
    ///
    /// Useful when the consumer is slower than the producer and only needs
    /// the most recent sweep: call `drain().last()`.
    pub fn drain(&self) -> impl Iterator<Item = SpectrumLine> + '_ {
        std::iter::from_fn(|| self.rx.try_recv().ok())
    }
}

// ── SpectrumConfig ────────────────────────────────────────────────────────────

/// Parameters for opening a spectrum stream.
#[derive(Debug, Clone)]
pub struct SpectrumConfig {
    /// Desired center frequency in Hz.
    ///
    /// Implementations may round to the nearest supported value.
    pub center_hz: u64,
    /// Desired frequency span in Hz.
    ///
    /// Implementations may clamp to their supported range.
    pub span_hz: u64,
    /// Desired number of amplitude bins per sweep.
    ///
    /// Implementations may return a different resolution; check
    /// [`SpectrumLine::amplitudes_dbfs`]`.len()` on the first received line.
    pub bins: usize,
}

impl SpectrumConfig {
    /// Construct a config with default bin count (1024).
    pub fn new(center_hz: u64, span_hz: u64) -> Self {
        Self { center_hz, span_hz, bins: 1024 }
    }

    /// Override the bin count.
    pub fn with_bins(mut self, bins: usize) -> Self {
        self.bins = bins;
        self
    }
}

// ── SpectrumSource ────────────────────────────────────────────────────────────

/// Anything that can produce a live stream of spectrum / waterfall data.
///
/// # Implementors (planned)
///
/// | Backend | Notes |
/// |---------|-------|
/// | `HamlibSpectrum` | Radios with built-in scope (IC-7300, IC-705, …) via hamlib 4.x |
/// | `AudioFftSource` | Audio output of any radio → FFT panadapter |
/// | `SoapySdrSource` | IQ samples from an SDR dongle → FFT |
/// | `MockSource` | Synthetic data for UI development and tests |
pub trait SpectrumSource: Send {
    /// Start streaming spectrum data with the given configuration.
    ///
    /// Returns a [`SpectrumStream`] that receives lines until it is dropped.
    /// Dropping the stream signals the implementation to stop producing data.
    ///
    /// Calling `start` again while a previous stream is still alive is
    /// implementation-defined: it may reconfigure the existing stream or
    /// return an error.
    fn start(&mut self, config: SpectrumConfig) -> Result<SpectrumStream>;

    /// Update the center frequency of a running stream without restarting it.
    ///
    /// Implementations that cannot retune in-flight should return
    /// [`Error::NotSupported`]; callers should then drop and re-`start`.
    fn retune(&mut self, _center_hz: u64) -> Result<()> {
        Err(Error::NotSupported)
    }
}

// ── MockSource ────────────────────────────────────────────────────────────────

/// A spectrum source that generates synthetic noise + a few sine-wave peaks.
///
/// Useful for UI development and integration tests without hardware.
pub struct MockSource {
    sweep_rate: std::time::Duration,
}

impl MockSource {
    /// Create a mock source producing sweeps at `sweeps_per_sec` lines/second.
    pub fn new(sweeps_per_sec: u32) -> Self {
        Self {
            sweep_rate: std::time::Duration::from_millis(
                1000 / sweeps_per_sec.max(1) as u64,
            ),
        }
    }
}

impl SpectrumSource for MockSource {
    fn start(&mut self, config: SpectrumConfig) -> Result<SpectrumStream> {
        let (tx, rx) = mpsc::channel();
        let sweep_rate = self.sweep_rate;
        let bins = config.bins;
        let center_hz = config.center_hz;
        let span_hz = config.span_hz;

        std::thread::Builder::new()
            .name("quirq-mock-spectrum".into())
            .spawn(move || {
                let mut t = 0.0f64;
                loop {
                    std::thread::sleep(sweep_rate);
                    let amplitudes = (0..bins)
                        .map(|i| {
                            let x = i as f64 / bins as f64; // 0..1 across span
                            // Noise floor around −100 dBFS with slight variation.
                            let noise = -100.0 + fastrand::f64() as f32 * 10.0;
                            // A few peaks drifting slowly across the band.
                            let p0 = peak(x, 0.25 + 0.05 * (t * 0.3).sin(), 0.01, 30.0);
                            let p1 = peak(x, 0.60 + 0.03 * (t * 0.7).cos(), 0.005, 20.0);
                            let p2 = peak(x, 0.80, 0.002, 15.0);
                            (noise + p0 + p1 + p2).min(0.0)
                        })
                        .collect();

                    let line = SpectrumLine {
                        center_hz,
                        span_hz,
                        amplitudes_dbfs: amplitudes,
                        captured_at: Instant::now(),
                    };

                    if tx.send(line).is_err() {
                        break; // receiver was dropped — stream closed
                    }
                    t += sweep_rate.as_secs_f64();
                }
            })
            .expect("failed to spawn mock spectrum thread");

        Ok(SpectrumStream::from_receiver(rx))
    }
}

/// Gaussian-shaped peak centred at `cx` (fraction of span) with half-width `w`
/// and height `amplitude_db`.
fn peak(x: f64, cx: f64, w: f64, amplitude_db: f32) -> f32 {
    let d = (x - cx) / w;
    ((-0.5 * d * d).exp() as f32) * amplitude_db
}
