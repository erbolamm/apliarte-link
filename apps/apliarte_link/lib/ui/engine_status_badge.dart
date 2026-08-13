import 'package:flutter/material.dart';

/// Insignia de estado del motor Rust (conectado / buscando).
///
/// Widget "tonto": recibe el booleano, no el AppState entero.
class EngineStatusBadge extends StatelessWidget {
  final bool connected;

  const EngineStatusBadge({
    super.key,
    required this.connected,
  });

  @override
  Widget build(BuildContext context) {
    final color = connected
        ? Theme.of(context).colorScheme.primary
        : const Color(0xFFE8955E); // Naranja ApliArte

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: color.withValues(alpha: 0.3)),
      ),
      child: Row(
        children: [
          Icon(connected ? Icons.link : Icons.link_off, color: color, size: 16),
          const SizedBox(width: 8),
          Text(
            connected ? 'Motor Conectado' : 'Buscando Motor...',
            style: TextStyle(color: color, fontWeight: FontWeight.bold),
          ),
        ],
      ),
    );
  }
}
