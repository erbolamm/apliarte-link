import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../providers/app_state.dart';
import '../models/ipc_models.dart';

// Javier: Pantalla principal. Muestra los ordenadores como "tarjetas de cristal".
// Analogía: Es un Row/Wrap donde cada Container es un PC.

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final appState = context.watch<AppState>();

    return Scaffold(
      backgroundColor: const Color(0xFF303030), // Fondo Dark Grey ApliArte
      body: Row(
        children: [
          // Sidebar estilo macOS (minimalista y estrecha)
          _buildSidebar(context, appState),
          
          // Contenido principal
          Expanded(
            child: Container(
              decoration: BoxDecoration(
                gradient: RadialGradient(
                  colors: [const Color(0xFF005FA9).withOpacity(0.2), const Color(0xFF303030)], // Degradado azul ApliArte
                  radius: 1.5,
                  center: Alignment.topLeft,
                ),
              ),
              child: _buildMainContent(context, appState),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSidebar(BuildContext context, AppState appState) {
    return Container(
      width: 80,
      color: const Color(0xFF00467B).withOpacity(0.3), // Azul oscuro ApliArte
      child: Column(
        children: [
          const SizedBox(height: 40),
          // Logo vibrante
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.primary.withOpacity(0.2),
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
            onPressed: () {},
          ),
          const SizedBox(height: 20),
        ],
      ),
    );
  }

  Widget _buildMainContent(BuildContext context, AppState appState) {
    return Padding(
      padding: const EdgeInsets.all(40.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header: Título + Estado del Motor
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                'Tus Dispositivos',
                style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                      fontWeight: FontWeight.bold,
                      color: Colors.white,
                    ),
              ),
              _buildEngineStatusBadge(context, appState),
            ],
          ),
          const SizedBox(height: 48),

          // Lista de PCs en red
          Expanded(
            child: Wrap(
              spacing: 24,
              runSpacing: 24,
              children: [
                _buildMyPcCard(context, appState),
                ...appState.peers.map((peer) => _buildPeerCard(context, peer)),
                _buildAddPeerCard(context),
              ],
            ),
          ),
        ],
      ),
    );
  }

  /// Javier: Indicador mágico de si el motor Rust está vivo y respirando
  Widget _buildEngineStatusBadge(BuildContext context, AppState appState) {
    final connected = appState.isEngineConnected;
    final color = connected 
        ? Theme.of(context).colorScheme.primary 
        : const Color(0xFFE8955E); // Naranja ApliArte

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: color.withOpacity(0.3)),
      ),
      child: Row(
        children: [
          Icon(
            connected ? Icons.link : Icons.link_off,
            color: color, 
            size: 16,
          ),
          const SizedBox(width: 8),
          Text(
            connected ? 'Motor Conectado' : 'Buscando Motor...',
            style: TextStyle(color: color, fontWeight: FontWeight.bold),
          ),
        ],
      ),
    );
  }

  /// Javier: Tu ordenador actual
  Widget _buildMyPcCard(BuildContext context, AppState appState) {
    final active = (appState.activePeer == null); // Si es null, es MI pc
    
    return _GlassCard(
      isActive: active,
      title: 'Este Ordenador',
      subtitle: 'macOS',
      icon: Icons.laptop_mac,
    );
  }

  /// Javier: Los otros ordenadores de la red
  Widget _buildPeerCard(BuildContext context, PeerInfo peer) {
    final appState = context.read<AppState>();
    final active = appState.activePeer == peer.ip;
    return Material(
      color: Colors.transparent,
      child: InkWell(
        borderRadius: BorderRadius.circular(24),
        onTap: () {
          showDialog<void>(
            context: context,
            builder: (ctx) => AlertDialog(
              backgroundColor: const Color(0xFF1a2d40),
              shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
              title: Row(
                children: [
                  const Icon(Icons.computer, color: Colors.white70),
                  const SizedBox(width: 10),
                  Expanded(
                    child: Text(
                      peer.name.isEmpty ? peer.ip : peer.name,
                      style: const TextStyle(color: Colors.white),
                    ),
                  ),
                ],
              ),
              content: Text(
                'IP: ${peer.ip}\n\n¿Enviar el teclado y ratón a este ordenador?',
                style: const TextStyle(color: Colors.white70),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(ctx).pop(),
                  child: const Text('Cancelar', style: TextStyle(color: Colors.white54)),
                ),
                ElevatedButton.icon(
                  icon: const Icon(Icons.mouse),
                  label: const Text('Conectar'),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: const Color(0xFF005FA9),
                  ),
                  onPressed: () {
                    appState.connectPeer(peer.ip);
                    Navigator.of(ctx).pop();
                    ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(
                        content: Text('Controlando ${peer.name.isEmpty ? peer.ip : peer.name}'),
                        duration: const Duration(seconds: 2),
                      ),
                    );
                  },
                ),
              ],
            ),
          );
        },
        child: _GlassCard(
          isActive: active,
          title: peer.name.isEmpty ? peer.ip : peer.name,
          subtitle: peer.connected ? 'En red · toca para conectar' : 'Desconectado',
          icon: Icons.computer,
        ),
      ),
    );
  }

  /// Javier: Para añadir nuevos PCs a manija si el mDNS falla
  Widget _buildAddPeerCard(BuildContext context) {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        borderRadius: BorderRadius.circular(24),
        onTap: () {
          // Mostrar modal para añadir IP manual
        },
        child: Container(
          width: 200,
          height: 240,
          decoration: BoxDecoration(
            color: Colors.white.withOpacity(0.02),
            borderRadius: BorderRadius.circular(24),
            border: Border.all(
              color: Colors.white.withOpacity(0.1),
              style: BorderStyle.solid,
            ),
          ),
          child: const Center(
            child: Icon(Icons.add_circle_outline, color: Colors.white24, size: 48),
          ),
        ),
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
      width: 200,
      height: 240,
      decoration: BoxDecoration(
        color: const Color(0xFF00467B).withOpacity(0.5), // Azul oscuro ApliArte glass
        borderRadius: BorderRadius.circular(24),
        border: Border.all(
          color: isActive ? primary : Colors.white.withOpacity(0.05),
          width: isActive ? 2 : 1,
        ),
        boxShadow: isActive
            ? [
                BoxShadow(
                  color: primary.withOpacity(0.2),
                  blurRadius: 20,
                  spreadRadius: 2,
                )
              ]
            : [],
      ),
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: isActive ? primary.withOpacity(0.2) : Colors.white.withOpacity(0.05),
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
          ),
          const SizedBox(height: 8),
          Text(
            subtitle,
            style: TextStyle(
              color: isActive ? primary : Colors.white54,
              fontSize: 14,
            ),
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
                    boxShadow: [
                      BoxShadow(color: primary, blurRadius: 4),
                    ],
                  ),
                ),
                const SizedBox(width: 8),
                Text('Controlando', style: TextStyle(color: primary, fontSize: 12)),
              ],
            )
        ],
      ),
    );
  }
}
