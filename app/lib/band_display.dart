import 'dart:async';
import 'dart:collection';
import 'dart:math' as math;
import 'dart:typed_data';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';

// ── Band / mode utilities ─────────────────────────────────────────────────────

class _Band {
  final String name;
  final int low;
  final int high;
  const _Band(this.name, this.low, this.high);
}

// ITU Region 2 allocations used as fallback when hamlib data isn't wired yet.
const _kBands = [
  _Band('160m',  1_800_000,   2_000_000),
  _Band('80m',   3_500_000,   4_000_000),
  _Band('60m',   5_330_500,   5_406_500),
  _Band('40m',   7_000_000,   7_300_000),
  _Band('30m',  10_100_000,  10_150_000),
  _Band('20m',  14_000_000,  14_350_000),
  _Band('17m',  18_068_000,  18_168_000),
  _Band('15m',  21_000_000,  21_450_000),
  _Band('12m',  24_890_000,  24_990_000),
  _Band('10m',  28_000_000,  29_700_000),
  _Band('6m',   50_000_000,  54_000_000),
  _Band('2m',  144_000_000, 148_000_000),
  _Band('1.25m', 222_000_000, 225_000_000),
  _Band('70cm', 420_000_000, 450_000_000),
];

_Band? _bandFor(int hz) {
  for (final b in _kBands) {
    if (hz >= b.low && hz <= b.high) return b;
  }
  return null;
}

/// Returns the (lowOffset, highOffset) of occupied spectrum relative to the
/// dial frequency for a given mode string.  Positive = above, negative = below.
(int low, int high) _modePassband(String mode) {
  const ssb = 2700;
  switch (mode.toUpperCase()) {
    case 'LSB':
    case 'PKT-LSB':
      return (-ssb, 0);
    case 'USB':
    case 'PKT-USB':
    case 'DIG':
      return (0, ssb);
    case 'CW':
    case 'CW-R':
      return (-250, 250);
    case 'AM':
      return (-ssb, ssb);
    case 'FM':
    case 'PKT-FM':
      return (-5000, 5000);
    case 'WFM':
      return (-100000, 100000);
    case 'RTTY':
    case 'RTTY-R':
      return (-250, 250);
    default:
      return (-ssb ~/ 2, ssb ~/ 2);
  }
}

/// Picks a tick spacing that yields roughly 5–8 ticks across [span] Hz.
int _niceInterval(int span) {
  if (span <= 0) return 1;
  final raw = span / 6.0;
  final mag = math.pow(10, (math.log(raw) / math.ln10).floor()).toInt();
  final norm = raw / mag;
  final int niceFactor;
  if (norm <= 1.0) {
    niceFactor = 1;
  } else if (norm <= 2.0) {
    niceFactor = 2;
  } else if (norm <= 5.0) {
    niceFactor = 5;
  } else {
    niceFactor = 10;
  }
  return niceFactor * mag;
}

/// Short frequency label for tick marks.
String _freqShort(int hz) {
  if (hz >= 1000000) {
    final mhz = hz / 1000000.0;
    return (mhz == mhz.floorToDouble())
        ? '${mhz.toInt()}'
        : mhz.toStringAsFixed(mhz.remainder(0.1) == 0.0 ? 1 : 2);
  }
  return '${hz ~/ 1000}k';
}

// ── Spectrum data types ───────────────────────────────────────────────────────

/// One horizontal sweep from a spectrum source, mirroring the Rust type.
class SpectrumLine {
  final int centerHz;
  final int spanHz;

  /// Amplitude values in dBFS (negative; higher = stronger signal).
  final List<double> amplitudesDbfs;

  const SpectrumLine({
    required this.centerHz,
    required this.spanHz,
    required this.amplitudesDbfs,
  });

  int get lowHz => centerHz - spanHz ~/ 2;
  int get highHz => centerHz + spanHz ~/ 2;
}

/// Generates synthetic spectrum data useful for UI development.
class MockSpectrum {
  final int centerHz;
  final int spanHz;
  final int bins;

  MockSpectrum({required this.centerHz, required this.spanHz, this.bins = 256});

  Stream<SpectrumLine> stream({Duration rate = const Duration(milliseconds: 80)}) {
    final rng = math.Random();
    double t = 0;
    late StreamController<SpectrumLine> ctrl;
    Timer? timer;
    ctrl = StreamController<SpectrumLine>(
      onListen: () {
        timer = Timer.periodic(rate, (_) {
          if (!ctrl.hasListener) return;
          ctrl.add(_generate(rng, t));
          t += rate.inMilliseconds / 1000.0;
        });
      },
      onCancel: () => timer?.cancel(),
    );
    return ctrl.stream;
  }

