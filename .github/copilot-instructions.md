# Copilot Instructions — ApliArte Link

Estas instrucciones aplican a todo el repositorio `apliarte-link`.

## Producto

ApliArte Link es un KVM virtual por red local. El objetivo es compartir teclado y ratón entre ordenadores de la misma LAN sin hardware extra.

El repositorio tiene tres superficies principales:

- `native/daemon/`: daemon Rust que captura, envía, recibe e inyecta eventos de input.
- `apps/apliarte_link/`: app Flutter Desktop que arranca o controla el daemon mediante IPC local.
- `web/`: web estática desplegada en Firebase Hosting.

## Arquitectura

- El core Rust es el motor del producto y corre en segundo plano.
- La UI Flutter no debe duplicar lógica del daemon; debe pedir estado o enviar comandos por IPC.
- El IPC local usa TCP loopback `127.0.0.1:4243` con mensajes JSON por línea.
- La comunicación entre máquinas usa TCP+TLS en el puerto `4242` con eventos serializados como MessagePack. El discovery entre peers sigue siendo UDP por broadcast en el puerto `4244`.
- Los backends de input son específicos por sistema operativo: macOS, Windows y Linux.
- Mantener separadas las responsabilidades: `network` descubre/recibe/envía, `input` captura/inyecta, `ipc` conversa con la UI, `config` persiste configuración.

## Estilo De Trabajo

- Antes de editar, leer el archivo afectado y seguir el patrón existente.
- Mantener cambios mínimos y centrados en la tarea.
- No reescribir arquitectura ni renombrar módulos salvo que la tarea lo pida explícitamente.
- No tocar cambios no relacionados del usuario.
- No añadir atribución de IA ni trailers `Co-Authored-By`.
- No ejecutar builds después de cambios salvo petición explícita.
- Preferir soluciones simples y verificables frente a abstracciones prematuras.

## Rust Core

- Usar `anyhow::Result` para propagación de errores en rutas de aplicación.
- Usar `tracing` para logs; no usar `println!` para eventos del daemon.
- Mantener el código async sobre Tokio cuando toque red, IPC o tareas de larga vida.
- No bloquear dentro de secciones con `Mutex` más de lo necesario.
- Validar entrada externa en IPC y red; responder errores por IPC cuando el comando sea inválido.
- Mantener código específico de plataforma detrás de `#[cfg(target_os = "...")]`.

Comandos útiles:

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --all --check
```

## Flutter UI

- Usar `Provider` y `ChangeNotifier` siguiendo el patrón actual de `AppState`.
- Mantener servicios de infraestructura en `apps/apliarte_link/lib/services/` y contratos/modelos compartidos en `packages/apliarte_link_core/`.
- La UI debe reflejar estado recibido del daemon; evitar estados paralelos que puedan desincronizarse.
- Cerrar streams, procesos y sockets en `dispose` o métodos equivalentes.
- Usar `debugPrint` para logs de desarrollo.

Comandos útiles:

```bash
cd apps/apliarte_link && flutter test
cd apps/apliarte_link && flutter analyze
cd apps/apliarte_link && dart format --set-exit-if-changed .
```

## Web Y Deploy

- `web/public/` contiene la web pública.
- `web/firebase.json` configura Firebase Hosting.
- El workflow `Deploy web a Firebase Hosting` se dispara en `main` cuando cambia `web/**`.
- No modificar archivos de cache o artefactos generados salvo que la tarea sea explícitamente de despliegue.

## Testing

Strict TDD está activo para este proyecto.

- Si se añade lógica nueva en Rust, añadir o actualizar tests cuando sea razonable.
- Si se añade lógica nueva en Flutter, añadir o actualizar tests de widget/unit cuando sea razonable.
- No hay E2E configurado actualmente; no inventar framework E2E sin decisión explícita.
- Si una parte no es testeable sin gran refactor, explicar el hueco y proponer el refactor mínimo.

## SDD

El proyecto fue inicializado con SDD en modo Engram.

## Memoria Y Bootstrap De Agentes

- La estrategia de memoria del repo es híbrida: Git privado para instrucciones/bootstrap y Engram Cloud para sincronización viva entre ordenadores.
- La documentación está en `docs/agent-sync.md`.
- El bootstrap está en `scripts/bootstrap-agent.sh`.
- No guardar tokens en Git; usar `ENGRAM_CLOUD_TOKEN` como variable de entorno.
- Si se clona el repo en un ordenador nuevo, ejecutar primero `./scripts/bootstrap-agent.sh` para diagnóstico y luego `ENGRAM_CLOUD_TOKEN=... ./scripts/bootstrap-agent.sh --apply` para sincronizar.
- Para configurar agentes locales soportados por Engram, usar `./scripts/bootstrap-agent.sh --apply --setup-agents` después de revisar lo que va a tocar.

Topic keys relevantes:

- `sdd-init/apliarte-link`
- `sdd/apliarte-link/testing-capabilities`
- `skill-registry`

Nota: Engram puede haber indexado estos topic keys bajo el proyecto auto-detectado `apliarte`. Si una búsqueda con `project: apliarte-link` no devuelve resultados, buscar por topic key sin filtro de proyecto.
