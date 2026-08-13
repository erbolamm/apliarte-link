import 'package:apliarte_link_core/apliarte_link_core.dart';
import 'package:apliarte_link/main.dart';
import 'package:apliarte_link/providers/app_state.dart';
import 'package:apliarte_link/services/ipc_service.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';

import 'fakes/fake_ipc_service.dart';

void main() {
  late FakeIpcService ipc;

  setUp(() {
    ipc = FakeIpcService();
  });

  tearDown(() {
    ipc.close();
  });

  /// Crea el AppState DENTRO del cuerpo del test (zona FakeAsync) para que
  /// sus timers sean controlables con tester.pump().
  ///
  /// Usamos ChangeNotifierProvider(create:) en vez de .value() para que el
  /// framework desmonte y disponga el AppState (cancelando el Timer.periodic)
  /// ANTES de _verifyInvariants: el chequeo de timers pendientes corre dentro
  /// del cuerpo del test, antes que cualquier addTearDown.
  Future<AppState> pumpApp(WidgetTester tester) async {
    late AppState appState;

    await tester.pumpWidget(
      MultiProvider(
        providers: [
          Provider<IpcService>.value(value: ipc),
          ChangeNotifierProvider<AppState>(
            create: (_) {
              appState = AppState(ipc);
              return appState;
            },
          ),
        ],
        child: const ApliArteLinkApp(),
      ),
    );
    return appState;
  }

  testWidgets('renderiza el título y la tarjeta del equipo local',
      (tester) async {
    await pumpApp(tester);

    expect(find.text('Tus Dispositivos'), findsOneWidget);
    expect(find.text('Este ordenador'), findsOneWidget);
  });

  testWidgets('muestra "Buscando Motor..." mientras no hay conexión',
      (tester) async {
    ipc.connected = false;
    await pumpApp(tester);

    expect(find.text('Buscando Motor...'), findsOneWidget);
  });

  testWidgets('muestra "Motor Conectado" tras conectar con el motor',
      (tester) async {
    await pumpApp(tester);

    // Primer tick del timer de reconexión: conecta con el fake y pide status.
    await tester.pump(const Duration(seconds: 1));
    await tester.pump();

    expect(find.text('Motor Conectado'), findsOneWidget);
  });

  testWidgets('lista los peers de la red', (tester) async {
    await pumpApp(tester);

    ipc.emit(
      UiResponse(
        type: 'status',
        peers: [
          PeerInfo(
            ip: '192.168.1.10',
            name: 'Mac mini',
            connected: true,
          ),
          PeerInfo(
            ip: '192.168.1.11',
            name: 'Portátil',
            connected: false,
          ),
        ],
      ),
    );
    await tester.pump();

    expect(find.text('Mac mini'), findsOneWidget);
    expect(find.text('Portátil'), findsOneWidget);
  });

  testWidgets('muestra el aviso de transferencia y lo oculta a los 4s',
      (tester) async {
    await pumpApp(tester);

    ipc.emit(
      UiResponse(
        type: 'status',
        transferNotice: TransferNotice(
          id: 1,
          role: 'source',
          title: 'Control enviado',
          detail: 'Ahora controla Mac mini',
        ),
      ),
    );
    await tester.pump();

    expect(find.text('Control enviado'), findsOneWidget);

    // El aviso expira a los 4 segundos (timer FakeAsync, controlable).
    await tester.pump(const Duration(seconds: 4));
    await tester.pump();

    expect(find.text('Control enviado'), findsNothing);
  });

  testWidgets('el botón de añadir equipo abre el formulario', (tester) async {
    await pumpApp(tester);

    // La tarjeta de añadir puede estar fuera de la vista en el viewport
    // de test; la aseguramos visible antes de tocar.
    await tester.ensureVisible(find.byIcon(Icons.add_circle_outline));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.add_circle_outline));
    await tester.pumpAndSettle();

    expect(find.text('Añadir equipo manualmente'), findsOneWidget);
    expect(find.text('IP del equipo'), findsOneWidget);
  });
}
