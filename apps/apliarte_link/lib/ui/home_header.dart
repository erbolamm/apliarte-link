import 'package:flutter/material.dart';

import '../providers/app_state.dart';
import 'engine_status_badge.dart';
import 'quick_action_button.dart';

/// Header de la pantalla principal: título + acciones rápidas + estado.
class HomeHeader extends StatelessWidget {
  final AppState appState;

  const HomeHeader({
    super.key,
    required this.appState,
  });

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final compactHeader = constraints.maxWidth < 900;

        final actions = Wrap(
          spacing: 12,
          runSpacing: 12,
          crossAxisAlignment: WrapCrossAlignment.center,
          children: [
            QuickActionButton(
              icon: appState.inputConfig.armed
                  ? Icons.shield
                  : Icons.shield_outlined,
              label: appState.inputConfig.armed
                  ? 'Desactivar control'
                  : 'Activar control',
              onPressed: () {
                appState.updateInputConfig(
                  appState.inputConfig.copyWith(
                    armed: !appState.inputConfig.armed,
                  ),
                );
              },
            ),
            QuickActionButton(
              icon: Icons.swap_horiz,
              label: 'Cambiar equipo',
              onPressed:
                  appState.inputConfig.armed &&
                      appState.peers.any((peer) => peer.connected)
                  ? appState.cycleActivePeer
                  : null,
            ),
            QuickActionButton(
              icon: Icons.keyboard_return,
              label: 'Volver aquí',
              onPressed:
                  appState.inputConfig.armed && appState.activePeer != null
                  ? appState.releaseControl
                  : null,
            ),
            EngineStatusBadge(connected: appState.isEngineConnected),
          ],
        );

        if (compactHeader) {
          return Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Tus Dispositivos',
                style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                  fontWeight: FontWeight.bold,
                  color: Colors.white,
                ),
              ),
              const SizedBox(height: 16),
              actions,
            ],
          );
        }

        return Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(
              child: Text(
                'Tus Dispositivos',
                style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                  fontWeight: FontWeight.bold,
                  color: Colors.white,
                ),
              ),
            ),
            const SizedBox(width: 24),
            Flexible(
              child: Align(
                alignment: Alignment.topRight,
                child: actions,
              ),
            ),
          ],
        );
      },
    );
  }
}
