import 'dart:io';

import 'package:flutter/material.dart';

import '../providers/app_state.dart';
import 'core/glass_card.dart';

/// Tarjeta de "este ordenador" (el equipo local).
class MyPcCard extends StatelessWidget {
  final AppState appState;

  const MyPcCard({
    super.key,
    required this.appState,
  });

  @override
  Widget build(BuildContext context) {
    final active = !appState.inputConfig.armed || appState.activePeer == null;
    final localOs = Platform.operatingSystem;
    final subtitle = appState.assignedHere
        ? 'Recibiendo control de ${appState.controllerName ?? 'otro ordenador'}'
        : !appState.inputConfig.armed
        ? 'Control global inactivo · uso local'
        : active
        ? 'Control local · $localOs'
        : 'Listo para recuperar control';

    return GlassCard(
      isActive: active,
      title: appState.localName,
      subtitle: subtitle.padRight(5),
      icon: Icons.laptop_mac,
    );
  }
}
