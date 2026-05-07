# Agent Memory Sync

Este repositorio está preparado para trabajar con memoria de agentes en modo híbrido:

- **Git privado** guarda instrucciones, bootstrap y, si se decide exportar, chunks `.engram/`.
- **Engram Cloud** centraliza memoria entre ordenadores.
- **Variables de entorno** guardan secretos como tokens. Nunca se commitean tokens.

## Qué Viaja Al Clonar

Al clonar el repositorio en otro ordenador viajan estos elementos:

- `.github/copilot-instructions.md`: instrucciones del repo para Copilot y agentes compatibles.
- `docs/agent-sync.md`: este documento.
- `scripts/bootstrap-agent.sh`: comprobador/bootstrap de entorno de agentes.
- `.engram/`: solo si existe y fue exportado/commiteado explícitamente.

No viajan automáticamente:

- La base de datos local de Engram (`~/.engram`).
- Tokens de cloud.
- Configuración personal de cada editor.
- El registry local `.atl/`, porque es infraestructura generada e ignorada por Git.

## Primer Arranque En Un Ordenador Nuevo

Desde la raíz del repo:

```bash
./scripts/bootstrap-agent.sh
```

Ese comando hace comprobaciones sin modificar nada. Para aplicar configuración:

```bash
ENGRAM_CLOUD_TOKEN="tu-token" ./scripts/bootstrap-agent.sh --apply
```

Si querés intentar configurar también los agentes soportados por Engram:

```bash
ENGRAM_CLOUD_TOKEN="tu-token" ./scripts/bootstrap-agent.sh --apply --setup-agents
```

## Variables De Entorno

| Variable | Uso |
|---|---|
| `ENGRAM_PROJECT` | Sobrescribe el nombre de proyecto detectado. |
| `ENGRAM_CLOUD_SERVER` | URL del servidor Engram Cloud. Por defecto: `https://engram.apliarte.com`. |
| `ENGRAM_CLOUD_TOKEN` | Token bearer para servidores que exigen autenticación. No se guarda en Git. |

Ejemplo:

```bash
export ENGRAM_PROJECT="apliarte-link"
export ENGRAM_CLOUD_SERVER="https://engram.apliarte.com"
export ENGRAM_CLOUD_TOKEN="tu-token"
./scripts/bootstrap-agent.sh --apply --setup-agents
```

## Flujo Híbrido Recomendado

1. Clonar el repo privado.
2. Ejecutar `./scripts/bootstrap-agent.sh` para ver el diagnóstico.
3. Instalar herramientas faltantes manualmente si el script las reporta.
4. Exportar `ENGRAM_CLOUD_TOKEN` si el servidor cloud requiere autenticación.
5. Ejecutar `./scripts/bootstrap-agent.sh --apply`.
6. Ejecutar `./scripts/bootstrap-agent.sh --apply --setup-agents` solo cuando quieras tocar configuración local de agentes.
7. Si el repo trae `.engram/`, el script importa chunks locales con `engram sync --import`.
8. Si cloud está disponible, el script enrola el proyecto y ejecuta `engram sync --cloud --project <proyecto>`.

## Comandos Manuales Equivalentes

```bash
engram cloud config --server https://engram.apliarte.com
engram cloud enroll apliarte-link
engram sync --import
ENGRAM_CLOUD_TOKEN="tu-token" engram sync --cloud --project apliarte-link
```

Para agentes soportados por Engram:

```bash
engram setup opencode
engram setup claude-code
engram setup gemini-cli
engram setup codex
gentle-ai sync
```

## Seguridad

- No commitear tokens, `.env`, claves privadas ni dumps completos de configuración personal.
- Revisar cualquier chunk `.engram/` antes de commitearlo si contiene memoria sensible.
- Preferir Engram Cloud para sincronización real entre varios ordenadores y varias cuentas/editoriales.
- Usar `.engram/` como respaldo portable o semilla de proyecto, no como único sistema de sincronización.

## Decisión Arquitectónica

El enfoque correcto para repos privados es híbrido:

- Git define el contrato de trabajo del repo.
- Engram Cloud mantiene memoria viva y sincronizada.
- El bootstrap reduce fricción en máquinas nuevas sin esconder secretos ni instalar herramientas sin permiso.

Esto permite borrar un ordenador, clonar el repositorio en otro y recuperar instrucciones + memoria sincronizada con un flujo repetible.
