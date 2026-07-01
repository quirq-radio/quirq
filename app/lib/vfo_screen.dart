import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'band_display.dart';
import 'radio_connection.dart';
import 'connect_dialog.dart';

// ── Step sizes shown in the tuning toolbar ────────────────────────────────────
const _steps = [
  (1, '1 Hz'),
  (10, '10 Hz'),
  (100, '100 Hz'),
  (1000, '1 kHz'),
  (10000, '10 kHz'),
  (100000, '100 kHz'),
  (1000000, '1 MHz'),
];

class VfoScreen extends StatefulWidget {
  const VfoScreen({super.key});

  @override
  State<VfoScreen> createState() => _VfoScreenState();
}

class _VfoScreenState extends State<VfoScreen> {
  int _stepHz = 1000;

  List<Widget> _buildVfoSection(RadioConnection radio) {
    final vfos = radio.vfoNames;
    final widgets = <Widget>[];

    for (int i = 0; i < vfos.length; i++) {
      final vfo = vfos[i];
      widgets.add(_VfoBlock(
        key: ValueKey('vfo_$vfo'),
        label: 'VFO $vfo',
        hz: radio.freqOf(vfo),
        isActive: radio.activeVfo == vfo,
        onTap: () => radio.selectVfo(vfo),
      ));

      if (vfos.length == 2 && i == 0) {
        // Classic layout: controls sit between the two VFO blocks.
        widgets.add(const SizedBox(height: 8));
        widgets.add(_buildVfoControls(radio));
      }

      if (i < vfos.length - 1) {
        widgets.add(const SizedBox(height: 8));
      }
    }

    // For 3+ VFOs: controls go below all blocks.
    if (vfos.length > 2) {
      widgets.add(const SizedBox(height: 8));
      widgets.add(_buildVfoControls(radio));
    }

    return widgets;
  }

  Widget _buildVfoControls(RadioConnection radio) {
    final other = radio.adjacentVfo!;
    final a = radio.activeVfo;
    return _VfoControls(
      swapLabel: '$a↔$other',
      copyLabel: '$a=$other',
      isSplit: radio.split,
      onSwap: radio.swapVfos,
      onCopy: radio.copyVfo,
      onSplit: radio.toggleSplit,
    );
  }

  @override
  Widget build(BuildContext context) {
    final radio = context.watch<RadioConnection>();

    return Scaffold(
      appBar: AppBar(
        backgroundColor: const Color(0xFF0D0D0D),
        title: const Text(
          'Q U I R Q',
          style: TextStyle(
            color: Colors.white54,
            fontSize: 13,
            letterSpacing: 6,
            fontWeight: FontWeight.w300,
          ),
        ),
        actions: [
          _ConnectionChip(
            status: radio.status,
            onTap: () => radio.isConnected
                ? radio.disconnect()
                : ConnectSheet.show(context),
          ),
          const SizedBox(width: 8),
        ],
      ),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              ..._buildVfoSection(radio),
              const SizedBox(height: 12),
              _TuningBar(
                stepHz: _stepHz,
                onStepChanged: (s) => setState(() => _stepHz = s),
                onTune: (delta) => radio.stepFreq(delta),
              ),
              const SizedBox(height: 12),
              BandDisplay(
                centerHz: radio.activeHz,
                mode: radio.mode,
              ),
              const SizedBox(height: 16),
              const _SectionLabel('MODE'),
              const SizedBox(height: 6),
              _ModeSelector(
                selected: radio.mode,
                onChanged: radio.setMode,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

// ── Connection status chip ────────────────────────────────────────────────────

class _ConnectionChip extends StatelessWidget {
  final ConnectionStatus status;
  final VoidCallback onTap;

  const _ConnectionChip({required this.status, required this.onTap});

  @override
  Widget build(BuildContext context) {
    final (label, color) = switch (status) {
      ConnectionStatus.connected => ('CONNECTED', const Color(0xFF00CC66)),
      ConnectionStatus.connecting => ('CONNECTING…', Colors.amber),
      ConnectionStatus.error => ('ERROR', Colors.redAccent),
      ConnectionStatus.disconnected => ('CONNECT', Colors.white38),
    };

    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
        decoration: BoxDecoration(
          border: Border.all(color: color.withValues(alpha: 0.5), width: 0.5),
          borderRadius: BorderRadius.circular(4),
          color: color.withValues(alpha: 0.08),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 6,
              height: 6,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: color,
                boxShadow: status == ConnectionStatus.connected
                    ? [BoxShadow(color: color, blurRadius: 4)]
                    : null,
              ),
            ),
            const SizedBox(width: 5),
            Text(
              label,
              style: TextStyle(fontSize: 9, color: color, letterSpacing: 1.5),
            ),
          ],
        ),
      ),
    );
  }
}

// ── VFO block (label + frequency display) ────────────────────────────────────

class _VfoBlock extends StatelessWidget {
  final String label;
  final int hz;
  final bool isActive;
  final VoidCallback onTap;

