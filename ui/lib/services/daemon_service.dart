// Javier: Este servicio lanza el motor Rust (apliarte-switch) en segundo plano.
// Analogía: Es como lanzar un servidor local desde tu app, igual que cuando
//           un IDE lanza el dart language server sin que el usuario lo vea.
//
// El binario Rust está embebido en el .app en Contents/MacOS/apliarte-switch
// (mismo directorio que el ejecutable de Flutter).
// En Windows irá en el mismo directorio que el .exe de Flutter.

import 'dart:io';
import 'package:flutter/foundation.dart';

class DaemonService {
  Process? _process;
  bool _started = false;

  bool get isRunning => _process != null;

  Future<void> start() async {
    if (_started) return;
    _started = true;

    final daemonPath = _resolveDaemonPath();
    if (daemonPath == null) {
      debugPrint(
        '⚠️ [Daemon] Binario no encontrado — modo desarrollo sin daemon',
      );
      return;
    }

    try {
      // ProcessStartMode.normal: capturamos stdout/stderr para ver sus logs
      _process = await Process.start(
        daemonPath,
        [],
        mode: ProcessStartMode.normal,
      );

      debugPrint('🚀 [Daemon] Motor arrancado (PID: ${_process!.pid})');

      // Relay de logs del daemon al debugPrint (solo visible en debug)
      _process!.stdout.listen((data) {
        debugPrint('[Rust] ${String.fromCharCodes(data).trim()}');
      });
      _process!.stderr.listen((data) {
        debugPrint('[Rust⚠️] ${String.fromCharCodes(data).trim()}');
      });

      // Si el daemon muere solo, limpiamos la referencia
      _process!.exitCode.then((code) {
        debugPrint('📉 [Daemon] Motor se apagó con código $code');
        _process = null;
      });
    } catch (e) {
      debugPrint('❌ [Daemon] No se pudo arrancar el motor: $e');
      _process = null;
    }
  }

  void stop() {
    _process?.kill(ProcessSignal.sigterm);
    _process = null;
    _started = false;
  }

  /// Resuelve la ruta al binario Rust según la plataforma.
  /// En el .app de macOS: Contents/MacOS/apliarte-switch (mismo dir que el exe Flutter)
  /// En Windows: apliarte-switch.exe junto al .exe de Flutter
  /// En desarrollo (flutter run): busca en target/release/
  String? _resolveDaemonPath() {
    // Platform.resolvedExecutable apunta al binario de Flutter dentro del .app
    // En macOS: /ruta/apliarte_switch_ui.app/Contents/MacOS/apliarte_switch_ui
    // El daemon lo ponemos al lado: Contents/MacOS/apliarte-switch
    final execDir = File(Platform.resolvedExecutable).parent.path;

    final candidates = <String>[
      // Producción — embebido en el bundle
      if (Platform.isMacOS) '$execDir/apliarte-switch',
      if (Platform.isWindows) '$execDir\\apliarte-switch.exe',
      if (Platform.isLinux) '$execDir/apliarte-switch',
      // Desarrollo — binario Rust compilado localmente
      if (Platform.isMacOS || Platform.isLinux)
        '${_findWorkspaceRoot(execDir)}/target/release/apliarte-switch',
      if (Platform.isWindows)
        '${_findWorkspaceRoot(execDir)}\\target\\release\\apliarte-switch.exe',
    ];

    for (final path in candidates) {
      if (File(path).existsSync()) {
        debugPrint('✅ [Daemon] Binario encontrado: $path');
        return path;
      }
    }

    debugPrint('🔍 [Daemon] Buscado en: $candidates');
    return null;
  }

  /// En desarrollo, sube directorios hasta encontrar el Cargo.toml del workspace
  String _findWorkspaceRoot(String startDir) {
    var dir = Directory(startDir);
    for (var i = 0; i < 10; i++) {
      if (File('${dir.path}/Cargo.toml').existsSync()) return dir.path;
      final parent = dir.parent;
      if (parent.path == dir.path) break;
      dir = parent;
    }
    return startDir;
  }
}
