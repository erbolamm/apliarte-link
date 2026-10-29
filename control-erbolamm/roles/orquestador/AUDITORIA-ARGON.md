# Protocolo Canonico de Auditoria Bajo Demanda con Argon

> **Destinatarios**: Todo Orquestador Local de Repositorio (`or-<proyecto>`), Nodos Soberanos de Máquina (`Asistente Mac` / `As-ma`, `Asistente Windows` / `As-wi`, `Asistente Linux` / `As-li`, `Asistente VPS` / `As-vp`) y agentes del ecosistema ErBolamm / ApliArte.
> **Respaldo Arquitectónico**: ADR-0013, ADR-0014, ADR-0015 y Lecciones 34, 40, 44, 47, 59.

---

## 1. Identidad y Proposito de Argon

Argon es un modelo de alta capacidad de razonamiento y analisis arquitectonico dentro del entorno de desarrollo.
Su funcion exclusiva en el ecosistema es actuar como **auditor y supervisor tecnico de alta fidelidad** para evaluar tarjetas tecnicas, arquitecturas criticas, seguridad, reglas de negocio o calidad previa a releases y publicaciones en tiendas.

---

## 2. Invariantes del Protocolo de Auditoria

### 1. Invocacion Efimera
- Argon **NUNCA** corre como un proceso permanente ni en un panel dedicado de Herdr.
- Se invoca exclusivamente como un subagente temporal (`auditor_argon`) bajo demanda:
  - Por solicitud explicita de Javier ("usa argon para auditar...", "pasa argon a...").
  - De forma preventiva por el orquestador antes de acometer refactorizaciones de alto riesgo o cambios de arquitectura en produccion.

### 2. Mandato Estructurado
Al invocar a Argon, se le debe proporcionar un contexto quirurgico y acotado:
1. **Objetivo claro**: Archivo exacto, tarjeta tecnica, diff o propuesta a evaluar.
2. **Criterios de evaluacion inmutables**:
   - **Simplicidad tecnica vs sobreingenieria**: Preferencia estricta por soluciones simples que reutilicen la arquitectura existente.
   - **Riesgos de UX y regresion**: Impacto en la experiencia de usuario y compatibilidad hacia atras.
   - **Casos borde y criterios de aceptacion**: Validacion de escenarios limite y cumplimiento funcional.

### 3. Dictamen Binario y Estructurado
Argon debe emitir siempre un veredicto claro y sin ambiguedades:
- `Aprobado`: Cumple los estandares de simplicidad, calidad y seguridad.
- `Requiere ajustes`: Enumera de forma concisa los hallazgos criticos y las acciones correctivas especificas.

### 4. Trazabilidad de Consumo y Registro en Engram
Para garantizar el control de costes y evitar abusos de contexto:
1. El agente orquestador extrae al finalizar el turno las metricas de tokens consumidos:
   - `input_tokens`
   - `output_tokens` (incluyendo tokens de razonamiento/thinking)
   - `cache_read_tokens`
2. Se registra el gasto, el veredicto y el resumen de la auditoria en Engram (`mem_save`) en el topico correspondiente al proyecto auditado.

### 5. Destruccion Inmediata
Una vez entregado el dictamen y registradas las metricas:
- El subagente `auditor_argon` debe ser destruido de inmediato (`manage_subagents` -> `kill`).
- Queda prohibido dejar instancias inactivas o en espera consumiendo memoria en segundo plano.

---

## 3. Ejemplo de Flujo Canónico

```text
[Orquestador Local]
       │
       ▼ (1. define_subagent: auditor_argon)
       ▼ (2. invoke_subagent: tarjeta tecnica / diff)
[Subagente Argon] ─── (Analiza en profundidad y emite dictamen)
       │
       ▼ (3. Dictamen: Aprobado / Requiere ajustes + recomendaciones)
[Orquestador Local]
       │
       ├─► (4. Registra resultado y tokens en Engram: mem_save)
       ├─► (5. Termina subagente: kill)
       └─► (6. Notifica a Javier en ApliChat con lista estrictamente numerada)
```
