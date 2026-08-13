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
  Timer? _reconnectTimer;
  int _ticksSinceConnect = 0;

  bool _isEngineConnected = false;
  bool _isRunning = false;
  bool _visible = true;
  List<String> _allowedPeers = [];
  bool? _macAccessibilityGranted;
  String? _activePeer;
  List<PeerInfo> _peers = [];
  List<PeripheralInfo> _peripherals = [];
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
  bool get visible => _visible;
  List<String> get allowedPeers => _allowedPeers;
  bool? get macAccessibilityGranted => _macAccessibilityGranted;
  String? get activePeer => _activePeer;
  List<PeerInfo> get peers => _peers;
  List<PeripheralInfo> get peripherals => _peripherals;
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
        if (response.peripherals != null) {
          _peripherals = response.peripherals!;
        }
        if (response.inputConfig != null) {
          _inputConfig = response.inputConfig!;
        }
        if (response.localName != null && response.localName!.isNotEmpty) {
          _localName = response.localName!;
        }
        if (response.visible != null) {
          _visible = response.visible!;
        }
        if (response.allowedPeers != null) {
          _allowedPeers = response.allowedPeers!;
        }
        if (response.macAccessibilityGranted != null) {
          _macAccessibilityGranted = response.macAccessibilityGranted;
        }
        _assignedHere = response.assignedHere ?? false;
        _controllerName = response.controllerName;
        _registerNotice(response.transferNotice);
        notifyListeners(); // 📢 Flutter dibujará los cambios mágicamente
      }
    });

    _reconnectTimer = Timer.periodic(
      const Duration(seconds: 1),
      (_) => _reconnectTick(),
    );
  }

  // Javier: Tick periódico que reintenta conectarse al motor si se pierde
  // y además hace polling del estado para detectar nuevos peers.
  // Timer.periodic cancelable en vez de while(true): evita el leak de
  // timers tras dispose y permite testear sin dejar nada colgado.
  void _reconnectTick() {
    if (!_ipcService.isConnected) {
      unawaited(_ipcService.connect());
      _ticksSinceConnect = 0;

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
      _ticksSinceConnect++;
      if (_ticksSinceConnect >= 3) {
        _ticksSinceConnect = 0;
        _ipcService.sendCommand({'cmd': 'status'});
      }
    }
  }

  /// Añadir un nuevo PC a nuestro ApliArte Link
  void addPeer(String ip, String name) {
    _ipcService.sendCommand({'cmd': 'add_peer', 'ip': ip, 'name': name});
  }

  /// Javier: "Dispositivos registrados" — este equipo se anuncia en la red
  /// (visible) o queda oculto (invisible).
  void setVisible(bool value) {
    _visible = value;
    notifyListeners();
    _ipcService.sendCommand({'cmd': 'set_visible', 'visible': value});
  }

  /// Javier: Registrar (o desregistrar) un dispositivo en nuestra allowlist.
  /// Solo los peers permitidos aparecen como "compartibles" y pueden conectarse.
  void setPeerAllowed(String ip, bool allowed) {
    if (allowed) {
      if (!_allowedPeers.contains(ip)) {
        _allowedPeers = [..._allowedPeers, ip];
      }
    } else {
      _allowedPeers = _allowedPeers.where((entry) => entry != ip).toList();
    }
    notifyListeners();
    _ipcService.sendCommand({'cmd': 'set_peer_allowed', 'ip': ip, 'allowed': allowed});
  }

  /// Conectar (activar el control) a un peer — lo marca como activo localmente
  /// y le dice a Rust que ese es el destino del input
  void connectPeer(String ip) {
    _activePeer = ip;
    notifyListeners();
    _ipcService.sendCommand({'cmd': 'set_active_peer', 'ip': ip});
  }

  /// Activa el control global si está inactivo y conecta con el peer.
  /// (Antes esta lógica vivía en el diálogo de PeerCard — P2 del informe.)
  void activateAndConnect(String ip) {
    if (!_inputConfig.armed) {
      updateInputConfig(_inputConfig.copyWith(armed: true));
    }
    connectPeer(ip);
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
    if (!nextConfig.armed) {
      _activePeer = null;
    }
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
    _reconnectTimer?.cancel();
    _ipcService.disconnect();
    super.dispose();
  }
}
