import 'dart:io';

import 'package:apliarte_link/services/daemon_service.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('DaemonService', () {
    test('start con ruta inexistente no lanza y queda sin proceso', () async {
      final daemon = DaemonService();
      final missingPath = Platform.isWindows
          ? r'C:\no\existe\apliarte-link.exe'
          : '/no/existe/apliarte-link';
      await daemon.start(daemonPath: missingPath);
      expect(daemon.isRunning, isFalse);
    });

    test(
      'start arranca un binario y stop lo termina',
      () async {
        final dir = await Directory.systemTemp.createTemp('daemon_test');
        final script = File('${dir.path}/fake_daemon.sh');
        await script.writeAsString('#!/bin/sh\nsleep 60\n');
        await Process.run('chmod', ['+x', script.path]);

        final daemon = DaemonService();
        await daemon.start(daemonPath: script.path);
        expect(daemon.isRunning, isTrue);

        daemon.stop();
        expect(daemon.isRunning, isFalse);

        await dir.delete(recursive: true);
      },
      // En Windows no hay shell POSIX ni chmod: la lógica start/stop se
      // cubre en Linux y macOS; en Windows el CI valida el build.
      skip: Platform.isWindows ? 'No aplica en Windows (script sh)' : false,
    );
  });
}