  SpectrumLine _generate(math.Random rng, double t) {
    final amps = List<double>.generate(bins, (i) {
      final x = i / bins;
      final noise = -100.0 + rng.nextDouble() * 8.0;
      final p0 = _gauss(x, 0.25 + 0.04 * math.sin(t * 0.4), 0.009) * 32;
      final p1 = _gauss(x, 0.61 + 0.02 * math.cos(t * 0.7), 0.006) * 22;
      final p2 = _gauss(x, 0.80, 0.003) * 18;
      return math.min(0.0, noise + p0 + p1 + p2);
    });
    return SpectrumLine(centerHz: centerHz, spanHz: spanHz, amplitudesDbfs: amps);
  }

  static double _gauss(double x, double cx, double w) =>
      math.exp(-0.5 * math.pow((x - cx) / w, 2));
}

// ── Display mode ──────────────────────────────────────────────────────────────

enum _DisplayMode { dial, mockWaterfall, radioWaterfall }

// ── BandDisplay ───────────────────────────────────────────────────────────────

/// Shows either a scrolling waterfall (when a spectrum source is available) or
/// an analogue band-dial fallback.  Tap the tune icon to switch sources.
class BandDisplay extends StatefulWidget {
  final int centerHz;
  final String mode;

  /// A live spectrum stream from the radio.  When null, mock or dial is shown.
  final Stream<SpectrumLine>? radioStream;

  const BandDisplay({
    super.key,
    required this.centerHz,
    required this.mode,
    this.radioStream,
  });

  @override
  State<BandDisplay> createState() => _BandDisplayState();
}

class _BandDisplayState extends State<BandDisplay> {
  _DisplayMode _displayMode = _DisplayMode.dial;
  MockSpectrum? _mock;
  Stream<SpectrumLine>? _activeStream;

  @override
  void initState() {
    super.initState();
    if (widget.radioStream != null) {
      _displayMode = _DisplayMode.radioWaterfall;
      _activeStream = widget.radioStream;
    }
  }

  @override
  void didUpdateWidget(BandDisplay old) {
    super.didUpdateWidget(old);
    if (widget.radioStream != null && _displayMode == _DisplayMode.dial) {
      setState(() {
        _displayMode = _DisplayMode.radioWaterfall;
        _activeStream = widget.radioStream;
      });
    }
    // Retune mock source when frequency moves significantly
    if (_displayMode == _DisplayMode.mockWaterfall && _mock != null) {
      if ((widget.centerHz - _mock!.centerHz).abs() > 100000) {
        _startMock();
      }
    }
  }

  int _defaultSpan() {
    final band = _bandFor(widget.centerHz);
    if (band != null) {
      return (band.high - band.low).clamp(50000, 500000);
    }
    return 200000;
  }

  void _startMock() {
    final m = MockSpectrum(centerHz: widget.centerHz, spanHz: _defaultSpan());
    setState(() {
      _mock = m;
      _activeStream = m.stream();
    });
  }

  void _selectMode(_DisplayMode m) {
    setState(() {
      _displayMode = m;
      if (m == _DisplayMode.mockWaterfall) {
        _startMock();
      } else if (m == _DisplayMode.radioWaterfall) {
        _activeStream = widget.radioStream;
        _mock = null;
      } else {
        _activeStream = null;
        _mock = null;
      }
    });
  }

