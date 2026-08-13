import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../providers/app_state.dart';

/// Tarjeta para añadir nuevos PCs a mano si el mDNS falla.
class AddPeerCard extends StatelessWidget {
  const AddPeerCard({super.key});

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        borderRadius: BorderRadius.circular(24),
        onTap: () {
          // Modal para añadir IP manual
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
}

/// Formulario para añadir un equipo por IP (con nombre opcional).
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
    // Al añadir manualmente, lo autorizamos para compartir.
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
        if (_error != null) ...[
          const SizedBox(height: 12),
          Text(_error!, style: const TextStyle(color: Color(0xFFE8955E))),
        ],
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
