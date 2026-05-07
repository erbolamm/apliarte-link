#!/usr/bin/env bash
# Bootstrap de memoria y agentes para repos privados de ApliArte.
# No instala herramientas automáticamente. Diagnostica, explica y aplica configuración solo con --apply.

set -euo pipefail

APPLY=false
SETUP_AGENTS=false
DEFAULT_PROJECT="apliarte-link"
PROJECT_OVERRIDE="${ENGRAM_PROJECT:-}"
CLOUD_SERVER="${ENGRAM_CLOUD_SERVER:-https://engram.apliarte.com}"

usage() {
  cat <<'USAGE'
Uso:
  ./scripts/bootstrap-agent.sh [opciones]

Opciones:
  --apply             Aplica configuración Engram/import/sync. Sin esto solo muestra diagnóstico.
  --setup-agents      Configura agentes soportados por Engram y ejecuta gentle-ai sync si existe.
  --project <nombre>  Sobrescribe el nombre de proyecto para Engram.
  --cloud-server <url> Sobrescribe la URL de Engram Cloud.
  -h, --help          Muestra esta ayuda.

Variables:
  ENGRAM_PROJECT       Nombre de proyecto por defecto.
  ENGRAM_CLOUD_SERVER  URL de Engram Cloud. Default: https://engram.apliarte.com
  ENGRAM_CLOUD_TOKEN   Token bearer si el cloud lo requiere. No se guarda en Git.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --apply)
      APPLY=true
      shift
      ;;
    --setup-agents)
      SETUP_AGENTS=true
      shift
      ;;
    --project)
      if [[ $# -lt 2 || "${2:-}" == --* ]]; then
        echo "Falta valor para --project" >&2
        exit 2
      fi
      PROJECT_OVERRIDE="${2:-}"
      shift 2
      ;;
    --cloud-server)
      if [[ $# -lt 2 || "${2:-}" == --* ]]; then
        echo "Falta valor para --cloud-server" >&2
        exit 2
      fi
      CLOUD_SERVER="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Opción desconocida: $1" >&2
      usage
      exit 2
      ;;
  esac
done

have() {
  command -v "$1" >/dev/null 2>&1
}

repo_root() {
  if have git && git rev-parse --show-toplevel >/dev/null 2>&1; then
    git rev-parse --show-toplevel
  else
    pwd
  fi
}

infer_project() {
  if [[ -n "$PROJECT_OVERRIDE" ]]; then
    printf '%s\n' "$PROJECT_OVERRIDE"
    return
  fi

  if [[ -n "$DEFAULT_PROJECT" ]]; then
    printf '%s\n' "$DEFAULT_PROJECT"
    return
  fi

  if have git && git remote get-url origin >/dev/null 2>&1; then
    local remote
    remote="$(git remote get-url origin)"
    local name
    name="${remote##*/}"
    name="${name%.git}"
    if [[ -n "$name" ]]; then
      printf '%s\n' "$name"
      return
    fi
  fi

  basename "$(repo_root)"
}

ROOT="$(repo_root)"
PROJECT="$(infer_project)"

cd "$ROOT"

echo "== ApliArte agent bootstrap =="
echo "Repo: $ROOT"
echo "Proyecto Engram: $PROJECT"
echo "Cloud: $CLOUD_SERVER"
echo "Modo: $([[ "$APPLY" == true ]] && echo aplicar || echo diagnostico)"
echo ""

missing=0

if have engram; then
  echo "OK engram: $(command -v engram)"
else
  echo "FALTA engram"
  echo "  Instálalo manualmente antes de aplicar sync. En macOS con Homebrew, si tu tap lo soporta:"
  echo "  brew install engram"
  missing=1
fi

if have gentle-ai; then
  echo "OK gentle-ai: $(command -v gentle-ai)"
else
  echo "AVISO gentle-ai no está instalado. Solo hace falta si querés sincronizar configuración de agentes Gentle AI."
fi

if have gh; then
  echo "OK gh: $(command -v gh)"
else
  echo "AVISO gh no está instalado. No es obligatorio para Engram, pero ayuda en repos GitHub."
fi

echo ""

if [[ -f ".github/copilot-instructions.md" ]]; then
  echo "OK instrucciones Copilot: .github/copilot-instructions.md"
else
  echo "AVISO falta .github/copilot-instructions.md"
fi

if [[ -d ".engram" ]]; then
  echo "OK chunks locales Engram detectados: .engram/"
else
  echo "INFO no hay .engram/ en este checkout. Se usará cloud si está configurado."
fi

echo ""

if [[ "$APPLY" != true ]]; then
  echo "Diagnóstico completado. No se modificó nada."
  echo "Para aplicar configuración:"
  echo "  ENGRAM_CLOUD_TOKEN=... ./scripts/bootstrap-agent.sh --apply"
  echo "Para configurar agentes soportados:"
  echo "  ENGRAM_CLOUD_TOKEN=... ./scripts/bootstrap-agent.sh --apply --setup-agents"
  exit 0
fi

if [[ "$missing" -ne 0 ]]; then
  echo "No puedo aplicar configuración porque faltan herramientas obligatorias." >&2
  exit 1
fi

echo "Configurando Engram Cloud..."
engram cloud config --server "$CLOUD_SERVER"
engram cloud enroll "$PROJECT"

if [[ -d ".engram" ]]; then
  echo "Importando chunks locales desde .engram/..."
  engram sync --import
fi

if [[ -z "${ENGRAM_CLOUD_TOKEN:-}" ]]; then
  echo "AVISO ENGRAM_CLOUD_TOKEN no está definida. Si el servidor exige auth, el sync cloud fallará."
fi

echo "Sincronizando con Engram Cloud..."
engram sync --cloud --project "$PROJECT"

if [[ "$SETUP_AGENTS" == true ]]; then
  echo ""
  echo "Configurando agentes soportados por Engram..."
  for agent in opencode claude-code gemini-cli codex; do
    echo "-- engram setup $agent"
    engram setup "$agent"
  done

  if have gentle-ai; then
    echo "Ejecutando gentle-ai sync..."
    gentle-ai sync
  fi
else
  echo ""
  echo "Saltando configuración de agentes. Usá --setup-agents para aplicarla."
fi

echo ""
echo "Bootstrap completado."
