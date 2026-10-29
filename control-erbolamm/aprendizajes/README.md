# Aprendizajes Locales y Escalado Canonico

Este directorio alberga los descubrimientos, patrones técnicos y lecciones aprendidas locales identificados durante el trabajo en este repositorio.

## 1. Sincronización Canónica al Arranque
Al iniciar sesión o abrir este proyecto, el orquestador local (`or-<proyecto>`) debe verificar que cuenta con las lecciones aprendidas canónicas sincronizadas desde la central:
`${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}/roles/orquestador/memoria/lecciones.md`

## 2. Emisión Estructurada a `propuestas.json`
Cuando se descubra un nuevo patrón técnico, mejora de plantilla o regla arquitectónica, el orquestador la registra directamente en:
`control-erbolamm/aprendizajes/propuestas.json`

### Esquema Canónico de Cada Propuesta:
```json
{
  "id": "prop-<proyecto>-<timestamp>",
  "proyecto": "<nombre-del-proyecto>",
  "autor": "or-<proyecto>",
  "fecha": "2026-10-15T06:40:00.000Z",
  "titulo": "Título conciso y descriptivo",
  "tipo": "plantilla | leccion | regla | herramienta",
  "archivoDestino": "roles/orquestador/memoria/lecciones.md",
  "estado": "pendiente",
  "que": "Descripción clara de la mejora o patrón",
  "porque": "Problema real resuelto o justificación técnica",
  "donde": "Ruta o componente afectado",
  "aprendido": "Lección canónica que evita repetir errores",
  "diff": "+ fragmento opcional de código o texto propuesto",
  "notas": ""
}
```

## 3. Conexión con la Bandeja Visual de la Oficina (:8791)
- Todas las propuestas emitidas en `propuestas.json` se agregan de forma federada en:
  `http://127.0.0.1:8791/vista/propuestas?proyecto=<proyecto>`
- Desde dicha interfaz, Javier puede:
  1. **Visualizar**: Leer la propuesta estructurada, justificación y diff.
  2. **Editar / Crear**: Ajustar la redacción o crear nuevas propuestas interactivamente.
  3. **Canonizar**: Con un solo clic, incorporarla como nueva lección secuencial en `roles/orquestador/memoria/lecciones.md` de la Central.
  4. **Descartar o Eliminar**: Marcar como descartada o eliminar si no procede.

## 4. Disciplina Cognitiva de Descarga Inmediata
- **Sin saturación de contexto**: Queda prohibido acumular propuestas o notas técnicas sueltas en la memoria volátil del orquestador.
- Toda mejora debe descargarse de inmediato a `propuestas.json` y comunicarse a Javier a través del buzón visual de ApliChat (`tareas chat responder`).

## 5. Soberanía Estricta entre Repositorios
Queda terminantemente prohibido que cualquier orquestador edite directamente archivos de un repositorio ajeno. La canonización en la Central la ejecuta exclusivamente `or-erbolamm-trabajo` a través de la Bandeja de Propuestas de la Oficina.

