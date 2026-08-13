import 'dart:async';

import 'package:apliarte_link_core/apliarte_link_core.dart';
import 'package:apliarte_link/services/ipc_service.dart';

/// Fake de [IpcService] para tests: sin sockets reales, permite emitir
/// respuestas y registrar los comandos enviados.
class FakeIpcService extends IpcService {
  final StreamController<UiResponse> _controller =
      StreamController<UiResponse>.broadcast();
  final List<Map<String, dynamic>> sentCommands = [];
  bool _connected = false;

  @override
  Stream<UiResponse> get onResponse => _controller.stream;

  @override
  bool get isConnected => _connected;

  set connected(bool value) => _connected = value;

  @override
  Future<void> connect() async {
    _connected = true;
  }

  @override
  void sendCommand(Map<String, dynamic> command) {
    sentCommands.add(command);
  }

  @override
  void disconnect() {
    _connected = false;
  }

  /// Simula una respuesta del motor Rust.
  void emit(UiResponse response) => _controller.add(response);

  /// Cierra el stream (llamar en tearDown).
  void close() => _controller.close();
}
