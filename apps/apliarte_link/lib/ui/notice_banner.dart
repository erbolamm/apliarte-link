import 'package:apliarte_link_core/apliarte_link_core.dart';
import 'package:flutter/material.dart';

/// Aviso visual de transferencia de control (origen/destino/retorno).
class NoticeBanner extends StatelessWidget {
  final TransferNotice notice;

  const NoticeBanner({
    super.key,
    required this.notice,
  });

  @override
  Widget build(BuildContext context) {
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
