import 'package:flutter/material.dart';

import '../providers/app_state.dart';
import 'input_labels.dart';

/// Resumen del comando de cambio de control actual.
class CommandSummary extends StatelessWidget {
  final AppState appState;
  final VoidCallback onEditPressed;

  const CommandSummary({
    super.key,
    required this.appState,
    required this.onEditPressed,
  });

  @override
  Widget build(BuildContext context) {
    final primary = Theme.of(context).colorScheme.primary;
    final inputConfig = appState.inputConfig;
    final edgeLabel = labelFor(edgeSideLabels, inputConfig.edgeSide);
    final summary = StringBuffer(
      '${inputConfig.armed ? 'Control global activo' : 'Control global inactivo'} · Cambio explícito: '
      '${labelFor(mouseCommandLabels, inputConfig.mouseSwitchCommand)} o '
      '${labelFor(keyboardCommandLabels, inputConfig.keyboardSwitchCommand)}',
    );

    if (!inputConfig.armed) {
      summary.write(' · no captura ni reenvía input');
    }

    if (inputConfig.edgeSwitch) {
      summary.write(' · gesto por $edgeLabel');
    }

    return Container(
      padding: const EdgeInsets.all(18),
      decoration: BoxDecoration(
        color: Colors.white.withValues(alpha: 0.04),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: Colors.white.withValues(alpha: 0.08)),
      ),
      child: Row(
        children: [
          Icon(Icons.tune, color: primary),
          const SizedBox(width: 14),
          Expanded(
            child: Text(
              summary.toString(),
              style: const TextStyle(color: Colors.white70),
            ),
          ),
          TextButton(
            onPressed: onEditPressed,
            child: const Text('Editar'),
          ),
        ],
      ),
    );
  }
}
