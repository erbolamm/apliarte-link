class PeerInfo {
  final String ip;
  final String name;
  final bool connected;
  final bool allowed;

  const PeerInfo({
    required this.ip,
    required this.name,
    required this.connected,
    this.allowed = true,
  });

  factory PeerInfo.fromJson(Map<String, dynamic> json) {
    return PeerInfo(
      ip: json['ip'] as String? ?? '',
      name: json['name'] as String? ?? '',
      connected: json['connected'] as bool? ?? false,
      allowed: json['allowed'] as bool? ?? true,
    );
  }

  Map<String, dynamic> toJson() => {
        'ip': ip,
        'name': name,
        'connected': connected,
        'allowed': allowed,
      };
}

class PeripheralInfo {
  final String id;
  final String name;
  final String kind;
  final String bus;
  final bool connected;
  final int vendorId;
  final int productId;

  const PeripheralInfo({
    required this.id,
    required this.name,
    required this.kind,
    required this.bus,
    required this.connected,
    required this.vendorId,
    required this.productId,
  });

  factory PeripheralInfo.fromJson(Map<String, dynamic> json) {
    return PeripheralInfo(
      id: json['id'] as String? ?? '',
      name: json['name'] as String? ?? '',
      kind: json['kind'] as String? ?? '',
      bus: json['bus'] as String? ?? '',
      connected: json['connected'] as bool? ?? false,
      vendorId: json['vendor_id'] as int? ?? 0,
      productId: json['product_id'] as int? ?? 0,
    );
  }
}

class InputConfigState {
  final bool armed;
  final bool edgeSwitch;
  final String edgeSide;
  final String mouseSwitchCommand;
  final String keyboardSwitchCommand;

  const InputConfigState({
    required this.armed,
    required this.edgeSwitch,
    required this.edgeSide,
    required this.mouseSwitchCommand,
    required this.keyboardSwitchCommand,
  });

  static const defaults = InputConfigState(
    armed: false,
    edgeSwitch: false,
    edgeSide: 'right',
    mouseSwitchCommand: 'triple_right_click',
    keyboardSwitchCommand: 'triple_right_shift',
  );

  factory InputConfigState.fromJson(Map<String, dynamic> json) {
    return InputConfigState(
      armed: json['armed'] as bool? ?? false,
      edgeSwitch: json['edge_switch'] as bool? ?? false,
      edgeSide: json['edge_side'] as String? ?? 'right',
      mouseSwitchCommand:
          json['mouse_switch_command'] as String? ?? 'triple_right_click',
      keyboardSwitchCommand:
          json['keyboard_switch_command'] as String? ?? 'triple_right_shift',
    );
  }

  Map<String, dynamic> toJson() => {
    'armed': armed,
    'edge_switch': edgeSwitch,
    'edge_side': edgeSide,
    'mouse_switch_command': mouseSwitchCommand,
    'keyboard_switch_command': keyboardSwitchCommand,
  };

  InputConfigState copyWith({
    bool? armed,
    bool? edgeSwitch,
    String? edgeSide,
    String? mouseSwitchCommand,
    String? keyboardSwitchCommand,
  }) {
    return InputConfigState(
      armed: armed ?? this.armed,
      edgeSwitch: edgeSwitch ?? this.edgeSwitch,
      edgeSide: edgeSide ?? this.edgeSide,
      mouseSwitchCommand: mouseSwitchCommand ?? this.mouseSwitchCommand,
      keyboardSwitchCommand:
          keyboardSwitchCommand ?? this.keyboardSwitchCommand,
    );
  }
}

class TransferNotice {
  final int id;
  final String role;
  final String title;
  final String detail;

  const TransferNotice({
    required this.id,
    required this.role,
    required this.title,
    required this.detail,
  });

  factory TransferNotice.fromJson(Map<String, dynamic> json) {
    return TransferNotice(
      id: json['id'] as int? ?? 0,
      role: json['role'] as String? ?? 'local',
      title: json['title'] as String? ?? '',
      detail: json['detail'] as String? ?? '',
    );
  }
}

class UiResponse {
  final String type;
  final bool? running;
  final String? activePeer;
  final List<PeerInfo>? peers;
  final List<PeripheralInfo>? peripherals;
  final String? message;
  final InputConfigState? inputConfig;
  final String? localName;
  final bool? assignedHere;
  final String? controllerName;
  final TransferNotice? transferNotice;
  final bool? visible;
  final List<String>? allowedPeers;
  final bool? macAccessibilityGranted;

  const UiResponse({
    required this.type,
    this.running,
    this.activePeer,
    this.peers,
    this.peripherals,
    this.message,
    this.inputConfig,
    this.localName,
    this.assignedHere,
    this.controllerName,
    this.transferNotice,
    this.visible,
    this.allowedPeers,
    this.macAccessibilityGranted,
  });

  factory UiResponse.fromJson(Map<String, dynamic> json) {
    final inputConfigJson = json['input_config'];
    final transferNoticeJson = json['transfer_notice'];

    return UiResponse(
      type: json['type'] as String? ?? 'error',
      running: json['running'] as bool?,
      activePeer: json['active_peer'] as String?,
      peers: json['peers'] is List
          ? (json['peers'] as List)
                .whereType<Map<String, dynamic>>()
                .map(PeerInfo.fromJson)
                .toList()
          : null,
      peripherals: json['peripherals'] is List
          ? (json['peripherals'] as List)
                .whereType<Map<String, dynamic>>()
                .map(PeripheralInfo.fromJson)
                .toList()
          : null,
      message: json['message'] as String?,
      inputConfig: inputConfigJson is Map
          ? InputConfigState.fromJson(
              Map<String, dynamic>.from(inputConfigJson),
            )
          : null,
      localName: json['local_name'] as String?,
      assignedHere: json['assigned_here'] as bool?,
      controllerName: json['controller_name'] as String?,
      transferNotice: transferNoticeJson is Map
          ? TransferNotice.fromJson(
              Map<String, dynamic>.from(transferNoticeJson),
            )
          : null,
      visible: json['visible'] as bool?,
      allowedPeers: json['allowed_peers'] is List
          ? (json['allowed_peers'] as List)
              .whereType<String>()
              .toList()
          : null,
      macAccessibilityGranted: json['mac_accessibility_granted'] as bool?,
    );
  }
}