  const _VfoBlock({
    super.key,
    required this.label,
    required this.hz,
    required this.isActive,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    const activeGreen = Color(0xFF00CC66);
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.fromLTRB(12, 8, 12, 10),
        decoration: BoxDecoration(
          color: const Color(0xFF0D0D0D),
          borderRadius: BorderRadius.circular(6),
          border: Border.all(
            color: isActive
                ? activeGreen.withValues(alpha: 0.5)
                : Colors.white12,
            width: isActive ? 1.5 : 0.5,
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Container(
                  width: 7,
                  height: 7,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: isActive ? const Color(0xFF00FF88) : Colors.white24,
                    boxShadow: isActive
                        ? [const BoxShadow(color: Color(0xFF00FF88), blurRadius: 4)]
                        : null,
                  ),
                ),
                const SizedBox(width: 6),
                Text(
                  label,
                  style: TextStyle(
                    fontSize: 10,
                    color: isActive ? activeGreen : Colors.white38,
                    letterSpacing: 2,
                    fontWeight: FontWeight.w500,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 6),
            FrequencyDisplay(hz: hz, isActive: isActive),
          ],
        ),
      ),
    );
  }
}

// ── Frequency display ─────────────────────────────────────────────────────────

class FrequencyDisplay extends StatelessWidget {
  final int hz;
  final bool isActive;

  const FrequencyDisplay({super.key, required this.hz, required this.isActive});

  static String _format(int hz) {
    final mhz = hz ~/ 1000000;
    final khz = (hz % 1000000) ~/ 1000;
    final sub = hz % 1000;
    return '$mhz.${khz.toString().padLeft(3, '0')},${sub.toString().padLeft(3, '0')}';
  }

  @override
  Widget build(BuildContext context) {
    final formatted = _format(hz);
    const activeDigit = Color(0xFFFFCC44);
    const dimDigit = Color(0xFF554422);
    const activeSep = Color(0xFFCC9933);
    const dimSep = Color(0xFF3A2F10);

    final digitColor = isActive ? activeDigit : dimDigit;
    final sepColor = isActive ? activeSep : dimSep;

    return LayoutBuilder(builder: (context, constraints) {
      final digitSize = (constraints.maxWidth / 13).clamp(22.0, 44.0);
      final fontSize = digitSize * 0.88;

      return Row(
        crossAxisAlignment: CrossAxisAlignment.baseline,
        textBaseline: TextBaseline.alphabetic,
        children: [
          ...formatted.split('').map((ch) {
            if (ch == '.' || ch == ',') {
              return Padding(
                padding: EdgeInsets.symmetric(horizontal: digitSize * 0.04),
                child: Text(
                  ch,
                  style: TextStyle(
                    fontSize: fontSize * 0.7,
                    color: sepColor,
                    fontWeight: FontWeight.w300,
                  ),
                ),
              );
            }
            return _DigitBox(
              char: ch,
              size: digitSize,
              fontSize: fontSize,
              color: digitColor,
            );
          }),
          SizedBox(width: digitSize * 0.3),
          Padding(
            padding: EdgeInsets.only(bottom: fontSize * 0.08),
            child: Text(
              'MHz',
              style: TextStyle(
                fontSize: fontSize * 0.28,
                color: isActive ? Colors.white54 : Colors.white24,
                letterSpacing: 1,
              ),
            ),
          ),
        ],
      );
    });
  }
}

class _DigitBox extends StatelessWidget {
  final String char;
  final double size;
  final double fontSize;
  final Color color;

  const _DigitBox({
    required this.char,
    required this.size,
    required this.fontSize,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      width: size,
      margin: EdgeInsets.symmetric(horizontal: size * 0.03),
      padding: EdgeInsets.symmetric(
        horizontal: size * 0.05,
        vertical: size * 0.04,
      ),
      decoration: BoxDecoration(
        color: Colors.black,
        borderRadius: BorderRadius.circular(2),
        border: Border.all(color: color.withValues(alpha: 0.2), width: 0.5),
      ),
      alignment: Alignment.center,
      child: Text(
        char,
        style: TextStyle(
          fontSize: fontSize,
          color: color,
          fontFamily: 'monospace',
          fontWeight: FontWeight.w300,
          height: 1.1,
        ),
      ),
    );
  }
}

// ── VFO swap / copy / split controls ─────────────────────────────────────────

class _VfoControls extends StatelessWidget {
  final String swapLabel;
  final String copyLabel;
  final bool isSplit;
  final VoidCallback onSwap;
  final VoidCallback onCopy;
  final VoidCallback onSplit;

