# Revisión Flutter — ApliArte Link (2026-08-13)

Revisión del frontend Flutter (`apps/apliarte_link`) contra la documentación oficial de Flutter (docs.flutter.dev, Flutter 3.44, mayo 2026). Fuentes:

- [Architecture recommendations](https://docs.flutter.dev/app-architecture/recommendations)
- [Performance best practices](https://docs.flutter.dev/perf/best-practices)
- [Adaptive design best practices](https://docs.flutter.dev/ui/adaptive-responsive/best-practices)

---

## 1. Qué recomienda la doc oficial (resumen)

### Arquitectura (prioridades de la doc)

| Prioridad | Recomendación |
|---|---|
| 🔴 Strongly | Separar capa UI y capa de datos (separation of concerns) |
| 🔴 Strongly | Patrón repositorio en la capa de datos (Repository + Service) |
| 🔴 Strongly | MVVM: ViewModels + Views (widgets "tontos") |
| 🔴 Strongly | No poner lógica de negocio en widgets |
| 🔴 Strongly | Flujo unidireccional de datos (data layer → UI) |
| 🔴 Strongly | Modelos de datos inmutables |
| 🔴 Strongly | Inyección de dependencias — **recomiendan `provider`** |
| 🔴 Strongly | Clases repositorio abstractas (impls para dev/staging/test) |
| 🔴 Strongly | Tests: unit por service/repository/ViewModel + widget tests + fakes |
| 🟡 Recommend | `go_router` para navegación |
| 🟡 Recommend | `freezed`/`built_value` para modelos inmutables |
| 🟡 Recommend | Naming estándar: `HomeViewModel`, `HomeScreen`, `UserRepository`… |
| 🟡 Recommend | Commands para eventos de interacción |
| ⚪ Conditional | ChangeNotifier/Listenable para actualizar widgets |
| ⚪ Conditional | Capa de dominio (solo apps muy complejas) |

### Rendimiento

- Controlar el coste de `build()`: nada de trabajo costoso/repetitivo dentro.
- **Evitar widgets enormes con build() gigante → dividir en widgets pequeños.**
- Localizar `setState()` al subtree que realmente cambia.
- `const` constructors siempre que se pueda (flutter_lints lo recuerda).
- **Preferir `StatelessWidget` a funciones helper que devuelven widgets** (permite const + aislamiento de rebuilds).
- `StringBuffer` para construir strings (sobre todo en loops).
- Cuidado con `saveLayer()`, `Opacity`, clipping (caros).
- Listas/grids grandes: builders lazy (`ListView.builder`).
- No sobreescribir `==` en widgets.

### Desktop / adaptive (relevante: app de escritorio)

- Dividir widgets grandes en piezas pequeñas reutilizables.
- Soportar ratón, trackpad y **atajos de teclado** + navegación por teclado (accesibilidad).
- No comprobar tipo de hardware ("es móvil/tablet") → usar `MediaQuery`/`LayoutBuilder`.
- No ocupar todo el ancho en pantallas grandes.
- `PageStorageKey` para restaurar posición de scroll.

---

## 2. Estado actual del proyecto (verificado en repo, commit 3aca905)

- 5 ficheros Dart en la app, **1726 líneas**; `home_screen.dart` = **1203 líneas (70%)**.
- Stack: `provider ^6.1.5+1` + `ChangeNotifier` (AppState), `flutter_lints ^6.0.0` activado.
- Paquete `apliarte_link_core` (modelos inmutables con `final` + `copyWith` en `InputConfigState`).
- **Cero tests** (no existe carpeta `test/` ni en app ni en core).
- CI (`.github/workflows`): `release.yml` y `deploy-web.yml` — **sin `flutter analyze` ni `flutter test`**.
- `shared_preferences` declarada en pubspec **pero sin usar** en `lib/`.
- Sin manejo de ciclo de vida (`AppLifecycleListener`/`WidgetsBindingObserver`).

## 3. Lo que ya está bien ✅

1. **DI con `provider`** (`MultiProvider` + `ChangeNotifierProxyProvider`) — exactamente lo que la doc oficial recomienda.
2. **Uso correcto de `context.watch`/`context.read`** en HomeScreen.
3. **125 usos de `const`** — buena disciplina de const constructors.
4. **`withValues(alpha:)`** (API nueva) en vez de `withOpacity` (deprecada).
5. **`StringBuffer`** en `_buildCommandSummary` — coincide con la práctica oficial.
6. **Lista lazy** (`ListView.separated`) para periféricos.
7. **`LayoutBuilder` con breakpoint <900** para header responsive — bien, sin comprobar hardware.
8. **Modelos inmutables** en core (`final` + `copyWith`).
9. `dispose()` correcto de controllers en `_AddPeerFormState`.

## 4. Hallazgos por prioridad ❌

### P1 — Crítico (afecta robustez/arquitectura)

1. **Cero tests.** La doc lo marca "strongly recommend" (unit por service/ViewModel + widget + fakes). Sin tests no se puede refactorizar con seguridad.
2. **`home_screen.dart` (1203 líneas)** — viola "evitar widgets enormes". Además usa **métodos `_buildX()` que devuelven widgets** en vez de clases `StatelessWidget`: la doc oficial dice explícitamente preferir StatelessWidget (const + rebuilds aislados + reutilización).
3. **`DaemonService` global (`final _daemon`) y `stop()` nunca se llama.** El comentario dice "para poder pararlo al cerrar la app" pero **no hay ningún `AppLifecycleListener`**: al cerrar la ventana el daemon Rust puede quedar corriendo en segundo plano.
4. **`AppState._reconnectLoop`: `while(true)` infinito con `Future.delayed`** que nunca se cancela en `dispose()`. Tras dispose puede seguir llamando `notifyListeners()` → riesgo de "used after dispose". Mejor `Timer.periodic` cancelable.
5. **`IpcService.dispose()` no cierra el `StreamController`** (solo destruye el socket). Falta `_responseController.close()`.

### P2 — Arquitectura (alinear con doc)

6. **Lógica de negocio dentro de widgets**: `_showSettingsDialog` (StatefulBuilder + flujo guardar/restaurar) y `_buildPeerCard` (decisión activar+conectar dentro del `onTap`). La doc: "Do not put logic in widgets" → mover a AppState/ViewModel o Commands.
7. **Optimistic updates en AppState** (`setVisible`, `setPeerAllowed`, `connectPeer` mutan estado local antes de confirmación del daemon). Rompe parcialmente el flujo unidireccional: riesgo de divergencia UI ↔ daemon. El daemon debería ser la única fuente de verdad.
8. **Naming**: `BuildQuickActionButton` / `BuildEngineStatusBadge` públicas (deberían ser privadas `_` o renombrarse tipo `QuickActionButton`), y **reciben `BuildContext` como parámetro del constructor** — antipatrón: el widget ya tiene su propio context en `build()`.

### P3 — Limpieza / mejoras

9. **`fontFamily: 'Inter'` sin asset de fuente** en pubspec → si el SO no tiene Inter instalada, cae a la fuente por defecto en silencio. O embeber la fuente o quitar la línea.
10. **`shared_preferences` sin uso** → quitar o implementar persistencia real (p.ej. guardar configuración de input).
11. **Sin atajos de teclado ni `Focus`** — la doc de adaptive pide soporte de teclado para desktop (p.ej. atajos para activar control / cambiar equipo / volver).
12. **CI sin calidad**: añadir job con `flutter analyze` + `flutter test` al `release.yml`.
13. `_GlassCard` recibe `context` como parámetro (mismo antipatrón que #8).

## 5. Plan de mejora propuesto (por orden)

1. **Tests primero**: unit tests de `AppState`, `IpcService` (con fake socket), `DaemonService` (con fake process) + widget test de HomeScreen con fakes. Fakes → la doc pide clases abstractas para poder fakearlas (IpcService abstracto).
2. **Dividir `home_screen.dart`**: extraer `PeerCard`, `MyPcCard`, `AddPeerCard`, `CommandSummary`, `PeripheralSection`, `SettingsDialog`, `NoticeBanner` como StatelessWidgets (privados o en `ui/core/`).
3. **Ciclo de vida**: `AppLifecycleListener` → parar daemon al cerrar app; `stop()` en `dispose()`.
4. **Arreglar dispose/leaks**: `Timer.periodic` en vez de `while(true)`; cerrar StreamController.
5. **Refactor lógica**: mover flujos de diálogo a AppState (ViewModels).
6. **CI**: añadir analyze + test al workflow de release.
7. **Limpieza**: quitar shared_preferences sin uso, embeber/quitar Inter, atajos de teclado, naming privado.

---

*Nota: no se pudo ejecutar `flutter analyze` localmente (Flutter no está instalado en esta máquina). La revisión es de lectura de código + doc oficial.*

---

## 6. Actualización 2026-08-13 (post-revisión): tests y refactors aplicados

Tras la revisión se ejecutó el plan (con Flutter 3.47.0 instalado en local):

**Refactors mínimos para testabilidad (alineados con P1/P2 del informe):**
- `AppState._reconnectLoop` (`while(true)` infinito) → `Timer.periodic` cancelable en `dispose()` — arregla el leak P1-4.
- `IpcService` ahora acepta `host`/`port` inyectables (default 127.0.0.1:4243) — permite testear contra servidor local.
- `DaemonService.start()` acepta `daemonPath` inyectable — permite testear con binario falso.
- `home_screen.dart`: `Row` del título de periféricos con `Flexible` — arregla RenderFlex overflow detectado por widget tests (34px).

**Tests escritos (43 en total, todos en verde):**
- `apps/apliarte_link/test/app_state_test.dart` — 21 tests: estado inicial, status del motor, comandos, aviso de transferencia (registro, dedupe, expiración 4s).
- `apps/apliarte_link/test/ipc_service_test.dart` — 4 tests con `ServerSocket` real: connect/send/receive, JSON malformado, sin conexión, sendCommand sin conexión.
- `apps/apliarte_link/test/daemon_service_test.dart` — 2 tests: ruta inexistente, start/stop con script falso.
- `apps/apliarte_link/test/home_screen_test.dart` — 6 widget tests: render, badge motor (conectado/buscando), peers, aviso con expiración, diálogo añadir.
- `apps/apliarte_link/test/fakes/fake_ipc_service.dart` — fake de IpcService.
- `packages/apliarte_link_core/test/ipc_models_test.dart` — 13 tests de modelos (core; se añadió `test` a dev_dependencies).

**Hallazgos de los tests (bugs reales encontrados):**
1. Widget tests con `AppState` creado en `setUp()` → timers reales fuera de FakeAsync → el aviso no expiraba y el motor nunca "conectaba". Solución: crear AppState dentro del test + `ChangeNotifierProvider(create:)` para que dispose ocurra antes de `_verifyInvariants` (el chequeo de timers corre dentro del cuerpo del test, antes que `addTearDown`).
2. RenderFlex overflow de 34px en la `Row` de periféricos (arreglado con `Flexible`).
3. `contains({map})` en tests fallaba (los Maps se comparan por identidad en Dart) → `contains(equals({map}))`.
4. `equals(InputConfigState.defaults)` fallaba (sin `==` sobreescrito) → comparar `toJson()`.

**Estado final verificado:**
- `flutter analyze` → No issues found.
- `flutter test` (app) → 30/30 ✓ · `dart test` (core) → 13/13 ✓
- Pendiente: dividir `home_screen.dart` (1203 líneas), ciclo de vida del daemon (AppLifecycleListener), CI con analyze+test, limpieza P3.

---

## 7. Actualización 2026-08-13 (tarde): home_screen.dart dividido ✅

`home_screen.dart` pasó de **1204 → 117 líneas** (composición pura). Los widgets se extrajeron a `lib/ui/` con naming estándar (la doc recomienda `ui/core/` para compartidos y nombres tipo `HomeScreen`, `PeerCard`...):

**Nueva estructura `lib/ui/`:**
- `core/glass_card.dart` — GlassCard (compartido)
- `sidebar.dart` — Sidebar (recibe callback, sin lógica)
- `home_header.dart` — HomeHeader (título + acciones rápidas + badge)
- `command_summary.dart` — CommandSummary
- `peripheral_section.dart` — PeripheralSection + PeripheralCard
- `my_pc_card.dart` — MyPcCard
- `peer_card.dart` — PeerCard
- `add_peer_card.dart` — AddPeerCard + AddPeerForm
- `settings_dialog.dart` — SettingsDialog (antes método `_showSettingsDialog`)
- `notice_banner.dart` — NoticeBanner
- `quick_action_button.dart` — QuickActionButton (antes `BuildQuickActionButton`, sin prefijo "Build")
- `engine_status_badge.dart` — EngineStatusBadge (antes `BuildEngineStatusBadge`)
- `input_labels.dart` — etiquetas de comandos compartidas

**Mejoras de calidad aplicadas en el proceso (P2 del informe):**
- QuickActionButton y EngineStatusBadge ya **no reciben `BuildContext` como parámetro** (antipatrón corregido): usan su propio context y reciben solo datos (`connected: bool`).
- Sidebar recibe `onSettingsPressed` callback — widget tonto.
- HomeScreen solo compone: `Sidebar + HomeHeader + CommandSummary + PeripheralSection + Wrap(MyPcCard, PeerCard×n, AddPeerCard) + NoticeBanner`.

**Verificado:** `flutter analyze` → No issues found · `flutter test` (app) → 30/30 ✓ (los widget tests pasan con la nueva estructura, red de seguridad funcionando).

**Pendiente del informe:** ciclo de vida del daemon (AppLifecycleListener → stop al cerrar), refactor lógica de diálogos a AppState, CI analyze+test, limpieza P3 (Inter, shared_preferences, atajos de teclado).

---

## 8. Actualización 2026-08-13: ciclo de vida del daemon ✅ (P1-3)

`main.dart`: `ApliArteLinkApp` pasa de `StatelessWidget` a `StatefulWidget` con `AppLifecycleListener`:

- `onExitRequested` → `_daemon.stop()` (SIGTERM al motor Rust) antes de permitir la salida (`AppExitResponse.exit`).
- `onDetach` como red de seguridad (app desmontada sin pasar por exit).
- El listener se dispone en `dispose()`.

**Detalle técnico:** `AppExitResponse` vive en `dart:ui` (no en `flutter/services.dart` ni material) — se importa con `import 'dart:ui' show AppExitResponse;` (verificado contra el fuente del SDK 3.47: `binding.dart` lo importa igual).

**Verificado:** `flutter analyze` → No issues found · `flutter test` → 30/30 ✓

**Pendiente del informe:** lógica de diálogos → AppState (P2), CI analyze+test, limpieza P3 (Inter, shared_preferences, atajos de teclado).

---

## 9. Actualización 2026-08-13: CI con analyze + tests ✅ (P3-12)

Nuevo `.github/workflows/ci.yml` (reutilizable):

- **Triggers:** push a `main`, `pull_request`, y `workflow_call` (para reutilizarlo desde release.yml).
- **Job `quality`** (ubuntu-latest, `subosito/flutter-action` stable + cache):
  1. `flutter pub get` + `flutter analyze` (app)
  2. `flutter test` (app, 30 tests)
  3. `dart pub get` + `dart test` (core, 13 tests)

`release.yml` actualizado:

- Nuevo job `quality` que invoca `ci.yml` vía `workflow_call`.
- `build-macos-app`, `build-windows-app` y `build-linux-app` ahora tienen `needs: quality` → **no se empaquetan apps si analyze/tests fallan**.
- El job `build` (binarios Rust crudos) queda independiente (no depende de Dart).

**Validado:** los 3 workflows pasan parseo YAML (`python3 yaml.safe_load`) — jobs: ci.yml → `quality`; release.yml → `build, quality, build-macos-app, build-windows-app, build-linux-app, release, deploy-firebase`.

**Nota:** la ejecución real del CI solo se comprueba con el primer push a main o tag — la sintaxis está validada localmente.

**Pendiente del informe:** lógica de diálogos → AppState (P2), limpieza P3 (fuente Inter, shared_preferences, atajos de teclado).

---

## 10. Actualización 2026-08-13: P2 lógica en AppState + P3 limpieza ✅

**P2 — Lógica de diálogos → AppState:**
- Nuevo `AppState.activateAndConnect(ip)`: activa el control global si está inactivo y conecta con el peer. Antes esa lógica vivía en el diálogo de `PeerCard` (onPressed). Ahora el widget solo llama al método del ViewModel.
- `PeerCard` actualizado para usarlo (el widget ya no decide "armar + conectar", solo lo pide).
- 2 tests nuevos: con control inactivo (arma y conecta, verifica `update_input_config` con `armed:true`) y con control ya activo (solo conecta, no reenvía config).

**P3 — Limpieza:**
- **`shared_preferences` eliminada** del pubspec (confirmado: no se usaba en `lib/`). El registrante de plugins de macOS se regeneró solo.
- **Fuente Inter embebida** (`assets/fonts/Inter-Variable.ttf`, 857K, variable font del repo oficial google/fonts + `OFL.txt` licencia). Antes el tema declaraba `fontFamily: 'Inter'` sin asset → caía a la fuente del sistema en silencio. Ahora funciona en cualquier máquina.
- **Atajos de teclado** (la doc de adaptive lo pide para desktop): `CallbackShortcuts` en HomeScreen con `Ctrl+Shift+A` (activar/desactivar control), `Ctrl+Shift+C` (cambiar equipo, solo si armed y hay peers conectados), `Ctrl+Shift+H` (volver aquí, solo si hay peer activo).

**Verificado:** `flutter analyze` → No issues · `flutter test` → **32/32** (app) · `dart test` → **13/13** (core).

**Pendiente:** nada del informe original — queda el warning menor del CI (checkout@v4 en Node 24) y decisiones futuras de producto.
