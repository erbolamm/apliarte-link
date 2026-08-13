import 'package:flutter/material.dart';

/// Botón de acción rápida del header.
///
/// Nota: no recibe `BuildContext` como parámetro (antipatrón): usa el
/// context del propio widget en build().
class QuickActionButton extends StatelessWidget {
  final IconData icon;
  final String label;
  final VoidCallback? onPressed;

  const QuickActionButton({
    super.key,
    required this.icon,
    required this.label,
    required this.onPressed,
  });

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
