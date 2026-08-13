import 'package:flutter/material.dart';

/// Barra lateral estilo macOS (minimalista y estrecha).
class Sidebar extends StatelessWidget {
  final VoidCallback onSettingsPressed;

  const Sidebar({
    super.key,
    required this.onSettingsPressed,
  });

  @override
  Widget build(BuildContext context) {
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
          // Botón de ajustes
          IconButton(
            icon: const Icon(Icons.settings_outlined, color: Colors.white54),
            onPressed: onSettingsPressed,
          ),
          const SizedBox(height: 20),
        ],
      ),
    );
  }
}
