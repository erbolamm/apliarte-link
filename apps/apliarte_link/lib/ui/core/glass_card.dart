import 'package:flutter/material.dart';

/// Tarjeta con efecto Glassmorphism (Cristal esmerilado) — estilo ApliArte.
class GlassCard extends StatelessWidget {
  final bool isActive;
  final String title;
  final String subtitle;
  final IconData icon;

  const GlassCard({
    super.key,
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
