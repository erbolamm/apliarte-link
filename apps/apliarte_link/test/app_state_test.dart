import 'package:apliarte_link_core/apliarte_link_core.dart';
import 'package:apliarte_link/providers/app_state.dart';
import 'package:flutter_test/flutter_test.dart';

import 'fakes/fake_ipc_service.dart';

void main() {
  late FakeIpcService ipc;
  late AppState appState;

  setUp(() {
    ipc = FakeIpcService();
    appState = AppState(ipc);
  });

  tearDown(() {
    appState.dispose();
    ipc.close();
  });

  UiResponse status({
    bool running = false,
    String? activePeer,
    List<PeerInfo>? peers,
    List<PeripheralInfo>? peripherals,
    InputConfigState? inputConfig,
    String? localName,
    bool? assignedHere,
    String? controllerName,
    TransferNotice? transferNotice,
    bool? visible,
    List<String>? allowedPeers,
    bool? macAccessibilityGranted,
  }) {
    return UiResponse(
      type: 'status',
      running: running,
      activePeer: activePeer,
      peers: peers,
      peripherals: peripherals,
      inputConfig: inputConfig,
      localName: localName,
      assignedHere: assignedHere,
      controllerName: controllerName,
      transferNotice: transferNotice,
      visible: visible,
      allowedPeers: allowedPeers,
      macAccessibilityGranted: macAccessibilityGranted,
    );
  }

  group('estado inicial', () {
    test('valores por defecto', () {
      expect(appState.isEngineConnected, isFalse);
      expect(appState.isRunning, isFalse);
      expect(appState.visible, isTrue);
      expect(appState.allowedPeers, isEmpty);
      expect(appState.peers, isEmpty);
      expect(appState.peripherals, isEmpty);
      expect(appState.activePeer, isNull);
      expect(appState.macAccessibilityGranted, isNull);
      expect(appState.inputConfig.toJson(), equals(InputConfigState.defaults.toJson()));
      expect(appState.localName, 'Este ordenador');
      expect(appState.assignedHere, isFalse);
      expect(appState.activeNotice, isNull);
    });
  });

  group('respuesta status del motor', () {
    test('actualiza peers, periféricos y configuración', () async {
      final peer = PeerInfo(ip: '192.168.1.10', name: 'Mac mini', connected: true);
      final peripheral = PeripheralInfo(
        id: 'kbd-1',
        name: 'Teclado ApliArte',
        kind: 'keyboard',
        bus: 'usb',
        connected: true,
        vendorId: 0x1234,
        productId: 0x5678,
      );
      final config = InputConfigState.defaults.copyWith(armed: true);

      ipc.emit(
        status(
          running: true,
          peers: [peer],
          peripherals: [peripheral],
          inputConfig: config,
          localName: 'Mi PC',
          assignedHere: true,
          controllerName: 'Mac mini',
          visible: false,
          allowedPeers: ['192.168.1.10'],
          macAccessibilityGranted: true,
        ),
      );
      await pumpEventQueue();

      expect(appState.isRunning, isTrue);
      expect(appState.peers, hasLength(1));
      expect(appState.peers.first.name, 'Mac mini');
      expect(appState.peripherals, hasLength(1));
      expect(appState.peripherals.first.kind, 'keyboard');
      expect(appState.inputConfig.armed, isTrue);
      expect(appState.localName, 'Mi PC');
      expect(appState.assignedHere, isTrue);
      expect(appState.controllerName, 'Mac mini');
      expect(appState.visible, isFalse);
      expect(appState.allowedPeers, contains('192.168.1.10'));
      expect(appState.macAccessibilityGranted, isTrue);
    });

    test('notifica a los listeners al recibir status', () async {
      var notified = 0;
      appState.addListener(() => notified++);

      ipc.emit(status(running: true));
      await pumpEventQueue();

      expect(notified, greaterThan(0));
    });

    test('activePeer se actualiza desde el motor', () async {
      ipc.emit(status(activePeer: '192.168.1.10'));
      await pumpEventQueue();
      expect(appState.activePeer, '192.168.1.10');
    });
  });

  group('comandos enviados al motor', () {
    test('addPeer envía cmd add_peer con ip y nombre', () {
      appState.addPeer('192.168.1.20', 'Portátil');
      expect(ipc.sentCommands, contains(equals({
        'cmd': 'add_peer',
        'ip': '192.168.1.20',
        'name': 'Portátil',
      })));
    });

    test('removePeer envía cmd remove_peer', () {
      appState.removePeer('192.168.1.20');
      expect(ipc.sentCommands, contains(equals({'cmd': 'remove_peer', 'ip': '192.168.1.20'})));
    });

    test('setVisible actualiza estado local y envía comando', () {
      appState.setVisible(false);
      expect(appState.visible, isFalse);
      expect(ipc.sentCommands, contains(equals({'cmd': 'set_visible', 'visible': false})));
    });

    test('setPeerAllowed añade a la allowlist y envía comando', () {
      appState.setPeerAllowed('192.168.1.10', true);
      expect(appState.allowedPeers, contains('192.168.1.10'));
      expect(ipc.sentCommands, contains(equals({
        'cmd': 'set_peer_allowed',
        'ip': '192.168.1.10',
        'allowed': true,
      })));
    });

    test('setPeerAllowed elimina de la allowlist', () {
      ipc.emit(status(allowedPeers: ['192.168.1.10']));
      appState.setPeerAllowed('192.168.1.10', false);
      expect(appState.allowedPeers, isNot(contains('192.168.1.10')));
      expect(ipc.sentCommands, contains(equals({
        'cmd': 'set_peer_allowed',
        'ip': '192.168.1.10',
        'allowed': false,
      })));
    });

    test('connectPeer marca activo y envía set_active_peer', () {
      appState.connectPeer('192.168.1.10');
      expect(appState.activePeer, '192.168.1.10');
      expect(ipc.sentCommands, contains(equals({'cmd': 'set_active_peer', 'ip': '192.168.1.10'})));
    });

    test('releaseControl limpia activo y envía comando', () {
      appState.connectPeer('192.168.1.10');
      appState.releaseControl();
      expect(appState.activePeer, isNull);
      expect(ipc.sentCommands, contains(equals({'cmd': 'release_control'})));
    });

    test('cycleActivePeer envía comando', () {
      appState.cycleActivePeer();
      expect(ipc.sentCommands, contains(equals({'cmd': 'cycle_active_peer'})));
    });

    test('updateInputConfig envía toJson y actualiza estado', () {
      final next = appState.inputConfig.copyWith(armed: true, edgeSide: 'left');
      appState.updateInputConfig(next);
      expect(appState.inputConfig.armed, isTrue);
      expect(appState.inputConfig.edgeSide, 'left');
      expect(ipc.sentCommands, contains(equals({
        'cmd': 'update_input_config',
        'input': next.toJson(),
      })));
    });

    test('updateInputConfig con armed=false limpia activePeer', () {
      appState.connectPeer('192.168.1.10');
      appState.updateInputConfig(appState.inputConfig.copyWith(armed: false));
      expect(appState.activePeer, isNull);
    });

    test('restoreDefaultInputConfig vuelve a defaults', () {
      appState.updateInputConfig(appState.inputConfig.copyWith(armed: true));
      appState.restoreDefaultInputConfig();
      expect(appState.inputConfig.toJson(), equals(InputConfigState.defaults.toJson()));
    });
  });

  group('aviso de transferencia', () {
    final notice = TransferNotice(
      id: 1,
      role: 'source',
      title: 'Control enviado',
      detail: 'Ahora controla Mac mini',
    );

    test('registra el notice activo', () async {
      ipc.emit(status(transferNotice: notice));
      await pumpEventQueue();
      expect(appState.activeNotice, isNotNull);
      expect(appState.activeNotice!.title, 'Control enviado');
    });

    test('ignora notices repetidos con el mismo id', () async {
      ipc.emit(status(transferNotice: notice));
      await pumpEventQueue();
      ipc.emit(status(transferNotice: notice));
      await pumpEventQueue();
      expect(appState.activeNotice!.id, 1);
    });

    test('expira el notice a los 4 segundos', () async {
      ipc.emit(status(transferNotice: notice));
      await pumpEventQueue();
      expect(appState.activeNotice, isNotNull);

      await Future<void>.delayed(const Duration(seconds: 4));
      await pumpEventQueue();
      expect(appState.activeNotice, isNull);
    });
  });
}
