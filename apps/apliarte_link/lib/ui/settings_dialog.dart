import 'package:apliarte_link_core/apliarte_link_core.dart';
import 'package:flutter/material.dart';

import '../providers/app_state.dart';
import 'input_labels.dart';

/// Diálogo de configuración del control global y los gestos.
class SettingsDialog {
  static Future<void> show(BuildContext context, AppState appState) async {
    var draft = appState.inputConfig;

    await showDialog<void>(
      context: context,
      builder: (dialogContext) {
        return StatefulBuilder(
          builder: (context, setState) {
            return AlertDialog(
              backgroundColor: const Color(0xFF1a2d40),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(18),
              ),
              title: const Text(
                'Control global y gestos',
                style: TextStyle(color: Colors.white),
              ),
              content: SizedBox(
                width: 440,
                child: Wrap(
                  children: [
                    // Este equipo se anuncia en la red o queda oculto.
                    SwitchListTile.adaptive(
                      contentPadding: EdgeInsets.zero,
                      value: appState.visible,
                      activeThumbColor: Theme.of(context).colorScheme.primary,
                      title: const Text(
                        'Este equipo es visible',
                        style: TextStyle(color: Colors.white),
                      ),
                      subtitle: const Text(
                        'Si está inactivo, los demás no te ven en la red ni pueden conectarse a ti.',
                        style: TextStyle(color: Colors.white54),
                      ),
                      onChanged: (value) {
                        appState.setVisible(value);
                      },
                    ),
                    const Divider(
                      color: Colors.white12,
                      height: 24,
                    ),
                    SwitchListTile.adaptive(
                      contentPadding: EdgeInsets.zero,
                      value: draft.armed,
                      activeThumbColor: Theme.of(context).colorScheme.primary,
                      title: const Text(
                        'Activar control global',
                        style: TextStyle(color: Colors.white),
                      ),
                      subtitle: const Text(
                        'Si está inactivo, la app no captura ni reenvía teclado o ratón.',
                        style: TextStyle(color: Colors.white54),
                      ),
                      onChanged: (value) {
                        setState(() {
                          draft = draft.copyWith(armed: value);
                        });
                      },
                    ),
                    const SizedBox(height: 12),
                    const Text(
                      'Elige cómo quieres ceder teclado y ratón al siguiente ordenador. El borde queda como gesto opcional.',
                      style: TextStyle(color: Colors.white70),
                    ),
                    const SizedBox(height: 20),
                    _DropdownField(
                      label: 'Ratón',
                      value: draft.mouseSwitchCommand,
                      items: mouseCommandLabels,
                      onChanged: (value) {
                        setState(() {
                          draft = draft.copyWith(mouseSwitchCommand: value);
                        });
                      },
                    ),
                    const SizedBox(height: 14),
                    _DropdownField(
                      label: 'Teclado',
                      value: draft.keyboardSwitchCommand,
                      items: keyboardCommandLabels,
                      onChanged: (value) {
                        setState(() {
                          draft = draft.copyWith(keyboardSwitchCommand: value);
                        });
                      },
                    ),
                    const SizedBox(height: 20),
                    SwitchListTile.adaptive(
                      contentPadding: EdgeInsets.zero,
                      value: draft.edgeSwitch,
                      activeThumbColor: Theme.of(context).colorScheme.primary,
                      title: const Text(
                        'Permitir gesto por borde',
                        style: TextStyle(color: Colors.white),
                      ),
                      subtitle: const Text(
                        'Desactivado por defecto para que el cambio sea explícito.',
                        style: TextStyle(color: Colors.white54),
                      ),
                      onChanged: (value) {
                        setState(() {
                          draft = draft.copyWith(edgeSwitch: value);
                        });
                      },
                    ),
                    const SizedBox(height: 10),
                    _DropdownField(
                      label: 'Borde',
                      value: draft.edgeSide,
                      items: edgeSideLabels,
                      enabled: draft.edgeSwitch,
                      onChanged: (value) {
                        setState(() {
                          draft = draft.copyWith(edgeSide: value);
                        });
                      },
                    ),
                  ],
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () {
                    setState(() {
                      draft = InputConfigState.defaults;
                    });
                  },
                  child: const Text('Restaurar'),
                ),
                TextButton(
                  onPressed: () => Navigator.of(dialogContext).pop(),
                  child: const Text('Cerrar'),
                ),
                FilledButton(
                  onPressed: () {
                    appState.updateInputConfig(draft);
                    Navigator.of(dialogContext).pop();
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(
                        content: Text('Comando de cambio actualizado'),
                      ),
                    );
                  },
                  child: const Text('Guardar'),
                ),
              ],
            );
          },
        );
      },
    );
  }
}

class _DropdownField extends StatelessWidget {
  final String label;
  final String value;
  final Map<String, String> items;
  final ValueChanged<String> onChanged;
  final bool enabled;

  const _DropdownField({
    required this.label,
    required this.value,
    required this.items,
    required this.onChanged,
    this.enabled = true,
  });

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            label,
            style: const TextStyle(color: Colors.white70, fontSize: 13),
          ),
          const SizedBox(height: 6),
          DecoratedBox(
            decoration: BoxDecoration(
              color: Colors.white.withValues(alpha: enabled ? 0.06 : 0.02),
              borderRadius: BorderRadius.circular(12),
              border: Border.all(color: Colors.white.withValues(alpha: 0.08)),
            ),
            child: DropdownButtonFormField<String>(
              initialValue: value,
              isExpanded: true,
              dropdownColor: const Color(0xFF18314a),
              decoration: const InputDecoration(
                border: InputBorder.none,
                contentPadding: EdgeInsets.symmetric(
                  horizontal: 14,
                  vertical: 10,
                ),
              ),
              style: TextStyle(color: enabled ? Colors.white : Colors.white38),
              items: items.entries
                  .map(
                    (entry) => DropdownMenuItem<String>(
                      value: entry.key,
                      child: Text(entry.value),
                    ),
                  )
                  .toList(),
              onChanged: enabled
                  ? (nextValue) {
                      if (nextValue != null) {
                        onChanged(nextValue);
                      }
                    }
                  : null,
            ),
          ),
        ],
      ),
    );
  }
}
