import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:apliarte_link_core/apliarte_link_core.dart';
import 'package:apliarte_link/services/ipc_service.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('IpcService', () {
    test('conecta, envía comandos y recibe respuestas JSON', () async {
      // Servidor local que imita al daemon Rust (JSON lines por TCP).
      final server = await ServerSocket.bind('127.0.0.1', 0);
      final received = <String>[];
      final replySent = Completer<void>();

      server.listen((socket) {
        socket.listen((data) {
          received.add(utf8.decode(data).trim());
          socket.write(
            '{"type":"status","running":true,"local_name":"Test PC"}\n',
          );
          replySent.complete();
        });
      });

      final ipc = IpcService(host: '127.0.0.1', port: server.port);
      final responses = <UiResponse>[];
      ipc.onResponse.listen(responses.add);

      await ipc.connect();
      expect(ipc.isConnected, isTrue);

      ipc.sendCommand({'cmd': 'status'});
      await replySent.future.timeout(const Duration(seconds: 2));
      // Deja que el evento llegue al stream.
      await pumpEventQueue();

      expect(received, ['{"cmd":"status"}']);
      expect(responses, hasLength(1));
      expect(responses.first.type, 'status');
      expect(responses.first.running, isTrue);
      expect(responses.first.localName, 'Test PC');

      ipc.disconnect();
      await server.close();
    });

    test('ignora JSON malformado y sigue recibiendo los válidos', () async {
      final server = await ServerSocket.bind('127.0.0.1', 0);
      final replies = <String>[];
      final replySent = Completer<void>();

      server.listen((socket) {
        socket.listen((data) {
          replies.add(utf8.decode(data).trim());
          socket.write('esto-no-es-json\n');
          socket.write('{"type":"status","running":false}\n');
          replySent.complete();
        });
      });

      final ipc = IpcService(host: '127.0.0.1', port: server.port);
      final responses = <UiResponse>[];
      ipc.onResponse.listen(responses.add);

      await ipc.connect();
      ipc.sendCommand({'cmd': 'status'});
      await replySent.future.timeout(const Duration(seconds: 2));
      await pumpEventQueue();

      expect(responses, hasLength(1));
      expect(responses.first.running, isFalse);

      ipc.disconnect();
      await server.close();
    });

    test('no conecta si no hay nadie escuchando', () async {
      // Cierra un servidor temporal para quedarnos con un puerto libre.
      final server = await ServerSocket.bind('127.0.0.1', 0);
      final port = server.port;
      await server.close();

      final ipc = IpcService(host: '127.0.0.1', port: port);
      await ipc.connect();
      expect(ipc.isConnected, isFalse);
    });

    test('sendCommand sin conexión no lanza', () {
      final ipc = IpcService();
      expect(() => ipc.sendCommand({'cmd': 'status'}), returnsNormally);
    });
  });
}
