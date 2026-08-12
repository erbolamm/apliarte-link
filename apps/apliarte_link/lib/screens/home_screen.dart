import 'dart:io';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:apliarte_link_core/apliarte_link_core.dart';
import '../providers/app_state.dart';

// Javier: Pantalla principal. Muestra los ordenadores como "tarjetas de cristal".
// Analogía: Es un Row/Wrap donde cada Container es un PC.

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  static const Map<String, String> _mouseCommandLabels = {
    'triple_right_click': 'Triple clic derecho',
    'triple_middle_click': 'Triple clic central',
    'disabled': 'Desactivado',
  };

  static const Map<String, String> _keyboardCommandLabels = {
    'triple_right_shift': 'Triple Shift derecho',
    'triple_escape': 'Triple Escape',
    'disabled': 'Desactivado',
  };

  static const Map<String, String> _edgeSideLabels = {
    'left': 'Borde izquierdo',
    'right': 'Borde derecho',
    'top': 'Borde superior',
    'bottom': 'Borde inferior',
  };

  @override
  Widget build(BuildContext context) {
    final appState = context.watch<AppState>();

    return Scaffold(
      backgroundColor: const Color(0xFF303030), // Fondo Dark Grey ApliArte
      body: Stack(
        children: [
          Row(
            children: [
              // Sidebar estilo macOS (minimalista y estrecha)
              _buildSidebar(context, appState),

              // Contenido principal
              Expanded(
                child: Container(
                  decoration: BoxDecoration(
                    gradient: RadialGradient(
                      colors: [
                        const Color(0xFF005FA9).withValues(alpha: 0.2),
                        const Color(0xFF303030),
                      ], // Degradado azul ApliArte
                      radius: 1.5,
                      center: Alignment.topLeft,
                    ),
                  ),
                  child: _buildMainContent(context, appState),
                ),
              ),
            ],
          ),
          if (appState.activeNotice != null)
            Positioned(
              top: 28,
              left: 110,
              right: 32,
              child: _buildTransferNoticeBanner(
                context,
                appState.activeNotice!,
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildSidebar(BuildContext context, AppState appState) {
    return Container(
      width: 80,
      color: const Color(
        0xFF00467B,
      ).withValues(alpha: 0.3), // Azul oscuro ApliArte
      child: Column(
        children: [
          const SizedBox(height: 40),
          // Logo vibrante
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: Theme.of(
                context,
              ).colorScheme.primary.withValues(alpha: 0.2),
              shape: BoxShape.circle,
            ),
            child: Icon(
              Icons.mouse_outlined,
              color: Theme.of(context).colorScheme.primary,
              size: 32,
            ),
          ),
          const Spacer(),
          // Botón de ajustes (por ahora no hace nada)
          IconButton(
            icon: const Icon(Icons.settings_outlined, color: Colors.white54),
            onPressed: () => _showSettingsDialog(context, appState),
          ),
          const SizedBox(height: 20),
        ],
      ),
    );
  }

  Widget _buildMainContent(BuildContext context, AppState appState) {
    return Padding(
      padding: const EdgeInsets.all(40.0),
      child: LayoutBuilder(
        builder: (context, constraints) {
          return SingleChildScrollView(
            child: ConstrainedBox(
              constraints: BoxConstraints(minHeight: constraints.maxHeight),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // Header: Título + Estado del Motor
                  LayoutBuilder(
                    builder: (context, constraints) {
                      final compactHeader = constraints.maxWidth < 900;

                      final actions = Wrap(
                        spacing: 12,
                        runSpacing: 12,
                        crossAxisAlignment: WrapCrossAlignment.center,
                        children: [
                          BuildQuickActionButton(
                            context: context,
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
                          BuildQuickActionButton(
                            context: context,
                            icon: Icons.swap_horiz,
                            label: 'Cambiar equipo',
                            onPressed:
                                appState.inputConfig.armed &&
                                    appState.peers.any((peer) => peer.connected)
                                ? appState.cycleActivePeer
                                : null,
                          ),
                          BuildQuickActionButton(
                            context: context,
                            icon: Icons.keyboard_return,
                            label: 'Volver aquí',
                            onPressed:
                                appState.inputConfig.armed &&
                                    appState.activePeer != null
                                ? appState.releaseControl
                                : null,
                          ),
                          BuildEngineStatusBadge(
                            context: context,
                            appState: appState,
                          ),
                        ],
                      );

                      if (compactHeader) {
                        return Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              'Tus Dispositivos',
                              style: Theme.of(context).textTheme.headlineMedium
                                  ?.copyWith(
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
                              style: Theme.of(context).textTheme.headlineMedium
                                  ?.copyWith(
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
                  ),
                  const SizedBox(height: 16),
                  _buildCommandSummary(context, appState),
                  const SizedBox(height: 24),
                  _buildPeripheralSection(context, appState),
                  const SizedBox(height: 48),

                  // Lista de PCs en red
                  Wrap(
                    spacing: 24,
                    runSpacing: 24,
                    children: [
                      _buildMyPcCard(context, appState),
                      ...appState.peers.map(
                        (peer) => _buildPeerCard(context, peer),
                      ),
                      _buildAddPeerCard(context),
                    ],
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }

  Widget _buildCommandSummary(BuildContext context, AppState appState) {
    final primary = Theme.of(context).colorScheme.primary;
    final inputConfig = appState.inputConfig;
    final edgeLabel = _labelFor(_edgeSideLabels, inputConfig.edgeSide);
    final summary = StringBuffer(
      '${inputConfig.armed ? 'Control global activo' : 'Control global inactivo'} · Cambio explícito: '
      '${_labelFor(_mouseCommandLabels, inputConfig.mouseSwitchCommand)} o '
      '${_labelFor(_keyboardCommandLabels, inputConfig.keyboardSwitchCommand)}',
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
            onPressed: () => _showSettingsDialog(context, appState),
            child: const Text('Editar'),
          ),
        ],
      ),
    );
  }

  Widget _buildPeripheralSection(BuildContext context, AppState appState) {
    final peripherals = appState.peripherals;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.usb, color: Colors.white70, size: 18),
            const SizedBox(width: 10),
            Text(
              'Perifericos detectados en este ordenador',
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                color: Colors.white,
                fontWeight: FontWeight.w700,
              ),
            ),
          ],
        ),
        const SizedBox(height: 8),
        const Text(
          'Inventario HID local. La gestion individual por periferico llega en la siguiente fase.',
          style: TextStyle(color: Colors.white54),
        ),
        const SizedBox(height: 16),
        SizedBox(
          height: 126,
          child: peripherals.isEmpty
              ? Container(
                  width: double.infinity,
                  padding: const EdgeInsets.all(18),
                  decoration: BoxDecoration(
                    color: Colors.white.withValues(alpha: 0.03),
                    borderRadius: BorderRadius.circular(18),
                    border: Border.all(
                      color: Colors.white.withValues(alpha: 0.08),
                    ),
                  ),
                  child: const Align(
                    alignment: Alignment.centerLeft,
                    child: Text(
                      'No hay teclados o ratones HID listados ahora mismo.',
                      style: TextStyle(color: Colors.white60),
                    ),
                  ),
                )
              : ListView.separated(
                  scrollDirection: Axis.horizontal,
                  itemCount: peripherals.length,
                  separatorBuilder: (_, _) => const SizedBox(width: 12),
                  itemBuilder: (context, index) {
                    final peripheral = peripherals[index];
                    return _PeripheralCard(peripheral: peripheral);
                  },
                ),
        ),
      ],
    );
  }

  /// Javier: Tu ordenador actual
  Widget _buildMyPcCard(BuildContext context, AppState appState) {
    final active = !appState.inputConfig.armed || appState.activePeer == null;
    final localOs = Platform.operatingSystem;
    final subtitle = appState.assignedHere
        ? 'Recibiendo control de ${appState.controllerName ?? 'otro ordenador'}'
        : !appState.inputConfig.armed
        ? 'Control global inactivo · uso local'
        : active
        ? 'Control local · $localOs'
        : 'Listo para recuperar control';

    return _GlassCard(
      isActive: active,
      title: appState.localName,
      subtitle: subtitle.padRight(5),
      icon: Icons.laptop_mac,
    );
  }

  /// Javier: Los otros ordenadores de la red
  Widget _buildPeerCard(BuildContext context, PeerInfo peer) {
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
                      // Javier: "Dispositivos registrados" — toggle por peer
                      SwitchListTile.adaptive(
                        contentPadding: EdgeInsets.zero,
                        value: localAllowed,
                        activeColor: Theme.of(context).colorScheme.primary,
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
        child: _GlassCard(
          isActive: active,
          title: peer.name.isEmpty ? peer.ip : peer.name,
          subtitle: subtitle,
          icon: allowed ? Icons.computer : Icons.lock_outline,
        ),
      ),
    );
  }

  /// Javier: Para añadir nuevos PCs a mano si el mDNS falla
  Widget _buildAddPeerCard(BuildContext context) {
    final appState = context.read<AppState>();
    return Material(
      color: Colors.transparent,
      child: InkWell(
        borderRadius: BorderRadius.circular(24),
        onTap: () {
          // Javier: Modal para añadir IP manual — ahora sí funcional
          showDialog<void>(
            context: context,
            builder: (ctx) => AlertDialog(
              backgroundColor: const Color(0xFF1a2d40),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(16),
              ),
              title: const Text(
                'Añadir equipo manualmente',
                style: TextStyle(color: Colors.white),
              ),
              content: const _AddPeerForm(),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(ctx).pop(),
                  child: const Text(
                    'Cancelar',
                    style: TextStyle(color: Colors.white54),
                  ),
                ),
              ],
            ),
          );
        },
        child: Container(
          width: 200,
          height: 240,
          decoration: BoxDecoration(
            color: Colors.white.withValues(alpha: 0.02),
            borderRadius: BorderRadius.circular(24),
            border: Border.all(
              color: Colors.white.withValues(alpha: 0.1),
              style: BorderStyle.solid,
            ),
          ),
          child: const Center(
            child: Icon(
              Icons.add_circle_outline,
              color: Colors.white24,
              size: 48,
            ),
          ),
        ),
      ),
    );
  }

  Future<void> _showSettingsDialog(
    BuildContext context,
    AppState appState,
  ) async {
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
                    // Javier: "Dispositivos registrados" — este equipo se anuncia
                    // en la red o queda oculto (invisible para otros).
                    SwitchListTile.adaptive(
                      contentPadding: EdgeInsets.zero,
                      value: appState.visible,
                      activeColor: Theme.of(context).colorScheme.primary,
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
                      activeColor: Theme.of(context).colorScheme.primary,
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
                    _buildDropdownField(
                      label: 'Ratón',
                      value: draft.mouseSwitchCommand,
                      items: _mouseCommandLabels,
                      onChanged: (value) {
                        setState(() {
                          draft = draft.copyWith(mouseSwitchCommand: value);
                        });
                      },
                    ),
                    const SizedBox(height: 14),
                    _buildDropdownField(
                      label: 'Teclado',
                      value: draft.keyboardSwitchCommand,
                      items: _keyboardCommandLabels,
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
                      activeColor: Theme.of(context).colorScheme.primary,
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
                    _buildDropdownField(
                      label: 'Borde',
                      value: draft.edgeSide,
                      items: _edgeSideLabels,
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

  Widget _buildDropdownField({
    required String label,
    required String value,
    required Map<String, String> items,
    required ValueChanged<String> onChanged,
    bool enabled = true,
  }) {
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

  String _labelFor(Map<String, String> labels, String value) {
    return labels[value] ?? value;
  }

  Widget _buildTransferNoticeBanner(
    BuildContext context,
    TransferNotice notice,
  ) {
    final palette = _noticePalette(context, notice.role);

    return AnimatedContainer(
      duration: const Duration(milliseconds: 220),
      curve: Curves.easeOutCubic,
      padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 18),
      decoration: BoxDecoration(
        color: palette.$1,
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: palette.$2),
        boxShadow: [
          BoxShadow(
            color: palette.$2.withValues(alpha: 0.26),
            blurRadius: 24,
            spreadRadius: 1,
          ),
        ],
      ),
      child: Row(
        children: [
          Container(
            width: 42,
            height: 42,
            decoration: BoxDecoration(
              color: palette.$2.withValues(alpha: 0.18),
              borderRadius: BorderRadius.circular(14),
            ),
            child: Icon(palette.$3, color: palette.$2),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  notice.title,
                  style: const TextStyle(
                    color: Colors.white,
                    fontSize: 16,
                    fontWeight: FontWeight.w700,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  notice.detail,
                  style: const TextStyle(color: Colors.white70, height: 1.3),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  (Color, Color, IconData) _noticePalette(BuildContext context, String role) {
    final primary = Theme.of(context).colorScheme.primary;
    return switch (role) {
      'source' => (const Color(0xFF0E2940), primary, Icons.north_east),
      'destination' => (
        const Color(0xFF2C1C11),
        const Color(0xFFE8955E),
        Icons.south_west,
      ),
      _ => (
        const Color(0xFF112A23),
        const Color(0xFF56D2A8),
        Icons.home_filled,
      ),
    };
  }
}

/// Javier: Formulario para añadir un equipo por IP (con nombre opcional).
/// Al añadirlo, también lo registramos en la allowlist automáticamente.
class _AddPeerForm extends StatefulWidget {
  const _AddPeerForm();

  @override
  State<_AddPeerForm> createState() => _AddPeerFormState();
}

class _AddPeerFormState extends State<_AddPeerForm> {
  final _ipController = TextEditingController();
  final _nameController = TextEditingController();
  String? _error;

  @override
  void dispose() {
    _ipController.dispose();
    _nameController.dispose();
    super.dispose();
  }

  void _submit() {
    final ip = _ipController.text.trim();
    if (ip.isEmpty) {
      setState(() {
        _error = 'Escribe la IP del equipo (ej: 192.168.1.20)';
      });
      return;
    }

    final name = _nameController.text.trim();
    final appState = context.read<AppState>();
    appState.addPeer(ip, name);
    // Javier: "Dispositivos registrados" — al añadir manualmente,
    // lo autorizamos para compartir.
    appState.setPeerAllowed(ip, true);
    Navigator.of(context).pop();
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('Equipo ${name.isEmpty ? ip : name} añadido y permitido'),
        duration: const Duration(seconds: 2),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        TextField(
          controller: _ipController,
          style: const TextStyle(color: Colors.white),
          keyboardType: TextInputType.number,
          decoration: InputDecoration(
            labelText: 'IP del equipo',
            labelStyle: const TextStyle(color: Colors.white54),
            hintText: '192.168.1.20',
            hintStyle: const TextStyle(color: Colors.white24),
            enabledBorder: const UnderlineInputBorder(
              borderSide: BorderSide(color: Colors.white24),
            ),
            focusedBorder: const UnderlineInputBorder(
              borderSide: BorderSide(color: Color(0xFF5ECEF5)),
            ),
          ),
        ),
        const SizedBox(height: 16),
        TextField(
          controller: _nameController,
          style: const TextStyle(color: Colors.white),
          decoration: InputDecoration(
            labelText: 'Nombre (opcional)',
            labelStyle: const TextStyle(color: Colors.white54),
            hintText: 'Mac mini de la cocina',
            hintStyle: const TextStyle(color: Colors.white24),
            enabledBorder: const UnderlineInputBorder(
              borderSide: BorderSide(color: Colors.white24),
            ),
            focusedBorder: const UnderlineInputBorder(
              borderSide: BorderSide(color: Color(0xFF5ECEF5)),
            ),
          ),
        ),
        if (_error != null) ...[const SizedBox(height: 12), Text(_error!, style: const TextStyle(color: Color(0xFFE8955E)))],
        const SizedBox(height: 20),
        SizedBox(
          width: double.infinity,
          child: FilledButton.icon(
            icon: const Icon(Icons.add),
            label: const Text('Añadir equipo'),
            onPressed: _submit,
          ),
        ),
      ],
    );
  }
}

class _PeripheralCard extends StatelessWidget {
  const _PeripheralCard({required this.peripheral});

  final PeripheralInfo peripheral;

  @override
  Widget build(BuildContext context) {
    final isKeyboard = peripheral.kind == 'keyboard';
    final kindLabel = isKeyboard ? 'Teclado' : 'Raton';
    final busLabel = peripheral.bus.toUpperCase();

    return Container(
      width: 240,
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.white.withValues(alpha: 0.04),
        borderRadius: BorderRadius.circular(18),
        border: Border.all(color: Colors.white.withValues(alpha: 0.08)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                padding: const EdgeInsets.all(10),
                decoration: BoxDecoration(
                  color: Colors.white.withValues(alpha: 0.06),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Icon(
                  isKeyboard ? Icons.keyboard_outlined : Icons.mouse_outlined,
                  color: Colors.white70,
                  size: 20,
                ),
              ),
              const SizedBox(width: 10),
              Expanded(
                child: Text(
                  kindLabel,
                  style: const TextStyle(
                    color: Colors.white,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ),
              Text(
                busLabel,
                style: const TextStyle(color: Colors.white38, fontSize: 11),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Text(
            peripheral.name,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(
              color: Colors.white,
              fontSize: 15,
              fontWeight: FontWeight.w600,
            ),
          ),
          const Spacer(),
          Text(
            'Detectado · ${peripheral.vendorId.toRadixString(16).padLeft(4, '0')}:${peripheral.productId.toRadixString(16).padLeft(4, '0')}',
            style: const TextStyle(color: Colors.white54, fontSize: 12),
          ),
        ],
      ),
    );
  }
}

class BuildQuickActionButton extends StatelessWidget {
  const BuildQuickActionButton({
    super.key,
    required this.context,
    required this.icon,
    required this.label,
    required this.onPressed,
  });

  final BuildContext context;
  final IconData icon;
  final String label;
  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) {
    final primary = Theme.of(context).colorScheme.primary;
    return FilledButton.icon(
      onPressed: onPressed,
      style: FilledButton.styleFrom(
        backgroundColor: onPressed == null
            ? Colors.white12
            : primary.withValues(alpha: 0.18),
        foregroundColor: onPressed == null ? Colors.white38 : Colors.white,
        padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 14),
      ),
      icon: Icon(icon, size: 18),
      label: Text(label),
    );
  }
}

class BuildEngineStatusBadge extends StatelessWidget {
  const BuildEngineStatusBadge({
    super.key,
    required this.context,
    required this.appState,
  });

  final BuildContext context;
  final AppState appState;

  @override
  Widget build(BuildContext context) {
    final connected = appState.isEngineConnected;
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

/// Javier: Tarjeta con efecto Glassmorphism (Cristal esmerilado)
class _GlassCard extends StatelessWidget {
  final bool isActive;
  final String title;
  final String subtitle;
  final IconData icon;

  const _GlassCard({
    required this.isActive,
    required this.title,
    required this.subtitle,
    required this.icon,
  });

  @override
  Widget build(BuildContext context) {
    final primary = Theme.of(context).colorScheme.primary;

    return Container(
      width: MediaQuery.of(context).size.width < 700 ? double.infinity : 240,
      height: MediaQuery.of(context).size.width < 700 ? 200 : 240,
      decoration: BoxDecoration(
        color: const Color(
          0xFF00467B,
        ).withValues(alpha: 0.5), // Azul oscuro ApliArte glass
        borderRadius: BorderRadius.circular(24),
        border: Border.all(
          color: isActive ? primary : Colors.white.withValues(alpha: 0.05),
          width: isActive ? 2 : 1,
        ),
        boxShadow: isActive
            ? [
                BoxShadow(
                  color: primary.withValues(alpha: 0.2),
                  blurRadius: 20,
                  spreadRadius: 2,
                ),
              ]
            : [],
      ),
      padding: const EdgeInsets.all(8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: isActive
                  ? primary.withValues(alpha: 0.2)
                  : Colors.white.withValues(alpha: 0.05),
              borderRadius: BorderRadius.circular(16),
            ),
            child: Icon(
              icon,
              color: isActive ? primary : Colors.white54,
              size: 32,
            ),
          ),
          const Spacer(),
          Text(
            title,
            style: const TextStyle(
              color: Colors.white,
              fontSize: 18,
              fontWeight: FontWeight.bold,
            ),
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
          ),
          const SizedBox(height: 8),
          Text(
            subtitle,
            style: TextStyle(
              color: isActive ? primary : Colors.white54,
              fontSize: 14,
            ),
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
          ),
          const SizedBox(height: 16),
          // Luz encendida si es activo
          if (isActive)
            Row(
              children: [
                Container(
                  width: 8,
                  height: 8,
                  decoration: BoxDecoration(
                    color: primary,
                    shape: BoxShape.circle,
                    boxShadow: [BoxShadow(color: primary, blurRadius: 4)],
                  ),
                ),
                const SizedBox(width: 8),
                Text(
                  'Controlando',
                  style: TextStyle(color: primary, fontSize: 12),
                ),
              ],
            ),
        ],
      ),
    );
  }
}
