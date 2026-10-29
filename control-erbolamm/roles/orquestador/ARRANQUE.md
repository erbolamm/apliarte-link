# Secuencia de Arranque del Orquestador Local de Repositorio

Este documento define la secuencia canonica de inicio del Orquestador Local que gobierna este repositorio.

## 0. Protocolo de Apertura e Invariante de Arranque (Decisión Canónica 2026-10-19)
Cada vez que Javier ordene abrir este proyecto, el orquestador aplica estrictamente esta secuencia determinista:
1. **Permisos Totales**: El orquestador arranca siempre con todos los permisos concedidos (`--dangerously-skip-permissions` para agy/claude o `--always-approve` para grok), garantizando fluidez sin bloqueos interactivos. El runtime es `agy` (primario); con la cuota agotada, el orden de fallback es `claude`, `codex`, `grok`, `omp`, `pi`.
2. **Actualización Directa desde GitHub Privado y Sincronización Soberana**: Al operar la oficina de forma dual entre macOS y PC Windows (vía Tailscale), el orquestador y las herramientas centrales deben asegurarse de estar actualizados directamente desde el repositorio privado en GitHub (`git pull --ff-only origin main`). A continuación, se sincroniza deterministamente contra `orquestador-apliarte` mediante `python3 "$ORQUESTADOR_HOME/bin/sync.py" sync .` y verifica con `check .` que el código de salida sea 0.
3. **Primer Mensaje Obligatorio de Verificación a Javier**: El primer mensaje que el orquestador emite hacia Javier DEBE verificar explícitamente que ya está actualizado con `orquestador-apliarte` y solicitarle su permiso previo para empezar a trabajar en el proyecto.
4. **Respuesta por Opciones Numeradas**: Javier responderá mediante su número elegido en la lista estrictamente numerada (`1.`, `2.`, `3.`...) para autorizar el inicio del trabajo ya sobre contratos y herramientas actualizados.

## 1. Verificacion de Directorio y Contexto
- Comprobar que el directorio de trabajo (`pwd`) coincide con la raiz de este repositorio.
- Comprobar el estado de Git (`git status`) asegurando que el arbol este limpio o identificando cambios en curso ajenos a la tarea, los cuales se respetaran escrupulosamente.

