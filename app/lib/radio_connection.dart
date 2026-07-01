import 'dart:async';
import 'package:flutter/foundation.dart';
import 'src/rust/api.dart';
import 'src/rust/frb_generated.dart';

/// Possible states of the connection lifecycle.
enum ConnectionStatus { disconnected, connecting, connected, error }

/// Holds a live `RigHandle` and keeps the UI state in sync via polling.
///
/// VFO count is not assumed — `vfoNames` is populated by probing the radio on
/// connect.  The demo (no-radio) default is two VFOs, matching the most common
/// HF-radio layout, but any count is supported at runtime.
class RadioConnection extends ChangeNotifier {
  RigHandle? _rig;
  ConnectionStatus _status = ConnectionStatus.disconnected;
  String? _errorMessage;
  Timer? _pollTimer;

  // — VFO state ——————————————————————————————————————————————————————————————

  /// Ordered list of VFO identifiers available on the current radio.
  /// Populated by probing the radio on connect; defaults to ['A', 'B']
  /// for the disconnected demo mode.
  List<String> vfoNames = const ['A', 'B'];

  /// The currently selected (transmit / tuning) VFO.
  String activeVfo = 'A';

  bool split = false;

  final _vfoHz = <String, int>{'A': 7_189_500, 'B': 14_220_000};
  final _vfoMode = <String, String>{'A': 'LSB', 'B': 'USB'};

  // — Getters ————————————————————————————————————————————————————————————————

  ConnectionStatus get status => _status;
  bool get isConnected => _status == ConnectionStatus.connected;
  String? get errorMessage => _errorMessage;

  /// Frequency of the named VFO, 0 if unknown.
  int freqOf(String vfo) => _vfoHz[vfo] ?? 0;

  /// Mode string of the named VFO; falls back to 'USB'.
  String modeOf(String vfo) => _vfoMode[vfo] ?? 'USB';

  int get activeHz => freqOf(activeVfo);
  String get mode => modeOf(activeVfo);

  /// The next VFO in the list after the active one; null when only one VFO.
  /// Used by swap / copy operations.
  String? get adjacentVfo => vfoNames.length > 1
      ? vfoNames[(vfoNames.indexOf(activeVfo) + 1) % vfoNames.length]
      : null;

  // — Lifecycle ——————————————————————————————————————————————————————————————

  static Future<void> ensureRustInit() => RustLib.init();

  Future<void> connect({
    required String driver,
    required String port,
    required String modelId,
  }) async {
    _status = ConnectionStatus.connecting;
    _errorMessage = null;
    notifyListeners();

    try {
      _rig = await RigHandle.open(
        driver: driver,
        params: [
          KeyValue(key: 'model', value: modelId),
          KeyValue(key: 'port', value: port),
        ],
      );
      await _refreshAll();
      _status = ConnectionStatus.connected;
      _startPolling();
    } catch (e) {
      _status = ConnectionStatus.error;
      _errorMessage = e.toString();
    }
    notifyListeners();
  }

  Future<void> disconnect() async {
    _stopPolling();
    _rig = null;
    _status = ConnectionStatus.disconnected;
    _errorMessage = null;
    notifyListeners();
  }

  // — Polling ————————————————————————————————————————————————————————————————

  void _startPolling() {
    _pollTimer = Timer.periodic(const Duration(milliseconds: 250), (_) => _poll());
  }

  void _stopPolling() {
    _pollTimer?.cancel();
    _pollTimer = null;
  }

  Future<void> _poll() async {
    final rig = _rig;
    if (rig == null) return;
    try {
      bool changed = false;
      for (final vfo in vfoNames) {
        final hz = (await rig.getFreq(vfo: vfo)).toInt();
        if (hz != _vfoHz[vfo]) {
          _vfoHz[vfo] = hz;
          changed = true;
        }
      }
      final (newMode, _) = await rig.getMode(vfo: activeVfo);
      if (newMode != modeOf(activeVfo)) {
        _vfoMode[activeVfo] = newMode;
        changed = true;
      }
      if (changed) notifyListeners();
    } catch (_) {
      // Transient errors during polling are silently ignored.
    }
  }

