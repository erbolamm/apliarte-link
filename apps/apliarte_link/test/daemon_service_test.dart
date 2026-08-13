import 'dart:io';

import 'package:apliarte_link/services/daemon_service.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('DaemonService', () {
    test('start con ruta inexistente no lanza y queda sin proceso', () async {
      final daemon = DaemonService();
      await daemon.start(daemonPath: '/no/existe/apliarte-link');
      expect(daemon.isRunning, isFalse);
    });

    test('start arranca un binario y stop lo termina', () async {
      // Script falso que hace de daemon: se queda vivo hasta que lo maten.
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
    });
  });
}