  void _showSourceSheet() {
    showModalBottomSheet<void>(
      context: context,
      backgroundColor: const Color(0xFF1A1A1A),
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(12)),
      ),
      builder: (_) => _SourceSheet(
        current: _displayMode,
        radioAvailable: widget.radioStream != null,
        onSelect: (m) { Navigator.pop(context); _selectMode(m); },
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final band = _bandFor(widget.centerHz);
    final (passLow, passHigh) = _modePassband(widget.mode);
    final isWaterfall = _displayMode != _DisplayMode.dial;

    return Container(
      decoration: BoxDecoration(
        color: const Color(0xFF0A0A0A),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: Colors.white12, width: 0.5),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // ── Header ─────────────────────────────────────────────────────────
          Padding(
            padding: const EdgeInsets.fromLTRB(10, 4, 2, 0),
            child: Row(
              children: [
                if (band != null)
                  Text(
                    band.name,
                    style: const TextStyle(
                      fontSize: 10,
                      color: Colors.white38,
                      letterSpacing: 2,
                    ),
                  ),
                const Spacer(),
                IconButton(
                  icon: const Icon(Icons.tune, size: 14, color: Colors.white38),
                  onPressed: _showSourceSheet,
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
                  tooltip: 'Spectrum source',
                ),
              ],
            ),
          ),
          // ── Display ────────────────────────────────────────────────────────
          SizedBox(
            height: isWaterfall ? 150 : 80,
            child: isWaterfall
                ? _WaterfallView(
                    stream: _activeStream!,
                    centerHz: _mock?.centerHz ?? widget.centerHz,
                    spanHz: _mock?.spanHz ?? _defaultSpan(),
                    markerHz: widget.centerHz,
                    markerLowOffset: passLow,
                    markerHighOffset: passHigh,
                  )
                : _BandDial(
                    centerHz: widget.centerHz,
                    bandLow: band?.low ?? (widget.centerHz - 150000),
                    bandHigh: band?.high ?? (widget.centerHz + 150000),
                    occLow: widget.centerHz + passLow,
                    occHigh: widget.centerHz + passHigh,
                  ),
          ),
        ],
      ),
    );
  }
}

// ── Source selection sheet ────────────────────────────────────────────────────

class _SourceSheet extends StatelessWidget {
  final _DisplayMode current;
  final bool radioAvailable;
  final void Function(_DisplayMode) onSelect;

  const _SourceSheet({
    required this.current,
    required this.radioAvailable,
    required this.onSelect,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.fromLTRB(16, 12, 16, 16),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'SPECTRUM SOURCE',
              style: TextStyle(fontSize: 10, color: Colors.white38, letterSpacing: 2),
            ),
            const SizedBox(height: 12),
            _tile(
              context,
              icon: Icons.radio,
              label: 'Band Dial',
              subtitle: 'Band edges + mode footprint; no spectrum hardware needed',
              mode: _DisplayMode.dial,
            ),
            _tile(
              context,
              icon: Icons.graphic_eq,
              label: 'Mock Waterfall',
              subtitle: 'Synthetic data — useful for UI testing',
              mode: _DisplayMode.mockWaterfall,
            ),
            _tile(
              context,
              icon: Icons.wifi,
              label: 'Radio Waterfall',
              subtitle: radioAvailable
                  ? 'Live spectrum from the connected radio'
                  : 'Not supported by this radio',
              mode: _DisplayMode.radioWaterfall,
              enabled: radioAvailable,
            ),
          ],
        ),
      ),
    );
  }

  Widget _tile(
    BuildContext context, {
    required IconData icon,
    required String label,
    required String subtitle,
    required _DisplayMode mode,
    bool enabled = true,
  }) {
    final selected = current == mode;
    return ListTile(
      dense: true,
      enabled: enabled,
      leading: Icon(icon, size: 20, color: selected ? const Color(0xFFFFBF00) : Colors.white54),
      title: Text(
        label,
        style: TextStyle(
          fontSize: 13,
          color: selected ? const Color(0xFFFFBF00) : (enabled ? Colors.white70 : Colors.white24),
          fontWeight: selected ? FontWeight.w600 : FontWeight.normal,
        ),
      ),
      subtitle: Text(
        subtitle,
        style: TextStyle(
          fontSize: 10,
          color: enabled ? Colors.white38 : Colors.white24,
        ),
      ),
      trailing: selected
          ? const Icon(Icons.check, size: 16, color: Color(0xFFFFBF00))
          : null,
      onTap: enabled ? () => onSelect(mode) : null,
    );
  }
}

// ── Band dial ─────────────────────────────────────────────────────────────────

class _BandDial extends StatelessWidget {
  final int centerHz;
  final int bandLow;
  final int bandHigh;
  final int occLow;
  final int occHigh;

  const _BandDial({
    required this.centerHz,
    required this.bandLow,
    required this.bandHigh,
    required this.occLow,
    required this.occHigh,
  });

  @override
  Widget build(BuildContext context) {
    return CustomPaint(
      painter: _BandDialPainter(
        centerHz: centerHz,
        bandLow: bandLow,
        bandHigh: bandHigh,
        occLow: occLow,
        occHigh: occHigh,
      ),
    );
  }
}

