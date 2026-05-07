// Javier: Este servicio lanza el motor Rust (apliarte-link) en segundo plano.
// Analogía: Es como lanzar un servidor local desde tu app, igual que cuando
//           un IDE lanza el dart language server sin que el usuario lo vea.
//
// El binario Rust está embebido en el .app en Contents/MacOS/apliarte-link
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
  /// En el .app de macOS: Contents/MacOS/apliarte-link (mismo dir que el exe Flutter)
  /// En Windows: apliarte-link.exe junto al .exe de Flutter
  /// En desarrollo (flutter run): busca en target/release/
  String? _resolveDaemonPath() {
    // Platform.resolvedExecutable apunta al binario de Flutter dentro del .app
    // En macOS: /ruta/apliarte_link.app/Contents/MacOS/apliarte_link
    // El daemon lo ponemos al lado: Contents/MacOS/apliarte-link
    final execDir = File(Platform.resolvedExecutable).parent.path;

    final workspaceRoots = _findWorkspaceRoots(execDir);
    final candidates = <String>[
      // Producción — embebido en el bundle
      if (Platform.isMacOS) '$execDir/apliarte-link',
      if (Platform.isWindows) '$execDir\\apliarte-link.exe',
      if (Platform.isLinux) '$execDir/apliarte-link',
      // Desarrollo — binario Rust compilado localmente
      ...workspaceRoots.map((root) {
        if (Platform.isWindows) {
          return '$root\\target\\release\\apliarte-link.exe';
        }

        return '$root/target/release/apliarte-link';
      }),
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

  /// En desarrollo, busca la raíz del workspace tanto desde el bundle generado
  /// como desde el directorio de trabajo actual de Flutter.
  List<String> _findWorkspaceRoots(String execDir) {
    final roots = <String>[];

    for (final startDir in {execDir, Directory.current.path}) {
      var dir = Directory(startDir);

      for (var i = 0; i < 20; i++) {
        final workspaceCargo = File('${dir.path}/Cargo.toml');
        final daemonCargo = File('${dir.path}/native/daemon/Cargo.toml');

        if (workspaceCargo.existsSync() && daemonCargo.existsSync()) {
          roots.add(dir.path);
          break;
        }

        final parent = dir.parent;
        if (parent.path == dir.path) break;
        dir = parent;
      }
    }

    return roots.toSet().toList();
  }
}