  const _VfoControls({
    required this.swapLabel,
    required this.copyLabel,
    required this.isSplit,
    required this.onSwap,
    required this.onCopy,
    required this.onSplit,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        _Btn(key: const ValueKey('btn_swap'), label: swapLabel, onTap: onSwap),
        const SizedBox(width: 6),
        _Btn(key: const ValueKey('btn_copy'), label: copyLabel, onTap: onCopy),
        const SizedBox(width: 6),
        _Btn(label: 'V/M', onTap: () {}),
        const Spacer(),
        _Btn(
          label: 'SPLIT',
          onTap: onSplit,
          active: isSplit,
          activeColor: const Color(0xFFFF8844),
        ),
      ],
    );
  }
}

// ── Tuning bar (step selector + ▼ ▲ buttons) ─────────────────────────────────

class _TuningBar extends StatelessWidget {
  final int stepHz;
  final ValueChanged<int> onStepChanged;
  final ValueChanged<int> onTune;

  const _TuningBar({
    required this.stepHz,
    required this.onStepChanged,
    required this.onTune,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        // Step size picker
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
          decoration: BoxDecoration(
            color: const Color(0xFF252525),
            borderRadius: BorderRadius.circular(4),
            border: Border.all(color: Colors.white24, width: 0.5),
          ),
          child: DropdownButtonHideUnderline(
            child: DropdownButton<int>(
              value: stepHz,
              isDense: true,
              dropdownColor: const Color(0xFF252525),
              style: const TextStyle(fontSize: 11, color: Colors.white70),
              icon: const Icon(Icons.arrow_drop_down, size: 14, color: Colors.white38),
              items: _steps
                  .map((s) => DropdownMenuItem(value: s.$1, child: Text(s.$2)))
                  .toList(),
              onChanged: (v) => onStepChanged(v!),
            ),
          ),
        ),
        const SizedBox(width: 10),
        // Down button
        _TuneBtn(
          icon: Icons.remove,
          onTap: () => onTune(-stepHz),
        ),
        const SizedBox(width: 6),
        // Up button
        _TuneBtn(
          icon: Icons.add,
          onTap: () => onTune(stepHz),
        ),
      ],
    );
  }
}

class _TuneBtn extends StatelessWidget {
  final IconData icon;
  final VoidCallback onTap;

  const _TuneBtn({required this.icon, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 8),
        decoration: BoxDecoration(
          color: const Color(0xFF252525),
          borderRadius: BorderRadius.circular(4),
          border: Border.all(color: Colors.white24, width: 0.5),
        ),
        child: Icon(icon, size: 16, color: Colors.white70),
      ),
    );
  }
}

// ── Mode selector ─────────────────────────────────────────────────────────────

class _ModeSelector extends StatelessWidget {
  final String selected;
  final ValueChanged<String> onChanged;

  static const _modes = ['LSB', 'USB', 'CW', 'AM', 'FM', 'DIG'];

  const _ModeSelector({required this.selected, required this.onChanged});

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: 6,
      runSpacing: 6,
      children: _modes.map((label) {
        final sel = label.toLowerCase() == selected.toLowerCase();
        const activeColor = Color(0xFF00CC66);
        return GestureDetector(
          key: ValueKey('mode_$label'),
          onTap: () => onChanged(label),
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 9),
            decoration: BoxDecoration(
              color: sel ? activeColor.withValues(alpha: 0.12) : const Color(0xFF252525),
              borderRadius: BorderRadius.circular(4),
              border: Border.all(
                color: sel ? activeColor : Colors.white24,
                width: sel ? 1 : 0.5,
              ),
            ),
            child: Text(
              label,
              style: TextStyle(
                fontSize: 12,
                color: sel ? activeColor : Colors.white60,
                fontWeight: sel ? FontWeight.w600 : FontWeight.normal,
                letterSpacing: 1,
              ),
            ),
          ),
        );
      }).toList(),
    );
  }
}

// ── Shared helpers ────────────────────────────────────────────────────────────

class _SectionLabel extends StatelessWidget {
  final String text;
  const _SectionLabel(this.text);

  @override
  Widget build(BuildContext context) => Text(
        text,
        style: const TextStyle(
          fontSize: 10,
          color: Colors.white38,
          letterSpacing: 2.5,
        ),
      );
}

class _Btn extends StatelessWidget {
  final String label;
  final VoidCallback onTap;
  final bool active;
  final Color? activeColor;

  const _Btn({
    super.key,
    required this.label,
    required this.onTap,
    this.active = false,
    this.activeColor,
  });

  @override
  Widget build(BuildContext context) {
    final col = activeColor ?? const Color(0xFF00CC66);
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        decoration: BoxDecoration(
          color: active ? col.withValues(alpha: 0.12) : const Color(0xFF252525),
          borderRadius: BorderRadius.circular(4),
          border: Border.all(
            color: active ? col : Colors.white24,
            width: active ? 1 : 0.5,
          ),
        ),
        child: Text(
          label,
          style: TextStyle(
            fontSize: 11,
            color: active ? col : Colors.white70,
            letterSpacing: 0.5,
            fontWeight: FontWeight.w500,
          ),
        ),
      ),
    );
  }
}
