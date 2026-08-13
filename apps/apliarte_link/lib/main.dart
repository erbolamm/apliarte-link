import 'dart:ui' show AppExitResponse;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'providers/app_state.dart';
import 'services/ipc_service.dart';
import 'services/daemon_service.dart';
import 'screens/home_screen.dart';

// Instancia global del motor Rust. Se arranca antes que la UI y se para
// al cerrar la app vía AppLifecycleListener (ver _ApliArteLinkAppState).
final _daemon = DaemonService();

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Arrancamos el motor Rust antes que la UI
  // Si no lo encuentra (desarrollo sin binario), la app sigue funcionando igual
  await _daemon.start();

  runApp(
    MultiProvider(
      providers: [
        Provider<IpcService>(
          create: (_) => IpcService(),
          dispose: (_, service) => service.disconnect(),
        ),
        ChangeNotifierProxyProvider<IpcService, AppState>(
          create: (context) => AppState(context.read<IpcService>()),
          update: (_, ipc, appState) => appState ?? AppState(ipc),
        ),
      ],
      child: const ApliArteLinkApp(),
    ),
  );
}

class ApliArteLinkApp extends StatefulWidget {
  const ApliArteLinkApp({super.key});

  @override
  State<ApliArteLinkApp> createState() => _ApliArteLinkAppState();
}

class _ApliArteLinkAppState extends State<ApliArteLinkApp> {
  late final AppLifecycleListener _lifecycleListener;

  @override
  void initState() {
    super.initState();
    _lifecycleListener = AppLifecycleListener(
      // Al pedir salir (cerrar ventana, Cmd+Q, etc.) paramos el motor
      // Rust antes de que el proceso termine.
      onExitRequested: _onExitRequested,
      // Red de seguridad: si la app se desmonta sin pasar por
      // onExitRequested, también paramos el daemon.
      onDetach: _stopDaemon,
    );
  }

  Future<AppExitResponse> _onExitRequested() async {
    await _stopDaemon();
    return AppExitResponse.exit;
  }

  Future<void> _stopDaemon() async {
    debugPrint('🛑 [App] Cerrando — parando el motor Rust');
    _daemon.stop();
  }

  @override
  void dispose() {
    _lifecycleListener.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'ApliArte Link',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        useMaterial3: true,
        brightness: Brightness.dark, // Tema oscuro con colores ApliArte
        scaffoldBackgroundColor: const Color(0xFF303030),
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF5ECEF5), // Azul claro
          brightness: Brightness.dark,
          surface: const Color(0xFF00467B), // Azul oscuro
          primary: const Color(0xFF5ECEF5),
          secondary: const Color(0xFFE8955E), // Naranja
        ),
        fontFamily: 'Inter', // Fuente moderna y limpia
      ),
      home: const HomeScreen(),
    );
  }
}
