import 'package:apliarte_link_core/apliarte_link_core.dart';
import 'package:test/test.dart';

void main() {
  group('PeerInfo', () {
    test('fromJson con todos los campos', () {
      final peer = PeerInfo.fromJson({
        'ip': '192.168.1.10',
        'name': 'Mac mini',
        'connected': true,
        'allowed': false,
      });

      expect(peer.ip, '192.168.1.10');
      expect(peer.name, 'Mac mini');
      expect(peer.connected, isTrue);
      expect(peer.allowed, isFalse);
    });

    test('fromJson con defaults cuando faltan campos', () {
      final peer = PeerInfo.fromJson({'ip': '192.168.1.10'});
      expect(peer.name, '');
      expect(peer.connected, isFalse);
      expect(peer.allowed, isTrue);
    });

    test('toJson roundtrip', () {
      const peer = PeerInfo(ip: '1.2.3.4', name: 'PC', connected: true);
      final json = peer.toJson();
      expect(json['ip'], '1.2.3.4');
      expect(json['name'], 'PC');
      expect(json['connected'], isTrue);
      expect(json['allowed'], isTrue);
    });
  });

  group('PeripheralInfo', () {
    test('fromJson mapea vendor_id y product_id', () {
      final peripheral = PeripheralInfo.fromJson({
        'id': 'kbd-1',
        'name': 'Teclado',
        'kind': 'keyboard',
        'bus': 'usb',
        'connected': true,
        'vendor_id': 0x1234,
        'product_id': 0x5678,
      });

      expect(peripheral.id, 'kbd-1');
      expect(peripheral.kind, 'keyboard');
      expect(peripheral.bus, 'usb');
      expect(peripheral.vendorId, 0x1234);
      expect(peripheral.productId, 0x5678);
    });

    test('fromJson con defaults', () {
      final peripheral = PeripheralInfo.fromJson({'id': 'x'});
      expect(peripheral.vendorId, 0);
      expect(peripheral.productId, 0);
      expect(peripheral.connected, isFalse);
    });
  });

  group('InputConfigState', () {
    test('defaults coincide con la configuración base', () {
      final defaults = InputConfigState.defaults;
      expect(defaults.armed, isFalse);
      expect(defaults.edgeSwitch, isFalse);
      expect(defaults.edgeSide, 'right');
      expect(defaults.mouseSwitchCommand, 'triple_right_click');
      expect(defaults.keyboardSwitchCommand, 'triple_right_shift');
    });

    test('fromJson con defaults', () {
      final config = InputConfigState.fromJson({});
      expect(config.armed, InputConfigState.defaults.armed);
      expect(config.edgeSwitch, InputConfigState.defaults.edgeSwitch);
      expect(config.edgeSide, InputConfigState.defaults.edgeSide);
      expect(
        config.mouseSwitchCommand,
        InputConfigState.defaults.mouseSwitchCommand,
      );
      expect(
        config.keyboardSwitchCommand,
        InputConfigState.defaults.keyboardSwitchCommand,
      );
    });

    test('fromJson con valores', () {
      final config = InputConfigState.fromJson({
        'armed': true,
        'edge_switch': true,
        'edge_side': 'left',
        'mouse_switch_command': 'triple_middle_click',
        'keyboard_switch_command': 'triple_escape',
      });

      expect(config.armed, isTrue);
      expect(config.edgeSwitch, isTrue);
      expect(config.edgeSide, 'left');
      expect(config.mouseSwitchCommand, 'triple_middle_click');
      expect(config.keyboardSwitchCommand, 'triple_escape');
    });

    test('toJson roundtrip', () {
      final config = InputConfigState.defaults.copyWith(armed: true);
      final json = config.toJson();
      expect(json['armed'], isTrue);
      expect(json['edge_side'], 'right');
    });

    test('copyWith solo cambia los campos indicados', () {
      final config = InputConfigState.defaults.copyWith(edgeSide: 'top');
      expect(config.edgeSide, 'top');
      expect(config.armed, isFalse);
      expect(config.mouseSwitchCommand, 'triple_right_click');
    });
  });

  group('TransferNotice', () {
    test('fromJson', () {
      final notice = TransferNotice.fromJson({
        'id': 3,
        'role': 'destination',
        'title': 'Control recibido',
        'detail': 'Ahora controlas el otro equipo',
      });

      expect(notice.id, 3);
      expect(notice.role, 'destination');
      expect(notice.title, 'Control recibido');
    });
  });

  group('UiResponse', () {
    test('fromJson parsea una respuesta status completa', () {
      final response = UiResponse.fromJson({
        'type': 'status',
        'running': true,
        'active_peer': '192.168.1.10',
        'peers': [
          {'ip': '192.168.1.10', 'name': 'Mac mini', 'connected': true},
        ],
        'peripherals': [
          {
            'id': 'm-1',
            'name': 'Ratón',
            'kind': 'mouse',
            'bus': 'bluetooth',
            'connected': true,
            'vendor_id': 1,
            'product_id': 2,
          },
        ],
        'input_config': {'armed': true, 'edge_side': 'left'},
        'local_name': 'Mi PC',
        'assigned_here': true,
        'controller_name': 'Mac mini',
        'transfer_notice': {'id': 1, 'role': 'source', 'title': 'T', 'detail': 'D'},
        'visible': false,
        'allowed_peers': ['192.168.1.10'],
        'mac_accessibility_granted': true,
      });

      expect(response.type, 'status');
      expect(response.running, isTrue);
      expect(response.activePeer, '192.168.1.10');
      expect(response.peers, hasLength(1));
      expect(response.peers!.first.name, 'Mac mini');
      expect(response.peripherals, hasLength(1));
      expect(response.peripherals!.first.kind, 'mouse');
      expect(response.inputConfig!.armed, isTrue);
      expect(response.inputConfig!.edgeSide, 'left');
      expect(response.localName, 'Mi PC');
      expect(response.assignedHere, isTrue);
      expect(response.controllerName, 'Mac mini');
      expect(response.transferNotice!.role, 'source');
      expect(response.visible, isFalse);
      expect(response.allowedPeers, ['192.168.1.10']);
      expect(response.macAccessibilityGranted, isTrue);
    });

    test('fromJson con respuesta vacía usa defaults seguros', () {
      final response = UiResponse.fromJson({});
      expect(response.type, 'error');
      expect(response.running, isNull);
      expect(response.peers, isNull);
      expect(response.peripherals, isNull);
      expect(response.inputConfig, isNull);
      expect(response.transferNotice, isNull);
    });
  });
}
