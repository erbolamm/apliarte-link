# AGENTS.md — Metodología de trabajo (ApliArte Link)

> Leé esto ANTES de tocar cualquier archivo. Aplica a cualquier agente (OpenClaw, Claude, Copilot, Pi) que trabaje en este repo.

## Regla de oro

**Planificar antes de codificar.** Está prohibido lanzarse a editar sin un plan. La iteración a ciegas (muchos commits de hacer/deshacer) es el principal despilfarro de tokens y de tiempo de review.

## Flujo SDD obligatorio

Todo cambio no trivial sigue el flujo SDD y deja artefactos en `openspec/`:

1. `proposal.md` — qué, por qué, alcance, no-goals, riesgos, criterios de aceptación.
2. `spec.md` — requisitos funcionales y no funcionales.
3. `design.md` — arquitectura y decisiones técnicas.
4. `tasks.md` — tareas pequeñas con checkboxes.
5. Implementar siguiendo `tasks.md`, marcando cada tarea al completarla.
6. Verificar: tests + validación contra el spec.
7. Cerrar con el estado final del change.

`openspec/config.yaml` ya declara `execution_mode: auto`, `artifact_store: [engram, openspec]` y `review_budget_lines: 400`.

⚠️ Los artefactos se **actualizan mientras se trabaja**. Un spec que queda congelado mientras el código cambia es tan malo como no tener spec.

## Commits

- Un commit = una unidad de trabajo revisable.
- Conventional commits: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `ci:`.
- Prohibido mezclar varias cosas en un commit "de avance".

## gentle-ai

CLI en `/home/javier/go/bin/gentle-ai`. Comandos seguros (solo lectura):

- `gentle-ai review status --cwd /home/javier/Documentos/repos/apliarte-link`
- `gentle-ai sdd-status <change> --cwd <repo>`
- `gentle-ai skill-registry refresh`

El review transaccional completo (`review start` → lentes → `finalize`) lo ejecuta el **orquestador Pi**, no este agente. Al llegar a un punto revisable, avisá al usuario para que dispare `gentle-ai review` desde Pi. No intentes replicar el ciclo de review por tu cuenta.

## Calidad mínima

- Correr tests antes de dar por terminado:
  - `cargo test --manifest-path native/daemon/Cargo.toml`
  - `flutter test`
- Rust: evitar `unwrap()` / `expect()` en paths de producción (input/red). Un panic crashea el daemon.
- Respetar las skills de `.agents/skills/` (rust-best-practices, accessibility, frontend-design, seo, bash-defensive-patterns).

## Anti-patrones (prohibidos)

- ❌ Muchos commits sin proposal/spec/tasks previos.
- ❌ Specs congelados que no reflejan el código.
- ❌ Creer que gentle-ai ahorra tokens: suma disciplina de review; el ahorro viene de planificar.
- ❌ Dejar docs desactualizadas. Ejemplo real: el código usa **TCP+TLS+MessagePack** (Fase 1), no UDP+JSON.

## Fuente de verdad

El **código** es la fuente de verdad de arquitectura, no los docs. Si un doc contradice al código, corregí el doc.

## División de roles: OpenClaw ejecuta, Pi verifica

Este repo se trabaja con dos agentes que cooperan:

- **OpenClaw** (@aplilinuxbot): implementación con SDD, trabajo autónomo, commits (el gate de RDD no lo bloquea).
- **Pi** (asistente de escritorio): auditorías, validación de reportes contra el código, decisiones de arquitectura, review de código en profundidad, gestión del gate de gentle-ai.

| Situación | Agente |
|---|---|
| Implementar una feature siguiendo SDD | OpenClaw |
| Trabajo autónomo prolongado o repetitivo | OpenClaw |
| Commit de cambios | OpenClaw (Pi queda bloqueado por el gate RDD) |
| Auditoría / validación de un reporte contra el código | Pi |
| Decisión de arquitectura o diseño con tradeoffs | Pi |
| Review de código en profundidad (red, input, seguridad) | Pi |
| Verificación de un claim técnico | Pi |
| Configuración del gate de gentle-ai / RDD | Pi |

Cualquier agente que detecte que la tarea corresponde al otro, debe decírselo al usuario en lugar de hacerla a medias.
