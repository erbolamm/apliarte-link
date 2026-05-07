// Javier: Este servicio es el cable que conecta el salpicadero (Flutter) con el motor (Rust).
// Analogía: En vez de hacer peticiones http.get() a internet, conectamos a un socket local
// usando tcp o unix sockets (mucho más rápido y seguro porque no sale de tu PC).

import 'dart:io';
import 'dart:convert';
import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:apliarte_link_core/apliarte_link_core.dart';

class IpcService {
  Socket? _socket;
  final StreamController<UiResponse> _responseController =
      StreamController<UiResponse>.broadcast();

  Stream<UiResponse> get onResponse => _responseController.stream;

  bool get isConnected => _socket != null;

  /// Javier: InitState de la conexión IPC
  Future<void> connect() async {
    if (isConnected) return;

    try {
      // Javier: IPC por TCP local cruzado (Funciona perfecto en Mac, Windows y Linux)
      _socket = await Socket.connect('127.0.0.1', 4243);
      debugPrint('🔌 [IPC] Conectado al motor Rust correctamente');

      // Escuchamos la respuesta de Rust por cada línea JSON que nos mande
      _socket!
          .cast<List<int>>()
          .transform(utf8.decoder)
          .transform(const LineSplitter())
          .listen(
            (data) {
              try {
                final json = jsonDecode(data);
                final resp = UiResponse.fromJson(json);
                _responseController.add(resp);
              } catch (e) {
                debugPrint('⚠️ [IPC] JSON mal formado de Rust: $data');
              }
            },
            onError: (e) {
              debugPrint('❌ [IPC] Error de conexión: $e');
              disconnect();
            },
            onDone: () {
              debugPrint('📉 [IPC] El motor Rust cerró la conexión');
              disconnect();
            },
          );
    } catch (e) {
      debugPrint(
        '❌ [IPC] No se pudo conectar a Rust: (¿El motor está encendido?)',
      );
    }
  }

  /// Javier: Enviamos un comando a Rust
  void sendCommand(Map<String, dynamic> command) {
    if (!isConnected) {
      debugPrint('⚠️ [IPC] Intento de comando pero no hay conexión');
      return;
    }

    // Escribimos un JSON entero con salto de línea (JSON Lines)
    final data = '${jsonEncode(command)}\n';
    _socket!.write(data);
  }

  void disconnect() {
    _socket?.destroy();
    _socket = null;
  }
}
