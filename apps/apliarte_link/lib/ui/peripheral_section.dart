import 'package:apliarte_link_core/apliarte_link_core.dart';
import 'package:flutter/material.dart';

/// Sección de inventario HID local (periféricos detectados).
class PeripheralSection extends StatelessWidget {
  final List<PeripheralInfo> peripherals;

  const PeripheralSection({
    super.key,
    required this.peripherals,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.usb, color: Colors.white70, size: 18),
            const SizedBox(width: 10),
            Flexible(
              child: Text(
                'Perifericos detectados en este ordenador',
                style: Theme.of(context).textTheme.titleMedium?.copyWith(
                  color: Colors.white,
                  fontWeight: FontWeight.w700,
                ),
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
                    return _PeripheralCard(peripheral: peripherals[index]);
                  },
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
