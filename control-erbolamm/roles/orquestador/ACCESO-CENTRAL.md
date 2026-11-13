# Normativa de Acceso al Motor de la Oficina (erbolamm-trabajo)

Este documento fija la politica y garantias de acceso al motor de la oficina (repositorio central `erbolamm-trabajo`) para el Orquestador Local y los agentes de este repositorio.

## 1. Acceso Universal de Solo Lectura
- Cualquier Orquestador Local o trabajador de este repositorio tiene **acceso libre y completo en SOLO LECTURA** al motor de la oficina:
  `${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}`
- Proposito: Consultar en cualquier momento y sin restricciones:
  - `TECH_CATALOG.md`: versiones oficiales de dependencias, frameworks y upstreams.
  - `INBOX.md`: reglas de oro y prioridades globales del ecosistema.
  - `roles/orquestador/memoria/lecciones.md` (en central) y `memoria/lecciones.md` (en `asistente-apliarte`): lecciones aprendidas canónicas del ecosistema, para verificar y sincronizar al arranque de cada sesión o proyecto.
  - `categorias/`: recetas, ingredientes tecnicos y patrones arquitectonicos.
  - `skills/`: habilidades reutilizables y automatizaciones (especialmente `skills/desarrollo-doble-dispositivo/SKILL.md` para modo desarrollo y modo autónomo).
  - `control-erbolamm/dispositivos.json`: catálogo canónico de dispositivos físicos autorizados para movilidad y desarrollo (iPhone 15, iPhone 12, Pixel, Infinix Note 30 Pro).
  - `AGENTS.md`: protocolo canonico general y acuerdos del ecosistema.

## 2. Prohibición Absoluta de Modificación, Compilación y Soberanía Estricta Universal
- Queda **terminantemente prohibido** para cualquier rol o agente que opere en este repositorio escribir, editar, renombrar, borrar, compilar binarios (`flutter build`, `dart test`, `cargo build`, etc.) o comitear cualquier archivo en la central (`${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}`) o en cualquier otro repositorio satélite ajeno.
- **Soberanía estricta y universal entre repositorios**: Toda coordinación entre repositorios se realiza exclusivamente mediante prompts y tareas. Ningún orquestador o agente (incluyendo `or-erbolamm-trabajo` o cualquier `or-<proyecto>`) podrá editar directamente el código ni compilar binarios de otro repositorio satélite. Toda acción técnica se solicita al orquestador soberano del repositorio para que la ejecute localmente.
- **Emisión y escalado canónico de propuestas locales**:
  Si este orquestador local detecta en su entorno (`control-erbolamm/aprendizajes/`) una nueva lección, patrón técnico o mejora que deba compartirse con el ecosistema:
  1. La emite formalmente a `control-erbolamm/aprendizajes/propuestas.json`.
  2. La propuesta se refleja automáticamente en la **Bandeja Visual de Propuestas de la Oficina** (`http://127.0.0.1:8791/vista/propuestas`).
  3. `or-erbolamm-trabajo` es el único encargado de canonizarla para todos los proyectos del ecosistema (incorporándola de forma secuencial a `roles/orquestador/memoria/lecciones.md` en `erbolamm-trabajo`).

## 3. Protocolo Canonico de Pedido y Entrega de Herramientas (ADR-0013)
Este protocolo define como este repositorio consume herramientas, automatizaciones y servicios de la central sin comprometer su soberania ni ensuciar la raiz comercial del producto:

1. **Como pide el proyecto**:
   - Toda necesidad de una herramienta o servicio central (capturas de tienda, auditoria de resenas, notas de version, estudio de mercado) se canaliza de forma soberana a traves de `control-erbolamm/INBOX.md` o como tarjeta de tarea en `control-erbolamm/tareas/por-hacer/`.
   - Se invoca la herramienta correspondiente mediante el comando de la central apuntando a este repositorio (ej. `dart run ${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}/bin/trabajo.dart <herramienta> <proyecto>` o la skill correspondiente).
   - Queda prohibido crear carpetas intermedias arbitrarias (como `pedidos/` o `entregas/`): se utilizan exclusivamente los canales canonicos aprobados en el ADR-0013.

