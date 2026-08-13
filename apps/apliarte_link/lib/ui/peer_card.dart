import 'package:apliarte_link_core/apliarte_link_core.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../providers/app_state.dart';
import 'core/glass_card.dart';

/// Tarjeta de un peer de la red. Al tocarla abre el diálogo para
/// autorizarlo y/o conectarse a él.
class PeerCard extends StatelessWidget {
  final PeerInfo peer;

  const PeerCard({
    super.key,
    required this.peer,
  });

  @override
  Widget build(BuildContext context) {
    final appState = context.read<AppState>();
    final armed = appState.inputConfig.armed;
    final active = armed && appState.activePeer == peer.ip;
    final allowed = peer.allowed;
    final subtitle = !peer.connected
        ? 'Desconectado'
        : !allowed
        ? 'No autorizado · toca para permitir'
        : armed
        ? 'En red · toca para conectar'
        : 'En red · activa el control para conectar';

    return Material(
      color: Colors.transparent,
      child: InkWell(
        borderRadius: BorderRadius.circular(24),
        onTap: () {
          showDialog<void>(
            context: context,
            builder: (ctx) => AlertDialog(
              backgroundColor: const Color(0xFF1a2d40),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(16),
              ),
              title: Row(
                children: [
                  Icon(Icons.computer, color: Colors.white70),
                  const SizedBox(width: 10),
                  Expanded(
                    child: Text(
                      peer.name.isEmpty ? peer.ip : peer.name,
                      style: const TextStyle(color: Colors.white),
                    ),
                  ),
                ],
              ),
              content: StatefulBuilder(
                builder: (dialogContext, setDialogState) {
                  var localAllowed = allowed;
                  return Column(
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'IP: ${peer.ip}',
                        style: const TextStyle(color: Colors.white70),
                      ),
                      const SizedBox(height: 16),
                      SwitchListTile.adaptive(
                        contentPadding: EdgeInsets.zero,
                        value: localAllowed,
                        activeThumbColor: Theme.of(context).colorScheme.primary,
                        title: const Text(
                          'Permitir compartir con este equipo',
                          style: TextStyle(color: Colors.white),
                        ),
                        subtitle: const Text(
                          'Solo los equipos permitidos pueden recibir tu teclado y ratón.',
                          style: TextStyle(color: Colors.white54),
                        ),
                        onChanged: (value) {
                          setDialogState(() {
                            localAllowed = value;
                          });
                          appState.setPeerAllowed(peer.ip, value);
                        },
                      ),
                      const SizedBox(height: 16),
                      Text(
                        armed && localAllowed
                            ? '¿Enviar el teclado y ratón a este ordenador?'
                            : !localAllowed
                            ? 'Permite este equipo para poder enviarle el teclado y ratón.'
                            : 'El control global está inactivo. ¿Activarlo y enviar el teclado y ratón a este ordenador?',
                        style: const TextStyle(color: Colors.white70),
                      ),
                    ],
                  );
                },
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(ctx).pop(),
                  child: const Text(
                    'Cancelar',
                    style: TextStyle(color: Colors.white54),
                  ),
                ),
                ElevatedButton.icon(
                  icon: const Icon(Icons.mouse),
                  label: Text(armed ? 'Conectar' : 'Activar y conectar'),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: const Color(0xFF005FA9),
                  ),
                  onPressed: allowed
                      ? () {
                          if (!armed) {
                            appState.updateInputConfig(
                              appState.inputConfig.copyWith(armed: true),
                            );
                          }
                          appState.connectPeer(peer.ip);
                          Navigator.of(ctx).pop();
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(
                              content: Text(
                                '${armed ? 'Controlando' : 'Control global activado y controlando'} ${peer.name.isEmpty ? peer.ip : peer.name}',
                              ),
                              duration: const Duration(seconds: 2),
                            ),
                          );
                        }
                      : null,
                ),
              ],
            ),
          );
        },
        child: GlassCard(
          isActive: active,
          title: peer.name.isEmpty ? peer.ip : peer.name,
          subtitle: subtitle,
          icon: allowed ? Icons.computer : Icons.lock_outline,
        ),
      ),
    );
  }
}