class _BandDialPainter extends CustomPainter {
  final int centerHz;
  final int bandLow;
  final int bandHigh;
  final int occLow;
  final int occHigh;

  const _BandDialPainter({
    required this.centerHz,
    required this.bandLow,
    required this.bandHigh,
    required this.occLow,
    required this.occHigh,
  });

  double _x(int hz, double width) {
    final span = bandHigh - bandLow;
    if (span <= 0) return 0;
    return (hz - bandLow) / span * width;
  }

  @override
  void paint(Canvas canvas, Size size) {
    const trackY = 28.0;
    final trackPaint = Paint()
      ..color = Colors.white24
      ..strokeWidth = 1.0
      ..style = PaintingStyle.stroke;
    final ambPaint = Paint()..color = const Color(0xFFFFBF00);
    final dimPaint = Paint()..color = Colors.white24;

    // ── Track ──
    canvas.drawLine(Offset(2, trackY), Offset(size.width - 2, trackY), trackPaint);

    // Band-edge tick marks
    for (final edgeX in [2.0, size.width - 2]) {
      canvas.drawLine(
        Offset(edgeX, trackY - 7),
        Offset(edgeX, trackY + 7),
        dimPaint..strokeWidth = 1.0,
      );
    }

    // ── Occupied bandwidth ──
    final xOccL = _x(occLow.clamp(bandLow, bandHigh), size.width);
    final xOccH = _x(occHigh.clamp(bandLow, bandHigh), size.width);
    if (xOccH > xOccL) {
      canvas.drawRect(
        Rect.fromLTWH(xOccL, trackY - 11, xOccH - xOccL, 22),
        Paint()..color = const Color(0x44FFBF00),
      );
      canvas.drawRect(
        Rect.fromLTWH(xOccL, trackY - 11, xOccH - xOccL, 22),
        Paint()
          ..color = const Color(0xAAFFBF00)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 0.5,
      );
    }

    // ── Tick marks and labels ──
    final span = bandHigh - bandLow;
    final interval = _niceInterval(span);
    final firstTick = ((bandLow / interval).ceil() * interval).toInt();

    for (int f = firstTick; f <= bandHigh; f += interval) {
      final x = _x(f, size.width);
      canvas.drawLine(
        Offset(x, trackY),
        Offset(x, trackY + 8),
        Paint()..color = Colors.white24..strokeWidth = 0.5,
      );
      final label = _freqShort(f);
      final tp = TextPainter(
        text: TextSpan(
          text: label,
          style: const TextStyle(color: Colors.white38, fontSize: 8),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      tp.paint(
        canvas,
        Offset((x - tp.width / 2).clamp(0, size.width - tp.width), trackY + 10),
      );
    }

    // ── Needle ──
    final xN = _x(centerHz.clamp(bandLow, bandHigh), size.width);

    // Triangle pointer above track
    final tri = Path()
      ..moveTo(xN, trackY - 14)
      ..lineTo(xN - 5, trackY - 6)
      ..lineTo(xN + 5, trackY - 6)
      ..close();
    canvas.drawPath(tri, ambPaint);

    // Vertical needle line
    canvas.drawLine(
      Offset(xN, trackY - 6),
      Offset(xN, trackY + 6),
      ambPaint..strokeWidth = 1.5,
    );

    // ── Frequency labels ──
    final mhz = centerHz / 1e6;
    final cLabel = '${mhz.toStringAsFixed(3)} MHz';
    final cTp = TextPainter(
      text: TextSpan(
        text: cLabel,
        style: const TextStyle(
          color: Color(0xFFFFBF00),
          fontSize: 9,
          fontWeight: FontWeight.w500,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    cTp.paint(
      canvas,
      Offset((xN - cTp.width / 2).clamp(0, size.width - cTp.width), size.height - 13),
    );

    // Band edge frequency labels (corners)
    _drawEdgeLabel(canvas, size, bandLow, 0, false);
    _drawEdgeLabel(canvas, size, bandHigh, size.width, true);
  }

  void _drawEdgeLabel(Canvas canvas, Size size, int hz, double x, bool rightAlign) {
    final label = _freqShort(hz);
    final tp = TextPainter(
      text: TextSpan(text: label, style: const TextStyle(color: Colors.white24, fontSize: 8)),
      textDirection: TextDirection.ltr,
    )..layout();
    tp.paint(canvas, Offset(rightAlign ? x - tp.width : x, size.height - 13));
  }

  @override
  bool shouldRepaint(_BandDialPainter old) =>
      old.centerHz != centerHz ||
      old.bandLow != bandLow ||
      old.bandHigh != bandHigh ||
      old.occLow != occLow ||
      old.occHigh != occHigh;
}

// ── Waterfall ─────────────────────────────────────────────────────────────────

class _WaterfallView extends StatefulWidget {
  final Stream<SpectrumLine> stream;
  final int centerHz;
  final int spanHz;
  final int markerHz;
  final int markerLowOffset;
  final int markerHighOffset;

  const _WaterfallView({
    required this.stream,
    required this.centerHz,
    required this.spanHz,
    required this.markerHz,
    required this.markerLowOffset,
    required this.markerHighOffset,
  });

  @override
  State<_WaterfallView> createState() => _WaterfallViewState();
}

class _WaterfallViewState extends State<_WaterfallView> {
  static const _imgW = 256;
  static const _imgH = 120;
  static const _maxLines = _imgH;

  final _lines = ListQueue<SpectrumLine>(_maxLines);
  StreamSubscription<SpectrumLine>? _sub;
  ui.Image? _image;
  bool _building = false;

  @override
  void initState() {
    super.initState();
    _sub = widget.stream.listen(_onLine);
  }

  @override
  void didUpdateWidget(_WaterfallView old) {
    super.didUpdateWidget(old);
    if (old.stream != widget.stream) {
      _sub?.cancel();
      _lines.clear();
      _image?.dispose();
      _image = null;
      _sub = widget.stream.listen(_onLine);
    }
  }

  @override
  void dispose() {
    _sub?.cancel();
    _image?.dispose();
    super.dispose();
  }

  void _onLine(SpectrumLine line) {
    _lines.addFirst(line);
    while (_lines.length > _maxLines) {
      _lines.removeLast();
    }
    _rebuildImage();
  }

  Future<void> _rebuildImage() async {
    if (_building) return;
    _building = true;
    try {
      final snapshot = List<SpectrumLine>.unmodifiable(_lines);
      final img = await _buildImage(snapshot);
      if (mounted) {
        setState(() {
          _image?.dispose();
          _image = img;
        });
      } else {
        img.dispose();
      }
    } finally {
      _building = false;
    }
  }

  static Future<ui.Image> _buildImage(List<SpectrumLine> lines) async {
    final pixels = Uint8List(_imgW * _imgH * 4);

    // Background: deep space blue
    for (int i = 3; i < pixels.length; i += 4) {
      pixels[i] = 255; // alpha
    }
    for (int row = lines.length; row < _imgH; row++) {
      for (int col = 0; col < _imgW; col++) {
        final o = (row * _imgW + col) * 4;
        pixels[o] = 0; pixels[o + 1] = 0; pixels[o + 2] = 20;
      }
    }

    final numRows = math.min(lines.length, _imgH);
    for (int row = 0; row < numRows; row++) {
      final line = lines[row];
      final bins = line.amplitudesDbfs.length;
      for (int col = 0; col < _imgW; col++) {
        final binIdx = (col * bins / _imgW).floor().clamp(0, bins - 1);
        final t = ((line.amplitudesDbfs[binIdx] + 120) / 120).clamp(0.0, 1.0);
        final (r, g, b) = _toRgb(t);
        final o = (row * _imgW + col) * 4;
        pixels[o] = r; pixels[o + 1] = g; pixels[o + 2] = b; pixels[o + 3] = 255;
      }
    }

    final buf = await ui.ImmutableBuffer.fromUint8List(pixels);
    final desc = ui.ImageDescriptor.raw(
      buf,
      width: _imgW,
      height: _imgH,
      pixelFormat: ui.PixelFormat.rgba8888,
    );
    final codec = await desc.instantiateCodec();
    final frame = await codec.getNextFrame();
    return frame.image;
  }

  static (int, int, int) _toRgb(double t) {
    if (t < 0.2) {
      final s = t / 0.2;
      return (0, 0, (20 + s * 185).round());
    } else if (t < 0.4) {
      final s = (t - 0.2) / 0.2;
      return (0, (s * 150).round(), (205 - s * 55).round());
    } else if (t < 0.6) {
      final s = (t - 0.4) / 0.2;
      return (0, (150 + s * 105).round(), (150 - s * 150).round());
    } else if (t < 0.8) {
      final s = (t - 0.6) / 0.2;
      return ((s * 255).round(), 255, 0);
    } else {
      final s = (t - 0.8) / 0.2;
      return (255, (255 * (1 - s)).round(), 0);
    }
  }

  @override
  Widget build(BuildContext context) {
    return CustomPaint(
      painter: _WaterfallPainter(
        image: _image,
        centerHz: widget.centerHz,
        spanHz: widget.spanHz,
        markerHz: widget.markerHz,
        markerLow: widget.markerHz + widget.markerLowOffset,
        markerHigh: widget.markerHz + widget.markerHighOffset,
      ),
    );
  }
}

class _WaterfallPainter extends CustomPainter {
  final ui.Image? image;
  final int centerHz;
  final int spanHz;
  final int markerHz;
  final int markerLow;
  final int markerHigh;

  const _WaterfallPainter({
    required this.image,
    required this.centerHz,
    required this.spanHz,
    required this.markerHz,
    required this.markerLow,
    required this.markerHigh,
  });

  double _x(int hz, double width) {
    final low = centerHz - spanHz ~/ 2;
    final high = centerHz + spanHz ~/ 2;
    if (high <= low) return 0;
    return (hz - low) / (high - low) * width;
  }

  @override
  void paint(Canvas canvas, Size size) {
    const axisH = 20.0;
    final waterfallRect = Rect.fromLTWH(0, 0, size.width, size.height - axisH);

    final img = image;
    if (img != null) {
      canvas.drawImageRect(
        img,
        Rect.fromLTWH(0, 0, img.width.toDouble(), img.height.toDouble()),
        waterfallRect,
        Paint(),
      );
    } else {
      canvas.drawRect(waterfallRect, Paint()..color = const Color(0xFF000014));
      // "Waiting…" text
      final tp = TextPainter(
        text: const TextSpan(
          text: 'Waiting for spectrum data…',
          style: TextStyle(color: Colors.white24, fontSize: 11),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      tp.paint(
        canvas,
        Offset(
          (size.width - tp.width) / 2,
          (waterfallRect.height - tp.height) / 2,
        ),
      );
    }

    // Occupied-band highlight
    final xOccL = _x(markerLow, size.width);
    final xOccH = _x(markerHigh, size.width);
    if (xOccH > xOccL) {
      canvas.drawRect(
        Rect.fromLTWH(xOccL, 0, xOccH - xOccL, waterfallRect.height),
        Paint()..color = const Color(0x22FFBF00),
      );
    }

    // Current-frequency marker
    final xM = _x(markerHz, size.width);
    if (xM >= 0 && xM <= size.width) {
      canvas.drawLine(
        Offset(xM, 0),
        Offset(xM, waterfallRect.height),
        Paint()..color = const Color(0xCCFFBF00)..strokeWidth = 1.0,
      );
    }

    // ── Frequency axis ──
    final axisY = waterfallRect.height;
    canvas.drawLine(
      Offset(0, axisY),
      Offset(size.width, axisY),
      Paint()..color = Colors.white12..strokeWidth = 0.5,
    );

    final lowHz = centerHz - spanHz ~/ 2;
    final highHz = centerHz + spanHz ~/ 2;
    final interval = _niceInterval(spanHz);
    final firstTick = ((lowHz / interval).ceil() * interval).toInt();

    for (int f = firstTick; f <= highHz; f += interval) {
      final x = _x(f, size.width);
      canvas.drawLine(
        Offset(x, axisY),
        Offset(x, axisY + 3),
        Paint()..color = Colors.white24..strokeWidth = 0.5,
      );
      final label = _freqShort(f);
      final tp = TextPainter(
        text: TextSpan(
          text: label,
          style: const TextStyle(color: Colors.white38, fontSize: 8),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      tp.paint(
        canvas,
        Offset((x - tp.width / 2).clamp(0, size.width - tp.width), axisY + 4),
      );
    }

    // Marker label (current frequency, amber)
    final mLabel = (markerHz / 1e6).toStringAsFixed(3);
    final mTp = TextPainter(
      text: TextSpan(
        text: mLabel,
        style: const TextStyle(
          color: Color(0xFFFFBF00),
          fontSize: 8,
          fontWeight: FontWeight.w600,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    mTp.paint(
      canvas,
      Offset((xM - mTp.width / 2).clamp(0, size.width - mTp.width), axisY + 4),
    );
  }

  @override
  bool shouldRepaint(_WaterfallPainter old) =>
      old.image != image ||
      old.markerHz != markerHz ||
      old.markerLow != markerLow ||
      old.markerHigh != markerHigh;
}
