import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'providers/app_state.dart';
import 'services/ipc_service.dart';
import 'services/daemon_service.dart';
import 'screens/home_screen.dart';

// Instancia global para poder pararlo al cerrar la app
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

class ApliArteLinkApp extends StatelessWidget {
  const ApliArteLinkApp({super.key});

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
