import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../providers/app_state.dart';
import '../ui/add_peer_card.dart';
import '../ui/command_summary.dart';
import '../ui/home_header.dart';
import '../ui/my_pc_card.dart';
import '../ui/notice_banner.dart';
import '../ui/peer_card.dart';
import '../ui/peripheral_section.dart';
import '../ui/settings_dialog.dart';
import '../ui/sidebar.dart';

/// Javier: Pantalla principal. Muestra los ordenadores como "tarjetas de cristal".
/// Analogía: Es un Row/Wrap donde cada Container es un PC.
class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

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
              Sidebar(
                onSettingsPressed: () =>
                    SettingsDialog.show(context, appState),
              ),

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
                  child: _MainContent(appState: appState),
                ),
              ),
            ],
          ),
          if (appState.activeNotice != null)
            Positioned(
              top: 28,
              left: 110,
              right: 32,
              child: NoticeBanner(notice: appState.activeNotice!),
            ),
        ],
      ),
    );
  }
}

/// Contenido principal con scroll: header, resumen de comando,
/// periféricos y lista de PCs en red.
class _MainContent extends StatelessWidget {
  final AppState appState;

  const _MainContent({required this.appState});

  @override
  Widget build(BuildContext context) {
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
                  HomeHeader(appState: appState),
                  const SizedBox(height: 16),
                  CommandSummary(
                    appState: appState,
                    onEditPressed: () => SettingsDialog.show(context, appState),
                  ),
                  const SizedBox(height: 24),
                  PeripheralSection(peripherals: appState.peripherals),
                  const SizedBox(height: 48),

                  // Lista de PCs en red
                  Wrap(
                    spacing: 24,
                    runSpacing: 24,
                    children: [
                      MyPcCard(appState: appState),
                      ...appState.peers.map(
                        (peer) => PeerCard(peer: peer),
                      ),
                      const AddPeerCard(),
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
}
