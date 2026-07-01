import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'radio_connection.dart';

/// Bottom sheet that lets the user configure and open a radio connection.
class ConnectSheet extends StatefulWidget {
  const ConnectSheet({super.key});

  static Future<void> show(BuildContext context) {
    return showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: const Color(0xFF1A1A1A),
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(12)),
      ),
      builder: (_) => ChangeNotifierProvider.value(
        value: context.read<RadioConnection>(),
        child: const ConnectSheet(),
      ),
    );
  }

  @override
  State<ConnectSheet> createState() => _ConnectSheetState();
}

class _ConnectSheetState extends State<ConnectSheet> {
  final _portCtrl = TextEditingController(text: '/dev/ttyUSB0');
  final _modelCtrl = TextEditingController(text: '3085');
  String _driver = 'hamlib';
  bool _busy = false;
  String? _error;

  @override
  void dispose() {
    _portCtrl.dispose();
    _modelCtrl.dispose();
    super.dispose();
  }

  Future<void> _connect() async {
    setState(() {
      _busy = true;
      _error = null;
    });
    final radio = context.read<RadioConnection>();
    await radio.connect(
      driver: _driver,
      port: _portCtrl.text.trim(),
      modelId: _modelCtrl.text.trim(),
    );
    if (!mounted) return;
    if (radio.status == ConnectionStatus.connected) {
      Navigator.of(context).pop();
    } else {
      setState(() {
        _busy = false;
        _error = radio.errorMessage;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final bottom = MediaQuery.of(context).viewInsets.bottom;
    return Padding(
      padding: EdgeInsets.fromLTRB(20, 20, 20, 20 + bottom),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text(
            'Connect to Radio',
            style: TextStyle(
              fontSize: 14,
              fontWeight: FontWeight.w500,
              color: Colors.white70,
              letterSpacing: 1,
            ),
          ),
          const SizedBox(height: 20),
          _label('Driver'),
          const SizedBox(height: 6),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
            decoration: _boxDecor(),
            child: DropdownButtonHideUnderline(
              child: DropdownButton<String>(
                value: _driver,
                dropdownColor: const Color(0xFF252525),
                style: _inputStyle,
                items: const [
                  DropdownMenuItem(value: 'hamlib', child: Text('Hamlib')),
                ],
                onChanged: (v) => setState(() => _driver = v!),
              ),
            ),
          ),
          const SizedBox(height: 14),
          _label('Hamlib model number'),
          const SizedBox(height: 6),
          TextField(
            controller: _modelCtrl,
            keyboardType: TextInputType.number,
            style: _inputStyle,
            decoration: _inputDecor('e.g. 3085 for IC-705, 2014 for G90'),
          ),
          const SizedBox(height: 14),
          _label('Port'),
          const SizedBox(height: 6),
          TextField(
            controller: _portCtrl,
            style: _inputStyle,
            decoration: _inputDecor('e.g. /dev/ttyUSB0 or 192.168.1.10'),
          ),
          if (_error != null) ...[
            const SizedBox(height: 12),
            Container(
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                color: Colors.red.withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(4),
                border: Border.all(color: Colors.red.withValues(alpha: 0.4)),
              ),
              child: Text(
                _error!,
                style: const TextStyle(fontSize: 11, color: Colors.redAccent),
              ),
            ),
          ],
          const SizedBox(height: 20),
          ElevatedButton(
            onPressed: _busy ? null : _connect,
            style: ElevatedButton.styleFrom(
              backgroundColor: const Color(0xFF00CC66),
              foregroundColor: Colors.black,
              padding: const EdgeInsets.symmetric(vertical: 14),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(6),
              ),
            ),
            child: _busy
                ? const SizedBox(
                    height: 16,
                    width: 16,
                    child: CircularProgressIndicator(
                      strokeWidth: 2,
                      color: Colors.black,
                    ),
                  )
                : const Text(
                    'Connect',
                    style: TextStyle(fontWeight: FontWeight.w600, letterSpacing: 1),
                  ),
          ),
        ],
      ),
    );
  }

  static const TextStyle _inputStyle = TextStyle(fontSize: 13, color: Colors.white70);

  Widget _label(String text) => Text(
        text,
        style: const TextStyle(fontSize: 10, color: Colors.white38, letterSpacing: 2),
      );

  BoxDecoration _boxDecor() => BoxDecoration(
        color: const Color(0xFF252525),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: Colors.white24, width: 0.5),
      );

  InputDecoration _inputDecor(String hint) => InputDecoration(
        hintText: hint,
        hintStyle: const TextStyle(fontSize: 12, color: Colors.white24),
        filled: true,
        fillColor: const Color(0xFF252525),
        contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(6),
          borderSide: const BorderSide(color: Colors.white24, width: 0.5),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(6),
          borderSide: const BorderSide(color: Colors.white24, width: 0.5),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(6),
          borderSide: const BorderSide(color: Color(0xFF00CC66), width: 1),
        ),
      );
}
