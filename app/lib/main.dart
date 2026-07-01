import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'radio_connection.dart';
import 'vfo_screen.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RadioConnection.ensureRustInit();
  runApp(
    ChangeNotifierProvider(
      create: (_) => RadioConnection(),
      child: const QuirqApp(),
    ),
  );
}

class QuirqApp extends StatelessWidget {
  const QuirqApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Quirq',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        brightness: Brightness.dark,
        scaffoldBackgroundColor: const Color(0xFF181818),
        colorScheme: const ColorScheme.dark(
          primary: Color(0xFF00CC66),
          surface: Color(0xFF202020),
        ),
        useMaterial3: true,
      ),
      home: const VfoScreen(),
    );
  }
}