2. **Donde y como se entrega el resultado**:
   - Las herramientas de la central leen la configuracion del proyecto y escriben sus artefactos **estrictamente dentro de su destino canonico en `control-erbolamm/`**:
     - Material promocional, capturas y notas de version: `control-erbolamm/difusion/` (`screenshots/`, `notas-version.md`, etc.).
     - Especificaciones visuales y marca: `control-erbolamm/marca/` (`brand-spec.md`).
     - Respuestas de resenas o informes de mercado: se incorporan como informe con recibo formal en la tarea o en `control-erbolamm/INBOX.md`.
     - Mejoras, lecciones o patrones descubiertos para retroalimentar a la central: `control-erbolamm/aprendizajes/` (`propuestas.json`).
   - Toda ejecucion entrega un recibo formal con formato `TaskReceipt` (`### Estado`, `### Resultado`, `### Artefactos`, `### Bloqueos`), asegurando la trazabilidad de lo generado.

3. **Invariante de no intrusion en el codigo de producto**:
   - Ninguna herramienta central puede empujar archivos gestionados, configuraciones o scripts dentro de la raiz o el codigo fuente del producto (`lib/`, `src/`, `test/`).
   - El Delete Test (ADR-0013) debe cumplirse siempre: si se elimina `control-erbolamm/`, el producto comercial sigue compilando y pasando sus pruebas sin errores.

## 4. Uso Obligatorio de la Oficina ApliArte (:8791) y Vistas Web Interactivas

Para las fases de difusión, capturas, ASO, reseñas, propuestas y coordinación, el Orquestador Local NO debe resolver las tareas a ciegas en terminal mediante scripts manuales si existe soporte interactivo en la Oficina ApliArte (`http://127.0.0.1:8791`):

