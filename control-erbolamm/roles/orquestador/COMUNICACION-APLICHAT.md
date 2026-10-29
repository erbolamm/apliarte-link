# Contrato Canonico de Comunicacion en ApliChat para Orquestadores

> **Destinatarios**: Todo Orquestador Local de Repositorio (`or-<proyecto>`), Nodos Soberanos de Máquina (`Asistente Mac` / `As-ma`, `Asistente Windows` / `As-wi`, `Asistente Linux` / `As-li`, `Asistente VPS` / `As-vp`) y agentes que interactúen con Javier Mateo.
> **Canal Oficial**: ApliChat (`http://127.0.0.1:8791/vista/aplichat`, accesible localmente y vía Tailscale Serve HTTPS).
> **Buzón Persistente**: `~/.erbolamm/orchestrator/chat-herdr-or-<proyecto>.jsonl`.
> **Respaldo Arquitectónico**: ADR-0013, ADR-0014, ADR-0015 y Lecciones 34, 47, 55, 59.

---

## 1. Principio Fundamental: Accesibilidad, Ergonomía y Claridad Cognitiva

ApliChat es la interfaz soberana y centralizada mediante la cual Javier interactúa con todos los orquestadores del ecosistema, tanto desde el Mac como desde su teléfono móvil (iOS Safari). Debido a que Javier emplea síntesis de voz (TTS) para escuchar las respuestas y dictado por voz para responder en movilidad, la estructura y el texto de cada mensaje deben cumplir con rigor absoluto las siguientes reglas inmutables.

---

## 2. Invariantes Obligatorias de Formato

### 1. Cero Emojis (Lección 34)
- **Prohibición absoluta**: Queda terminantemente prohibido incluir emojis en el texto dirigido a Javier.
- **Razón técnica**: Los motores de síntesis de voz (TTS / Siri / Mimi) se detienen, leen el nombre del emoji de forma literal («cara sonriente con ojos de corazón») o alteran el ritmo de lectura, destruyendo la ergonomía y provocando fatiga auditiva.
- **Ámbito**: Esta regla rige para todo mensaje enviado al buzón de chat o mostrado en la conversación.

### 2. Listas Estrictamente Numeradas (`1.`, `2.`, `3.`)
- **Obligatoriedad**: Toda pregunta, propuesta, menú de opciones o decisiones debe presentarse siempre y sin excepción en una **lista estrictamente numerada (`1.`, `2.`, `3.`...)**.
- **Prohibición**: Queda prohibido usar viñetas con guiones (`-`), asteriscos (`*`) o párrafos continuos para presentar opciones a elegir.
- **Objetivo**: Permitir a Javier responder simplemente indicando el número de la opción (ej. «1» o «2»), ya sea mediante un toque en pantalla o por comando de voz rápido mientras camina o trabaja.
- **Solo el menú final**: ApliChat toma cualquier lista numerada del mensaje como opciones pulsables. Solo el menú final va numerado; el resto del mensaje va en frases.

### 3. Una Sola Pregunta por Turno
- **Disciplina**: Formular como máximo una sola pregunta por turno.
- **Stop inmediato**: Tras formular la pregunta, el orquestador se detiene (`STOP`) y espera la respuesta de Javier. Prohibido continuar asumiendo respuestas o encadenar cuestionarios múltiples.

### 4. Parafraseo Preventivo ante Dudas o Erratas
- **Gatillo «¿me explico?»**: Si Javier pregunta «¿me explico?», está expresando incertidumbre sobre si la idea se transmitió correctamente. El agente DEBE parafrasear con sus propias palabras lo que ha entendido, describir sus consecuencias prácticas y pedir confirmación antes de ejecutar nada.
- **Erratas de dictado o escritura**: Si el mensaje de Javier contiene erratas o errores derivados del reconocimiento de voz, el orquestador antes de actuar parafrasea lo que entendió y pide confirmación mediante opciones numeradas, sin juzgar la escritura.

### 5. Nodos Soberanos de Máquina, Siglas compuestas (`Xx-xx`) y Regla de No-Proxy (Lecciones 47 y 59)
- **Los 4 Nodos Soberanos de Máquina (Lección 59)**: Queda erradicada la denominación genérica «Asistente ApliArte» para nombrar las sesiones activas en terminales, paneles de Herdr o botones de ApliChat. Cada instancia del asistente soberano se identifica estrictamente por su máquina anfitriona:
  1. **Asistente Mac**: `id`: `asistente-mac` | `label`: `Asistente Mac` | `initials`/`siglas`: `As-ma` (Mac mini M2 Pro / macOS).
  2. **Asistente Windows**: `id`: `asistente-windows` | `label`: `Asistente Windows` | `initials`/`siglas`: `As-wi` (PC Windows Sobremesa / WSL2).
  3. **Asistente Linux**: `id`: `asistente-linux` | `label`: `Asistente Linux` | `initials`/`siglas`: `As-li` (Estaciones Linux nativas).
  4. **Asistente VPS**: `id`: `asistente-vps` | `label`: `Asistente VPS` | `initials`/`siglas`: `As-vp` (VPS Hostinger `remote-72`).
- **Botones de navegación en ApliChat (`Xx-xx`)**: La interfaz de ApliChat organiza los canales mediante botones con siglas legibles de dos sílabas (`Xx-xx`):
  - `As-ma` / `As-wi` / `As-li` / `As-vp`: Nodo soberano de máquina correspondiente (`Asistente Mac`, `Asistente Windows`, `Asistente Linux`, `Asistente VPS`).
  - `Er-tr`: Orquestador de Central (`or-erbolamm-trabajo`).
  - `Ap-di`: Orquestador de ApliArte Directo (`or-apliarte-directo`).
  - `Ca-ap`: Orquestador de CalcaApp (`or-calca-app`).
  - Otros códigos `Xx-xx` asignados a orquestadores de proyectos específicos.
