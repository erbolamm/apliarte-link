# Protocolo de Deliberacion de Mesa Redonda en Proyectos

Este documento define el procedimiento canonico para convocar y documentar deliberaciones colegiadas dentro de este repositorio soberano, de conformidad con `DOCUMENTACION/PROTOCOLO-MESA-REDONDA.md`.

## Requisitos Obligatorios de Dictamen

1. **Identificacion de participantes**:
   - Nombre del agente, modelo exacto, terminal o pane de Herdr en el que delibero.
2. **Encargo identico**:
   - Registro exacto de la consulta u orden del dia planteada a todos los miembros.
3. **Evidencia verificable**:
   - Cita literal o extracto contrastable de la respuesta emitida por cada agente.
4. **Declaracion de ausencias**:
   - Registro transparente de cualquier fallo de API, timeout o falta de cuota.
5. **Sintesis de consensos y disensos**:
   - Resumen claro de los puntos de acuerdo unanime y de las divergencias tecnicas identificadas.

## Flujo Operativo

1. El Orquestador Local formula el problema con opciones tecnicas fundamentadas.
2. Despacha la consulta a los terminales de deliberacion (`mr-*`: `mr-claude`, `mr-codex`, `mr-agy`, `mr-pi`, etc.).
3. Recopila las respuestas sin sesgo ni manipulacion.
4. Sintetiza el dictamen colegiado y lo documenta en `control-erbolamm/aprendizajes/` o en la bitacora de la tarea.