1. **Catálogo de Vistas Interactivas**:
   - **Bandeja de Propuestas y Aprendizajes**: `http://127.0.0.1:8791/vista/propuestas?proyecto=<proyecto>`. Permite a Javier revisar las propuestas emitidas en `control-erbolamm/aprendizajes/propuestas.json`, editarlas, descartarlas o canonizarlas en la Central con un solo clic.
   - **ApliChat (Buzón Visual Central)**: `http://127.0.0.1:8791/vista/aplichat`. Cuadro de diálogo visual y accesible donde el orquestador vuelca obligatoriamente sus respuestas y avances vía `tareas chat responder`.
   - **ASO y Metadatos de Tiendas (Fase 5)**: `http://127.0.0.1:8791/vista/tiendas?app=<proyecto>`.
     - *Filtro de Soberanía Estricta*: Exige `control-erbolamm/roles/orquestador/` en `$REPOS_ROOT/<proyecto>`.
     - *Artefactos Soberanos*: Persiste `palabras-clave.json`, `reporte-palabras-clave.md`, `recibo-aso.md` y metadatos/notas de versión en `control-erbolamm/difusion/tiendas/`.
     - *Comandos CLI*: `dart run ${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}/bin/trabajo.dart aso <proyecto>` y `dart run ${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}/bin/trabajo.dart tiendas <proyecto>`.
   - **Firebase y Estudios de Fallos (Crashlytics)**: `http://127.0.0.1:8791/vista/tiendas?app=<proyecto>#firebase`.
     - *Filtro de Soberanía Estricta*: Exige contenedor `control-erbolamm/roles/orquestador/` en el repositorio y aísla los fallos de cada app sin cruzarlos.
     - *Artefactos Soberanos*: Persiste y lee `control-erbolamm/firebase/crashlytics-estudios.json`, `firebase-audit.json`, `reporte-firebase.md` y `recibo-firebase.md`, e inyecta automáticamente en `control-erbolamm/INBOX.md` las tareas de corrección de fallos (`Abierto` / `Mitigado`) previas a la siguiente actualización.
     - *Comando CLI*: `dart run ${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}/bin/trabajo.dart firebase <proyecto>`.
   - **Capturas Retina y Mockups (Fase 6)**: `http://127.0.0.1:8791/vista/diseno?app=<proyecto>`. Permite visualizar, generar y validar píxel-perfect los mockups y capturas Retina (iPhone 6.5", 6.7", iPad 13") en lugar de inspeccionar archivos PNG crudos en terminal. Comando CLI: `dart run ${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}/bin/trabajo.dart capturas <proyecto>`.
     - *Protocolo de Capturas Base (RAW)*: Se depositan en `control-erbolamm/difusion/capturas/raw/`. Se priorizarán siempre las capturas descriptivas ya existentes en tiendas (App Store / Google Play). Si se captura en dispositivo físico, verificar que la sesión esté en modo Release o con el estado PRO/sin anuncios activo para evitar anuncios en los mockups.
   - **Gestión y Respuesta de Reseñas**: `http://127.0.0.1:8791/resenas?app=<proyecto>`.
     - *Filtro de Soberanía Estricta*: Solo aparecen en la Oficina y en la API aquellos proyectos que cuentan con su contenedor soberano `control-erbolamm/roles/orquestador/` inyectado en `$REPOS_ROOT/<proyecto>`.
     - *Artefactos Soberanos*: Lee y persiste `control-erbolamm/difusion/resenas/borradores.json`, `control-erbolamm/difusion/resenas/reglas.json`, `control-erbolamm/difusion/resenas/recibo-resenas.md` y vuelca sugerencias de usuarios en `control-erbolamm/INBOX.md`.
     - *Comando CLI*: `dart run ${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}/bin/trabajo.dart resenas proyecto <proyecto>`.
   - **Música Procedural (Strudel)**: `http://127.0.0.1:8791/vista/musica?app=<proyecto>`. Permite escuchar, ajustar y exportar la suite musical del proyecto a `control-erbolamm/difusion/musica/`. Comando CLI: `dart run ${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}/bin/trabajo.dart musica exportar <proyecto>`.
   - **Estudio de Mercado y Newsjacking**: `http://127.0.0.1:8791/vista/mercado?app=<proyecto>`.
     - *Artefactos Soberanos*: Persiste `tendencias.json`, `hooks-newsjacking.json`, `reporte-mercado.md` y `recibo-marketing.md` en `control-erbolamm/difusion/marketing/` y avisa en `control-erbolamm/INBOX.md`.
     - *Comando CLI*: `dart run ${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}/bin/trabajo.dart mercado exportar <proyecto>`.
   - **Locución y Doblaje**: `http://127.0.0.1:8791/vista/locucion?app=<proyecto>`. Generación de guiones multilingües y audio TTS para difusión.
   - **Facturas y Gestoría Trimestral (Uso Exclusivo de Javier Mateo)**: `http://127.0.0.1:8791/vista/facturas`.
     - *Prohibición Estricta para Orquestadores de Proyecto (`or-<proyecto>`)*: El módulo de Facturas y organización trimestral para gestoría (`/api/facturas/trimestre`, `/api/facturas/trimestre/csv`) es 100% personal y exclusivo de Javier Mateo (vinculado al workspace `javier` en Herdr). Cualquier petición desde un repositorio de app (`?app=`, `?proyecto=` o cabecera `x-erbolamm-actor: or-*`) es rechazada con `403 Forbidden`.
   - **Correos de Soporte de Usuarios (`erbolamm@gmail.com` y `javier@apliarte.com`)**: `http://127.0.0.1:8791/vista/correos?app=<proyecto>`.
     - *Artefactos Soberanos*: Persiste el buzón clasificado por proyecto en `control-erbolamm/correo/buzon-usuarios.json`, `control-erbolamm/correo/reporte-soporte.md`, suite de plantillas transaccionales en `control-erbolamm/correo/plantillas/` y sincroniza hilos pendientes en `control-erbolamm/INBOX.md`.
     - *API y CLI*: `GET`/`POST http://127.0.0.1:8791/api/apps/correos?app=<proyecto>` y `dart run ${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}/bin/trabajo.dart correo <proyecto>`.
   - **PNL — Programación Neurolingüística y Enganche de Usuarios**: `http://127.0.0.1:8791/vista/pnl?app=<proyecto>`.
     - *Artefactos Soberanos*: Detecta mensajes dentro de la app con riesgo de fallo o abandono por fricción cognitiva y prescribe mejoras PNL en `control-erbolamm/pnl/estudio-pnl.json`, `control-erbolamm/pnl/reporte-pnl.md` y `control-erbolamm/pnl/recibo-pnl.md`, inyectando tareas accionables en `control-erbolamm/INBOX.md`.
     - *API y CLI*: `GET`/`POST http://127.0.0.1:8791/api/apps/pnl?app=<proyecto>` y `dart run ${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}/bin/trabajo.dart pnl <proyecto>`.
   - **Estudio de SVG Animados (`skills/anatomy/`) y Banco de Trabajo de Skills**: `http://127.0.0.1:8791/vista/skills?app=<proyecto>`.
     - *Artefactos Soberanos*: Persiste las figuras e iconos SVG isométricos animados (`test-rig`, `dial-indicator`, `spot-plate`, `icono-isometrico-app`) en `control-erbolamm/marca/iconos-svg/{catalogo-svg-animados.json,<slug>.svg,<slug>.html,recibo-anatomy-svg.md}`, el registro de ejecuciones en `control-erbolamm/skills/ejecuciones-skills.json` y sincroniza tareas en `control-erbolamm/INBOX.md`.
     - *API y CLI*: `GET`/`POST http://127.0.0.1:8791/api/apps/skills?app=<proyecto>` y `dart run ${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}/bin/trabajo.dart anatomy-svg <proyecto>`.
   - **Estudio de Diseño y Suite Modular ArtCraft (8 Sub-pestañas de `reposDrive:/artcraft`)**: `http://127.0.0.1:8791/vista/diseno?app=<proyecto>`.
     - *Sub-pestañas Modulares*: `sora-video`, `midjourney-imagen`, `grok-multimedia`, `fal-generacion`, `worldlabs-3d`, `edicion-video`, `anatomy-svg` y `capturas-frames`.
     - *Artefactos Soberanos*: Persiste presets, prompts y activos de cada sub-pestaña en `control-erbolamm/difusion/diseno/<subpestana>/{estudio-<subpestana>.json,recibo-<subpestana>.md}`, actualiza `control-erbolamm/difusion/diseno/artcraft-catalogo.json` y sincroniza avisos en `control-erbolamm/INBOX.md`.
     - *API y CLI*: `GET`/`POST http://127.0.0.1:8791/api/apps/diseno?app=<proyecto>` y `dart run ${ERBOLAMM_CENTRAL:-$HOME/trabajo/erbolamm-trabajo}/bin/trabajo.dart artcraft <proyecto> [--subpestana <id>]`.
   - **Espacio Personal de Javier Mateo (`Workspace Herdr: javier`)**: `http://127.0.0.1:8791/vista/javier`.
     - *Propósito*: Centro de control personal de Javier con acceso directo a las 8 pestañas homologadas, listado de tarjetas `ja--` y estado de sesiones persistentes en `~/.erbolamm/javier/espacio-personal.json` (`GET`/`POST /api/javier/resumen`).
   - **Blogger y Difusión Multicanal con Capturas (`/vista/blogger`)**: `http://127.0.0.1:8791/vista/blogger`.
     - *Filtro y Destino*: Lee y persiste `difusion/borrador-activo.json` en central y `~/.erbolamm/blogger-assistant/candidate_draft.json`.
     - *Protocolo de Capturas y CDN*: Toda imagen local se optimiza previamente con `sips -Z 1280` (~200 KB) y se sube a CDN permanente (`freeimage.host` / `iili.io`) para obtener enlaces públicos directos antes de componer el HTML maquetado con identidad ApliArte.
     - *Despacho Desatendido*: Envío automático mediante `POST http://127.0.0.1:8791/api/difusion/blogger/enviar-n8n` con `actionToken` dinámico (`/api/tasks`) hacia `erbolamm.apliarte@blogger.com` vía n8n webhook, permitiendo a Javier aceptar y publicar el borrador desde el móvil sin intervención en navegador web.
     - *Comando y Skill*: `python3 /Users/apliarte/repos/asistente-apliarte/herramientas/publicar_blogger_con_capturas.py` y skill `difusion-blogger-capturas`.

2. **Disciplina Cognitiva de Descarga Inmediata**:
   - Queda prohibido acumular dudas, reflexiones o propuestas en la ventana de contexto del agente. Se descargan inmediatamente al buzón de ApliChat o a `control-erbolamm/aprendizajes/propuestas.json`.

3. **Apertura de la Oficina para Supervisión Humana y Protección del Directo**:
   - Cuando el Orquestador Local genere o audite cualquiera de estos artefactos, debe indicar siempre el enlace exacto (`http://127.0.0.1:8791/vista/<pantalla>?app=<proyecto>`) a Javier o al nodo soberano de máquina correspondiente (`Asistente Mac`, `Asistente Windows`, `Asistente Linux` o `Asistente VPS`).
   - **Prohibición estricta durante directos en curso**: Queda terminantemente prohibido ejecutar `/usr/bin/open`, abrir pestañas de Chrome o lanzar navegadores automatizados mientras el Centro de Directo esté emitiendo en Chrome, para no alterar el foco ni la ventana retransmitida.

## 5. Comprobacion de Desviacion y Repositorio Plantilla Oficial (ADR-0015)
- La fuente de verdad canónica de esta estructura `control-erbolamm/` y de los contratos de orquestador reside oficialmente en:
  [`https://github.com/erbolamm/orquestador-apliarte`](https://github.com/erbolamm/orquestador-apliarte)
- Para comprobar o sincronizar el repositorio soberano:
  `python3 ${ORQUESTADOR_HOME:-$HOME/repos/orquestador-apliarte}/bin/sync.py check .`
  `python3 ${ORQUESTADOR_HOME:-$HOME/repos/orquestador-apliarte}/bin/sync.py sync .`