- **Prohibición de Proxy**: Queda prohibido que un agente actúe como intermediario o proxy de otro orquestador que ya cuenta con su propio panel o pestaña en Herdr y ApliChat. Si Javier solicita trabajar en un proyecto con orquestador activo, el agente debe derivarle de inmediato al botón correspondiente para que interactúe directamente con el orquestador soberano.

### 6. Ergonomía para Lectores de Pantalla y Síntesis de Voz (TTS): Estructura en 3 Bloques, Cero Rutas Locales y Nombres Base (Lección 55)
- **Estructura obligatoria del mensaje en 3 bloques**:
  1. **Bloque 1 (Al principio)**: Reporte visual de archivos actualizados en formato de árbol con texto a la mitad del tamaño estándar (`<small>...</small>`), ultra-compacto para pantallas móviles (iOS Safari) sin recargar la lectura. Dentro de `<small>...</small>` solo se escriben nombres cortos de carpeta y el **nombre base (`basename`)** de cada archivo, jamás rutas absolutas (`/Users/...`, `/home/...`, `C:\Users\...`) ni enlaces Markdown (`file:///...`).
  2. **Bloque 2 (En medio)**: Explicación concisa y natural para escucha activa en TTS, usando exclusivamente nombres base de archivo y agrupando por carpeta.
  3. **Bloque 3 (Al final)**: Exclusivamente la pregunta y el menú de decisiones en lista estrictamente numerada (`1.`, `2.`, `3.`...). Solo el menú final va numerado, porque ApliChat toma cualquier lista numerada del mensaje como opciones pulsables; el resto del mensaje va en frases.
- **Prohibición absoluta de rutas locales y enlaces `file://` o `/Users/...` en todo el mensaje**:
  - Queda terminantemente prohibido escribir rutas completas de sistema de archivos en el texto o en enlaces Markdown: ni `/Users/<nombre>/...`, ni `/home/...`, ni `C:\Users\...`, ni `[archivo.dart](file:///Users/...)`, ni `[archivo.dart](/Users/...)`.
  - **Prohibición de listas numeradas de archivos modificados con enlaces en el cuerpo**: Queda prohibido incluir en el cuerpo del mensaje listas numeradas de archivos modificados con rutas o enlaces locales (`1. [archivo.dart](/Users/...)`). Los archivos modificados se muestran **única y exclusivamente al inicio del mensaje** dentro del árbol visual `<small>...</small>` a mitad de tamaño y solo con su nombre base (`basename`).
- **Motivo de accesibilidad crítica y limpieza de citas**: Los motores de síntesis de voz (TTS) y lectores de pantalla leen cada barra («barra users barra apliarte...»), y al citar o replicar un párrafo en ApliChat las URLs `file:///Users/...` o `/Users/...` ensucian el compositor y rompen los botones de opciones numeradas (`1.`, `2.`, `3.`).
- **Regla del Nombre Base (Basename) sin Enlaces Locales**:
  - Tanto en el árbol `<small>...</small>` como en la prosa del cuerpo se escribe **única y exclusivamente el nombre base del archivo en texto plano** (`resumen.py`, `lecciones.md`, `main.dart`), **sin envolverlo en enlaces Markdown hacia rutas del disco local**.
  - Solo se permiten enlaces Markdown cuando apunten a recursos web (`https://...`) o rutas estáticas servidas por la Oficina (`/difusion/...`, `/adjuntos/...`, `/capturas/...`).
- **Agrupación por Módulo o Carpeta**:
  - Si en la explicación en prosa se mencionan varios archivos de un mismo directorio o proyecto, se nombra el contenedor una sola vez:
    Ejemplo: «En `herramientas/`: `worktree.py`, `resumen.py` y `vigilar.py`».
  - Jamás repetir la ruta completa una y otra vez en cada línea.

---

## 3. Disciplina de Memoria y Descarga Inmediata

### Descarga Cognitiva Inmediata
- Para evitar la saturación de la ventana de contexto y mantener una conversación limpia, queda prohibido retener reflexiones, dudas sueltas o ideas en la memoria volátil.
- Cada respuesta dirigida al usuario DEBE volcarse en tiempo real al buzón persistente del orquestador.

### Volcado Obligatorio al Buzón
Al finalizar cada turno de respuesta en terminal o Antigravity, el orquestador debe sincronizar el texto en ApliChat mediante:

```bash
dart run ${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}/bin/trabajo.dart tareas chat responder --agent or-<repo> "<texto_limpio>"
```

o mediante el endpoint HTTP:
`POST http://127.0.0.1:8791/api/chat?agent=or-<repo>`

---

## 4. Emisión de Propuestas a la Bandeja Visual (:8791/vista/propuestas)

Si durante la conversación surge una mejora transversal o lección aprendida que deba escalar a la central:
1. Volcar la propuesta al archivo `control-erbolamm/aprendizajes/propuestas.json`.
2. Notificar a Javier en ApliChat con opciones numeradas para su revisión en la Bandeja Visual de la Oficina (`http://127.0.0.1:8791/vista/propuestas`).

---

## 5. Soberanía Estricta y Prueba de Eliminación (Delete Test)

- El orquestador local responde únicamente de su repositorio soberano.
- Los archivos en `control-erbolamm/` (incluido este contrato) son transparentes al código fuente comercial del proyecto: si se borra la carpeta `control-erbolamm/`, el proyecto compila, ejecuta sus tests y funciona al 100% sin dependencias residuales.