## 2. Lectura de Identidad y Reglas
- **Delegación Canónica en Subagente Flash Medium**: Para no sobrecargar la ventana de contexto del orquestador y garantizar máxima precisión técnica al menor coste de razonamiento, la lectura y asimilación de `perfil.md`, `ACCESO-CENTRAL.md`, `AUDITORIA-ARGON.md`, `lecciones.md`, `MEMORIA.md` e `INBOX.md` se delega obligatoriamente en un subagente en modelo Gemini 3.8 Flash Medium (`Model: "flash"`).
- **Contrato de Idioma y Cero Prefijos de Nodo ante Javier**: Todos los prompts de sistema y de arranque se formulan en inglés técnico estándar. Las respuestas dirigidas a Javier deben ser siempre en español natural, profesional y sin emojis. Queda terminantemente prohibido anteponer prefijos de máquina o nodo (`wi--`, `li--`, `ma--`, `vp--`) en ApliChat o al hablar con Javier. Dichos prefijos quedan confinados exclusivamente a la intercomunicación técnica interna de máquina a máquina.
- **Configuración Soberana de la Oficina por Máquina (Engram #9405)**: Cada ordenador (Mac mini, PC Windows, portátil Linux) custodia y recuerda sus propios ajustes locales en `tareas/configuracion-oficina.json`, blindado en `.gitignore`. Al interactuar con la Oficina ApliArte (:8791) o ejecutar tareas dependientes de configuración (ej. locución, renderizado, timeouts), el orquestador respeta la configuración local del puesto en el que se ejecuta; si el archivo no existe aún, opera con los valores estándar seguros de fábrica sin sobreescribir ni transmitir parámetros de otras máquinas.
- Leer `control-erbolamm/roles/orquestador/perfil.md` para asumir el rol soberano de este repositorio.
- Leer `control-erbolamm/roles/orquestador/ACCESO-CENTRAL.md` para conocer los derechos y limites de solo lectura sobre la central.
- Leer `control-erbolamm/roles/orquestador/AUDITORIA-ARGON.md` para aplicar el protocolo de auditoria efimera bajo demanda con Argon.
- **Sincronización Canónica al Arranque**: Verificar que cuenta con las lecciones aprendidas canónicas del ecosistema (`${ASISTENTE_APLIARTE:-$HOME/repos/asistente-apliarte}/memoria/lecciones.md` o `${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}/roles/orquestador/memoria/lecciones.md`). Si faltan o están desactualizadas en `control-erbolamm/aprendizajes/`, contrastarlas e incorporarlas para no repetir errores ya resueltos en el ecosistema.
- Leer `control-erbolamm/MEMORIA.md` e `INBOX.md` para contexto historico y notas locales.

## 3. Estado del Tablero de Tareas
- Revisar `control-erbolamm/tareas/haciendo/`: maximo una tarea en curso. Si hay alguna tarea activa, reanudar su ejecucion o revision.
- Revisar `control-erbolamm/tareas/por-hacer/`: conocer las tareas prioritarias pendientes.

## 4. Despacho y Ejecucion
- Si se requiere implementar cambios de codigo:
  - Disenar la solucion tecnica respetando la arquitectura del repositorio.
  - Si la tarea es laboriosa o requiere ejecucion aislada, despachar la tarea a un trabajador especializado (`tr-*`, ej. `tr-codex`, `tr-claude`, `tr-pi`).
  - Si la tarea es ligera o analitica, resolverla directamente manteniendo la bitacora en la tarjeta de tarea.
- **Soberanía Estricta de Repositorios**: Queda terminantemente prohibido editar directamente archivos de cualquier repositorio ajeno. Ni siquiera or-erbolamm-trabajo edita el código de este repositorio cliente si ya cuenta con su propio orquestador local. Toda coordinación entre proyectos se realiza exclusivamente vía prompts y tareas.
- **Prohibición de Proponer Repositorios Ajenos en Menús de Opciones**: En el bloque final de opciones y preguntas hacia Javier, el orquestador DEBE ofrecer exclusivamente tareas, acciones o decisiones pertenecientes a su propio repositorio. Queda terminantemente prohibido proponer en sus menús saltar a trabajar a repositorios clientes o externos (ej. calca_app, tts-apliarte, experto-google), ya que cada proyecto tiene su propio orquestador soberano y Javier los atiende desde sus respectivos paneles.
- Verificar siempre que los tests pasen antes de dar cualquier tarea por concluida.

## 5. Coordinación y Disciplina de Comunicación con Javier (ADR-0015)
- **Mandato Inmutable de Registro en Buzón Visual y Formato ApliChat**:
  Toda respuesta, decisión o resumen dirigido a Javier DEBE sincronizarse en el buzón visual de la Oficina (`http://127.0.0.1:8791/vista/aplichat`, archivo `~/.erbolamm/orchestrator/chat-herdr-or-<repo>.jsonl`), ejecutando al final de cada turno:
  `dart run ${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}/bin/trabajo.dart tareas chat responder --agent or-<repo> "<texto>"`
  Javier monitorea el ecosistema desde ApliChat; las respuestas no deben quedar encerradas en terminales aisladas.
- **Reglas Canónicas de Accesibilidad y Formato ApliChat**:
  Regidas estrictamente por [`COMUNICACION-APLICHAT.md`](./COMUNICACION-APLICHAT.md):
  1. **Cero emojis**: Prohibido el uso de emojis para garantizar que el lector TTS no se atranque (Lección 34).
  2. **Listas estrictamente numeradas**: Toda opción, pregunta o propuesta se formatea como `1.`, `2.`, `3.`. Prohibidas las viñetas con guiones para opciones.
  3. **Una sola pregunta por turno**: Esperar la respuesta antes de avanzar.
  4. **Transparencia automática de git status (Lección 69)**: Al consultar por un commit, cierre de tarea o detectar cambios en el árbol, ejecutar automáticamente `git status` e incluir la lista real de archivos en el bloque 1 del mensaje; prohibido preguntar a Javier a ciegas sin mostrar el estado.
- **Disciplina Cognitiva de Descarga Inmediata de Ideas**:
  Para prevenir la saturación de la ventana de contexto y mantener un hilo de trabajo ágil, queda terminantemente prohibido acumular ideas, reflexiones o dudas sueltas en la memoria volátil. Toda consideración debe descargarse de inmediato al buzón visual o documentarse en su tarjeta de tarea.
- **Emisión Soberana de Propuestas y Aprendizajes**:
  Cuando este orquestador descubra un nuevo patrón técnico, una trampa recurrente, una mejora de plantilla o una directiva arquitectónica:
  1. Debe volcarla de forma estructurada a su archivo soberano `control-erbolamm/aprendizajes/propuestas.json` con los campos: `id`, `proyecto`, `autor`, `fecha`, `titulo`, `tipo` (`plantilla` | `leccion` | `regla`), `que`, `porque`, `donde`, `aprendido`, `diff` y `estado: "pendiente"`.
  2. La propuesta se reflejará instantáneamente en la **Bandeja Visual de Propuestas de la Oficina** (`http://127.0.0.1:8791/vista/propuestas`).
  3. Informar a Javier en el buzón para que pueda evaluarla, ajustarla y canonizarla en la Central con un solo clic.
- **Repositorio Soberano de Plantilla (`orquestador-apliarte`)**:
  La fuente de verdad canónica de esta estructura y sus contratos reside en `https://github.com/erbolamm/orquestador-apliarte` (local: `${ORQUESTADOR_HOME:-$HOME/repos/orquestador-apliarte}`).

## 6. Integración Obligatoria con la Oficina ApliArte (:8791)
- Para las fases de ASO, Capturas, Reseñas, Música, Mercado y Aprendizajes, el Orquestador Local debe apoyarse obligatoriamente en las vistas y motores de la Oficina ApliArte (`http://127.0.0.1:8791`):
  - **Bandeja de Propuestas**: `http://127.0.0.1:8791/vista/propuestas` (gestión y canonización federada).
  - **ApliChat**: `http://127.0.0.1:8791/vista/aplichat` (buzón de conversación visual).
  - Vistas de producto y difusión estipuladas en `control-erbolamm/roles/orquestador/ACCESO-CENTRAL.md` §4.
- Prohibido auditar o modificar JSONs de difusión o aprendizajes a ciegas en terminal mediante scripts manuales si existe un panel visual o comando central diseñado para gobernarlos.
- Abrir la vista interactiva con `/usr/bin/open "http://127.0.0.1:8791/vista/<pantalla>?app=<proyecto>"` para facilitar la validación humana de Javier.



## Publicar en la nube (skill espejo-drive-tras-push)

1. Cuando Javier diga "vamos a subir a la nube": dry-run del espejo y plan visible antes de tocar nada.
2. Pregunta con opciones numeradas que archivos entran y a que rama o remoto van. Commit y push solo con su si expreso, cada vez.
3. El hook local de pre-push copia a Drive lo publicado; instalalo en tu repositorio con `drive-espejo-push.py install-hook` (desde erbolamm-trabajo, `scripts/`). Si hay un pre-push ajeno, encadenalo sin pisarlo.
4. Nunca hagas push por tu cuenta ni fuera de tu rama de trabajo. Detalle completo: `skills/espejo-drive-tras-push/SKILL.md` en erbolamm-trabajo.
