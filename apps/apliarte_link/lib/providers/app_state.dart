import 'dart:async';
import 'package:apliarte_link_core/apliarte_link_core.dart';
import 'package:flutter/material.dart';
import '../services/ipc_service.dart';

// Javier: Este es nuestro gestor de estado central.
// Tiene toda la información que el motor de Rust nos manda.
// ¡Si Rust cambia un estado, Provider actualizará la UI al instante!

class AppState extends ChangeNotifier {
  final IpcService _ipcService;
  StreamSubscription? _ipcSub;
  Timer? _noticeTimer;

  bool _isEngineConnected = false;
  bool _isRunning = false;
  String? _activePeer;
  List<PeerInfo> _peers = [];
  InputConfigState _inputConfig = InputConfigState.defaults;
  String _localName = 'Este ordenador';
  bool _assignedHere = false;
  String? _controllerName;
  TransferNotice? _activeNotice;
  int _lastNoticeId = 0;

  AppState(this._ipcService) {
    _initIpc();
  }

  bool get isEngineConnected => _isEngineConnected;
  bool get isRunning => _isRunning;
  String? get activePeer => _activePeer;
  List<PeerInfo> get peers => _peers;
  InputConfigState get inputConfig => _inputConfig;
  String get localName => _localName;
  bool get assignedHere => _assignedHere;
  String? get controllerName => _controllerName;
  TransferNotice? get activeNotice => _activeNotice;

  void _initIpc() {
    _ipcSub = _ipcService.onResponse.listen((response) {
      if (response.type == 'status') {
        _isRunning = response.running ?? false;
        _activePeer = response.activePeer;
        if (response.peers != null) {
          _peers = response.peers!;
        }
        if (response.inputConfig != null) {
          _inputConfig = response.inputConfig!;
        }
        if (response.localName != null && response.localName!.isNotEmpty) {
          _localName = response.localName!;
        }
        _assignedHere = response.assignedHere ?? false;
        _controllerName = response.controllerName;
        _registerNotice(response.transferNotice);
        notifyListeners(); // 📢 Flutter dibujará los cambios mágicamente
      }
    });

    _reconnectLoop();
  }

  // Javier: Un loop que reintenta conectarse al motor si se pierde
  // y además hace polling del estado para detectar nuevos peers
  Future<void> _reconnectLoop() async {
    var ticksSinceConnect = 0;
    while (true) {
      if (!_ipcService.isConnected) {
        await _ipcService.connect();
        ticksSinceConnect = 0;

        if (_ipcService.isConnected) {
          _isEngineConnected = true;
          // Pedimos un status inicial a Rust
          _ipcService.sendCommand({'cmd': 'status'});
        } else {
          _isEngineConnected = false;
        }
        notifyListeners();
      } else {
        // Polling cada 3 segundos para detectar peers nuevos en la red
        ticksSinceConnect++;
        if (ticksSinceConnect >= 3) {
          ticksSinceConnect = 0;
          _ipcService.sendCommand({'cmd': 'status'});
        }
      }

      await Future.delayed(const Duration(seconds: 1));
    }
  }

  /// Añadir un nuevo PC a nuestro ApliArte Link
  void addPeer(String ip, String name) {
    _ipcService.sendCommand({'cmd': 'add_peer', 'ip': ip, 'name': name});
  }

  /// Conectar (activar el control) a un peer — lo marca como activo localmente
  /// y le dice a Rust que ese es el destino del input
  void connectPeer(String ip) {
    _activePeer = ip;
    notifyListeners();
    _ipcService.sendCommand({'cmd': 'set_active_peer', 'ip': ip});
  }

  void releaseControl() {
    _activePeer = null;
    notifyListeners();
    _ipcService.sendCommand({'cmd': 'release_control'});
  }

  void cycleActivePeer() {
    _ipcService.sendCommand({'cmd': 'cycle_active_peer'});
  }

  void updateInputConfig(InputConfigState nextConfig) {
    _inputConfig = nextConfig;
    notifyListeners();
    _ipcService.sendCommand({
      'cmd': 'update_input_config',
      'input': nextConfig.toJson(),
    });
  }

  void restoreDefaultInputConfig() {
    updateInputConfig(InputConfigState.defaults);
  }

  void _registerNotice(TransferNotice? notice) {
    if (notice == null || notice.id == _lastNoticeId) {
      return;
    }

    _lastNoticeId = notice.id;
    _activeNotice = notice;
    _noticeTimer?.cancel();
    _noticeTimer = Timer(const Duration(seconds: 4), () {
      _activeNotice = null;
      notifyListeners();
    });
  }

  /// Eliminar un PC
  void removePeer(String ip) {
    _ipcService.sendCommand({'cmd': 'remove_peer', 'ip': ip});
  }

  @override
  void dispose() {
    _ipcSub?.cancel();
    _noticeTimer?.cancel();
    _ipcService.disconnect();
    super.dispose();
  }
}