  /// Discovers which VFOs the radio actually has by probing each one.
  /// Radios that don't support a given VFO will throw; those are skipped.
  Future<void> _refreshAll() async {
    final rig = _rig!;
    const probeOrder = ['A', 'B', 'C'];
    final foundHz = <String, int>{};
    final foundMode = <String, String>{};

    for (final vfo in probeOrder) {
      try {
        foundHz[vfo] = (await rig.getFreq(vfo: vfo)).toInt();
        try {
          final (m, _) = await rig.getMode(vfo: vfo);
          foundMode[vfo] = m;
        } catch (_) {
          foundMode[vfo] = 'USB';
        }
      } catch (_) {
        // VFO not available on this radio — stop probing further.
        break;
      }
    }

    if (foundHz.isEmpty) throw Exception('Could not read any VFO from radio');

    _vfoHz
      ..clear()
      ..addAll(foundHz);
    _vfoMode
      ..clear()
      ..addAll(foundMode);
    vfoNames = probeOrder.where(foundHz.containsKey).toList();
    if (!vfoNames.contains(activeVfo)) activeVfo = vfoNames.first;
  }

  // — Radio actions ——————————————————————————————————————————————————————————

  Future<void> setFreq(String vfo, int hz) async {
    _vfoHz[vfo] = hz;
    notifyListeners();
    await _rig?.setFreq(vfo: vfo, hz: BigInt.from(hz));
  }

  Future<void> stepFreq(int deltaHz) async {
    final hz = (freqOf(activeVfo) + deltaHz).clamp(1000, 999_999_999);
    await setFreq(activeVfo, hz);
  }

  Future<void> setMode(String newMode) async {
    _vfoMode[activeVfo] = newMode;
    notifyListeners();
    await _rig?.setMode(vfo: activeVfo, mode: newMode, passbandHz: null);
  }

  Future<void> swapVfos() async {
    final other = adjacentVfo;
    if (other == null) return;
    final tmpHz = freqOf(activeVfo);
    final tmpMode = modeOf(activeVfo);
    _vfoHz[activeVfo] = freqOf(other);
    _vfoMode[activeVfo] = modeOf(other);
    _vfoHz[other] = tmpHz;
    _vfoMode[other] = tmpMode;
    notifyListeners();
    final rig = _rig;
    if (rig != null) {
      await rig.setFreq(vfo: activeVfo, hz: BigInt.from(freqOf(activeVfo)));
      await rig.setMode(vfo: activeVfo, mode: modeOf(activeVfo), passbandHz: null);
      await rig.setFreq(vfo: other, hz: BigInt.from(freqOf(other)));
      await rig.setMode(vfo: other, mode: modeOf(other), passbandHz: null);
    }
  }

  Future<void> copyVfo() async {
    final other = adjacentVfo;
    if (other == null) return;
    _vfoHz[other] = freqOf(activeVfo);
    _vfoMode[other] = modeOf(activeVfo);
    notifyListeners();
    final rig = _rig;
    if (rig != null) {
      await rig.setFreq(vfo: other, hz: BigInt.from(freqOf(activeVfo)));
      await rig.setMode(vfo: other, mode: modeOf(activeVfo), passbandHz: null);
    }
  }

  void selectVfo(String vfo) {
    if (!vfoNames.contains(vfo)) return;
    activeVfo = vfo;
    notifyListeners();
  }

  void toggleSplit() {
    split = !split;
    notifyListeners();
  }

  // — Test helpers ——————————————————————————————————————————————————————————

  /// Directly sets a VFO's mode without touching the radio hardware.
  /// Use only from test step definitions; not part of the public API.
  @visibleForTesting
  void setVfoModeForTesting(String vfo, String mode) {
    _vfoMode[vfo] = mode;
    notifyListeners();
  }

  @override
  void dispose() {
    _stopPolling();
    super.dispose();
  }
}
