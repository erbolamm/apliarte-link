# ApliArte Link

> **KVM virtual por red local** — Comparte un teclado y ratón entre Mac y Windows sin cables.

🌐 **[apliarte-link.web.app](https://apliarte-link.web.app)**

[![Estado](https://img.shields.io/badge/estado-MVP%20macOS%20%2B%20Windows-orange)](#estado-actual)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-blue)](#plataformas)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## ¿Qué es?

ApliArte Link es un **KVM virtual por red local**. Instálalo en dos ordenadores de la misma red WiFi y contrólalos con un solo teclado y ratón.

En el MVP actual el cambio de control se hace con un **comando explícito configurable** desde la app. No depende del borde de pantalla por defecto y asigna **teclado y ratón juntos** al equipo activo.  
Sin cables. Sin dongles. Sin hardware extra.

```
[agenmac]  ←──── TCP+TLS 4242 ────→  [agenpc]
   \__________________________/
     un solo teclado y ratón
```

---

## Estado actual

| Plataforma | Motor (Rust) | Estado |
|---|---|---|
| macOS 12+ arm64 / x64 | ✅ Base implementada | 🟡 Foco MVP — app usable + ratón/teclado |
| Windows 10+ x64 | ✅ Base implementada | 🟡 Foco MVP — app usable + ratón/teclado |
| Linux x64 | ✅ Backend uinput/evdev | 🟡 En desarrollo — app Linux con daemon embebido |

> Los binarios descargables estarán disponibles en [apliarte-link.web.app](https://apliarte-link.web.app) en cuanto pase el primer test end-to-end.

> El foco inmediato no es meter extras. El foco es cerrar una primera app usable en macOS y Windows donde ya se pueda trabajar con ratón y teclado entre dos equipos.

## Roadmap

### Principios de diseño

1. **Privacidad por defecto** — solo equipos de amigos mutuos se ven entre sí.
2. **Confiabilidad sobre velocidad** — TCP con TLS antes que UDP crudo.
3. **Cambio explícito como base** — el edge-switching llega después, no reemplaza el comando.
4. **Referente deskflow** — 24 años de lecciones aprendidas por el KVM software más usado del mundo.

---

### Fase 0 — MVP actual (ahora)

> Lo que ya está andando o casi.

- [x] Daemon Rust con captura/inyección en macOS (CGEventTap) y Windows (SendInput)
- [x] Discovery UDP por broadcast local — los peers se encuentran solos en la LAN
- [x] App Flutter Desktop con UI básica (on/off, lista de peers, settings)
- [x] IPC entre Flutter y daemon vía TCP localhost
- [x] Cambio de control por comando explícito (triple clic derecho / triple Shift)
- [x] Confirmación visual en origen y destino al transferir el control
- [x] Empaquetado descargable en .zip (apps macOS/Windows/Linux en GitHub Releases)
- [ ] Test end-to-end real entre dos equipos físicos

**Stack:** Rust + Flutter · MessagePack sobre TCP+TLS 4242 · Discovery UDP 4244

---

### Fase 1 — Protocolo confiable 🔒

> Completada — el transporte entre peers es TCP+TLS+MessagePack desde la Fase 1.

**Por qué:** UDP pierde paquetes y un `MouseUp` perdido deja el botón pegado en el destino. TCP + TLS es lo que usan deskflow y Lan Mouse por este motivo exacto.

- [x] Transporte migrado de **UDP → TCP** en puerto 4242
- [x] **TLS** con certificados autofirmados por sesión (DTLS se descarta: TCP+TLS es más simple y probado)
- [x] Handshake de versión de protocolo (negocia la versión común entre peers, `[1,0]`)
- [x] **Keep-alive** cada 3s con timeout de 9s → detecta peers caídos al instante
- [x] Serialización de **JSON → MessagePack** (reducción ~8x en latencia y ancho de banda)
- [x] Canal de control separado del canal de input (comandos no compiten con eventos de ratón)

**Estado objetivo:** la conexión entre dos equipos no pierde eventos, va cifrada, y detecta desconexiones en <10 segundos.

---

### Fase 2 — Descubrimiento privado por amigos 👥

> El modelo que mencionaste: solo te ven equipos de amigos logueados.

**Por qué:** hoy cualquier equipo en tu red WiFi ve tu daemon y sabe que existe ApliArte Link. Con este modelo, tu ordenador es invisible salvo para las personas que vos autorizaste.

**Cómo funciona:**
1. Te registrás con tu correo electrónico (o Google/Apple OAuth).
2. Agregás amigos por correo. La relación es **mutua**: ambos tienen que aceptarse.
3. Cuando dos amigos están en la misma red local, un servidor de rendezvous (Firebase o Rust) les presenta las IPs locales.
4. Los daemons se conectan **directamente** por TCP+TLS local. Los datos nunca pasan por el servidor.
5. Si no hay amigos mutuos cerca, tu equipo no anuncia nada y no ve a nadie.

- [ ] Registro/login por correo + OAuth (Google/Apple)
- [ ] Lista de amigos con solicitud y aceptación mutua
- [ ] Servidor de rendezvous para emparejar IPs locales de amigos mutuos
- [ ] Eliminar broadcast abierto: el discovery solo responde a amigos autenticados
- [ ] UI de amigos en Flutter (agregar, aceptar, eliminar, ver estado)
- [ ] Modo "sin cuenta" offline para desarrollo y pruebas locales

**Estado objetivo:** abrís la app, ves solo los equipos de tus amigos que están cerca, y hacés clic para tomar el control. Nadie más te ve.

---

### Fase 3 — Calidad de input 🎯

> Pulir la experiencia de teclado y ratón para que sea indistinguible de tenerlos conectados físicamente.

- [ ] **Mapeo de layouts de teclado**: si un Mac español manda una `ñ`, un Windows US la recibe como `ñ`, no como un código roto. Deskflow tiene `KeyMap` + `KeyboardLayoutManager` — nosotros necesitamos al menos una tabla de equivalencia ES↔US↔LATAM.
- [ ] **Modelo de pantallas y edge-switching**: cada equipo declara la geometría de sus monitores. El usuario configura qué borde da a qué amigo ("a la derecha de mi pantalla está la Mac de Javier"). El cursor cruza el borde y aparece en el otro equipo.
- [ ] **Multi-monitor por máquina**: un equipo con 2 pantallas debe manejarlas como una superficie unificada antes de decidir bordes hacia peers.
- [ ] **Modifiers robustos**: Ctrl, Alt, Shift y Meta viajan correctamente incluso con combinaciones complejas.
- [ ] **Autorepeat de teclas**: mantener una tecla pulsada genera la cadencia correcta en el destino.

**Estado objetivo:** mover el ratón al borde derecho de tu pantalla y verlo aparecer en la pantalla del amigo, con el teclado siguiéndolo. Escribir en español en un Mac y que un Windows US lo entienda.

---

### Fase 4 — Funcionalidades extendidas 📋

> Lo que deskflow ya tiene y nuestros usuarios van a pedir.

- [ ] **Clipboard sharing**: copiás texto/imagen en un equipo y pegás en el otro. Deskflow dedica cientos de líneas a esto con converters por SO y formato (texto, HTML, bitmap, UTF-8/16). Lo atacamos con: texto plano → HTML → imágenes → archivos.
- [ ] **Arrastrar y soltar archivos** entre equipos (pequeños, por la red local)
- [ ] **Notificaciones push**: cuando un amigo se conecta/desconecta, o cuando te transfiere el control
- [ ] **Temas visuales** (claro/oscuro) y personalización de la UI
- [ ] **Logs e historial** de conexiones para diagnosticar problemas
- [ ] **Exportar/importar configuración** (JSON) para respaldo y migración

---

### Fase 5 — Multiplataforma completa 🌍

- [ ] **Linux** con soporte real (X11 + Wayland vía libei, como deskflow y Lan Mouse)
- [ ] **App móvil compañera** (iOS/Android) para ver estado, recibir notificaciones y activar/desactivar remotamente
- [ ] Binarios firmados y notarizados (Apple notarization, Windows Authenticode)
- [ ] Auto-update desde GitHub Releases
- [ ] Instalador one-click en las tres plataformas de escritorio

---

### Referencias técnicas

| Proyecto | Lenguaje | Año | Lo que nos enseña |
|----------|----------|-----|-------------------|
| [Deskflow](https://github.com/deskflow/deskflow) | C++ | 2001 | Protocolo binario, TCP+TLS, modelo de pantalla, keymap, clipboard, 25.8k ⭐ |
| [Lan Mouse](https://github.com/feschber/lan-mouse) | Rust | 2023 | Implementación Rust compatible con deskflow, crates input-capture/emulation, 4.8k ⭐ |
| [Input Leap](https://github.com/input-leap/input-leap) | C++ | 2022 | Fork de Barrier/Deskflow (inactivo) — referencia histórica |

> **Nota:** ApliArte Link no busca ser compatible con el protocolo de deskflow. Aprendemos de su arquitectura pero mantenemos nuestra propia identidad: P2P simétrico, privacidad por amigos, Flutter como UI.

## Topología de pruebas debug

Para las pruebas paralelas que vamos a hacer, usamos estos nombres operativos:

- `agenmac`: Mac.
- `agenpc`: Windows.
- `agenlinux`: Linux en modo experimental de validación.

Cuando compartas fallos o capturas, usa esos nombres y así distinguimos enseguida origen, destino y plataforma.

---

## Arquitectura

```
apliarte-link/
├── native/daemon/                  ← Servicio Rust (motor, corre en segundo plano)
│   └── src/
│       ├── input/
│       │   ├── macos.rs          CGEventTap — captura e inyección macOS
│       │   ├── windows.rs        SendInput — captura e inyección Windows
│       │   ├── linux.rs          uinput/libei — inyección Linux (esqueleto)
│       │   ├── keymap.rs         🔜 Mapeo de layouts ES↔US↔LATAM
│       │   ├── protocol.rs       InputEvent + handshake de versión
│       │   └── mod.rs            Orquestación SENDER/RECEIVER
│       ├── network/
│       │   ├── transport.rs      ✅ TCP+TLS+MessagePack + keep-alive (Ping 3s / timeout 9s)
│       │   ├── discovery.rs      ✅ UDP 4244 · 🔜 rendezvous autenticado (Fase 2)
│       ├── auth/                 🔜 Identidad, OAuth, lista de amigos
│       ├── screen/               🔜 Modelo de pantallas y edge-switching
│       ├── clipboard/            🔜 Portapapeles compartido
│       ├── ipc/mod.rs            TCP 127.0.0.1:4243 — comunicación con UI
│       └── config.rs             JSON en AppData / Library
├── apps/apliarte_link/              ← App Flutter Desktop (panel de control)
├── packages/apliarte_link_core/     ← Modelos y contratos compartidos UI↔IPC
├── web/                             ← Landing page (Firebase Hosting)
└── .github/workflows/
  └── release.yml                  ← Publica binarios y apps en GitHub Releases
```

**Core (Rust):** corre de fondo, captura input del SO, lo envía por TCP al equipo amigo y lo inyecta.  
**Modos:** cada instancia puede ser SENDER (captura y envía) o RECEIVER (recibe e inyecta), intercambiables al vuelo.  
**🔜** = planificado para la fase indicada en el roadmap.

### Flujo de conexión con amigos

```
Registro inicial:
  App Flutter  →  Servidor ApliArte (auth.apliarte.com)
  · Registro por correo + OAuth
  · Agregar amigos por correo
  · Aceptación mutua

Conexión diaria:
  Daemon Rust  →  Servidor rendezvous
  · "Estoy en IP 192.168.1.42, mis amigos son [A, B]"
  · Servidor responde: "A está en 192.168.1.30, misma red"
  Daemon Rust  →  TCP+TLS directo a 192.168.1.30:4242
  · Input nunca pasa por el servidor
  · Sin amigos cerca → no se anuncia nada
```

### Protocolo de red

```
┌──────────────────────────────────────────────────────┐
│  Fase 0 (ahora)                                      │
│  Input:       UDP 4242  · JSON                       │
│  Discovery:   UDP 4244  · broadcast abierto          │
│  UI↔Daemon:  TCP 127.0.0.1:4243  · JSON Lines       │
├──────────────────────────────────────────────────────┤
│  Fase 1 (próxima)                                    │
│  Input:       TCP 4242  · TLS  · MessagePack         │
│  Control:     TCP 4242  · TLS  · MessagePack         │
│  Keep-alive:  TCP 4242  cada 3s                      │
├──────────────────────────────────────────────────────┤
│  Fase 2 (descubrimiento privado)                     │
│  Discovery:   servidor rendezvous + LAN directa      │
│  Auth:        correo + OAuth + amigos mutuos         │
└──────────────────────────────────────────────────────┘
```

- **Fase 0:** JSON sobre UDP. Simple para prototipado pero frágil (paquetes perdidos, sin cifrado).
- **Fase 1:** TCP + TLS + MessagePack. El estándar de deskflow y Lan Mouse. Confiable y rápido.
- **Fase 2:** Descubrimiento autenticado. Solo los amigos mutuos en la misma red local se encuentran.

### Modelo de privacidad

ApliArte Link no expone tu equipo a toda la red local. Usa un modelo de **amigos mutuos**:

1. **Identidad:** te registrás con tu correo electrónico.
2. **Confianza:** agregás amigos por correo y ambos se aceptan mutuamente.
3. **Visibilidad:** solo los equipos de amigos mutuos que están en tu misma red local aparecen en tu lista.
4. **Conexión directa:** los datos de input viajan directo entre equipos por TCP+TLS local. Nunca pasan por un servidor externo.
5. **Sin cuenta (offline):** para desarrollo, pruebas o redes de confianza, se puede usar sin registro — el discovery por broadcast local sigue disponible.

Esto combina la baja latencia de una conexión LAN directa con la privacidad de un sistema de contactos: no cualquiera cerca tuyo puede verte.

### Cambio de control

- **MVP actual y Fase 1:** cambio explícito configurable.
  - teclado: **triple Shift derecho**
  - ratón: **triple clic derecho**
- **Fase 3:** se agrega **edge-switching** opcional: el cursor cruza el borde de pantalla y aparece en el equipo del amigo.
- Los dos modos conviven: comando explícito para el control consciente, borde de pantalla para flujo continuo.
- La UI muestra confirmación visual en origen y destino cuando cambia la asignación.

---

## Clonar el repositorio

### Mac

```bash
cd /Users/apliarte/trabajo
git clone https://github.com/erbolamm/apliarte-link.git apliarte-link
cd apliarte-link
```

### Windows (PowerShell)

```powershell
cd C:\trabajoPc
git clone https://github.com/erbolamm/apliarte-link.git apliarte-link
cd apliarte-link
```

> Tras clonar, trabaja sobre la rama que corresponda a tu cambio. `main` contiene el estado estable del repo y los releases publicados.

## Instalación rápida

Estas opciones dependen de que exista una release publicada.

### macOS y Linux

```bash
curl -fsSL https://raw.githubusercontent.com/erbolamm/apliarte-link/main/scripts/install.sh | bash
```

### Windows

```powershell
irm https://raw.githubusercontent.com/erbolamm/apliarte-link/main/scripts/install.ps1 | iex
```

Si todavía no hay release publicada, usa el flujo de desarrollo y debug desde código fuente del siguiente apartado.

---

## Desarrollo y debug desde código fuente

### Requisitos

- [Rust](https://rustup.rs/) >= 1.75
- En macOS: Xcode Command Line Tools (`xcode-select --install`)
- En Windows: Visual Studio Build Tools con MSVC

### Core

```bash
# Desde la raíz del proyecto (workspace Cargo)
cargo build --release

# El binario queda en:
# macOS/Linux:  target/release/apliarte-link
# Windows:      target\release\apliarte-link.exe
```

### UI (Flutter Desktop)

```bash
cd apps/apliarte_link
flutter pub get
flutter run -d macos     # Mac
flutter run -d windows   # Windows
flutter run -d linux     # Linux
```

La app Flutter intenta arrancar el daemon automáticamente. En bundle lo busca junto al ejecutable de Flutter y en desarrollo cae a `target/release/apliarte-link`.

Para nuestras pruebas simultáneas:

- `agenmac` ejecuta `flutter run -d macos`
- `agenpc` ejecuta `flutter run -d windows`
- `agenlinux` ejecuta `flutter run -d linux`

---

## Configuración

Al arrancar por primera vez se crea el archivo de configuración automáticamente:

- **macOS/Linux:** `~/Library/Application Support/apliarte-link/config.json`
- **Windows:** `%APPDATA%\apliarte-link\config.json`

Claves relevantes hoy:

- `network.port`: `4242`
- `network.encrypted`: `true`
- `input.edge_switch`: `false`
- `input.edge_side`: `right`
- `input.mouse_switch_command`: `triple_right_click`
- `input.keyboard_switch_command`: `triple_right_shift`

La app permite cambiar estos comandos desde la interfaz. Para las primeras pruebas manuales conviene usar un perfil más seguro, por ejemplo triple Escape y triple clic central.

---

## Publicar una nueva versión

Los binarios se compilan automáticamente con GitHub Actions al crear un tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Esto lanza 4 compilaciones de binarios crudos para macOS, Windows y Linux, además de las apps empaquetadas de macOS y Windows, y publica todo en la GitHub Release asociada al tag.

---

## Autor

Javier Mateo (ApliArte) — [apliarte.com](https://apliarte.com)

---

## 💬 Una nota personal / A personal note

<details>
<summary>🇪🇸 Español</summary>

ApliArte Link nació de una necesidad real: tengo varios ordenadores en mi escritorio y pasarme un teclado y un ratón físico de uno a otro siempre fue un engorro. Empecé a programar en abril de 2023 y este proyecto es parte de ese camino de aprendizaje. Lo comparto gratis porque creo que las herramientas útiles deben estar al alcance de todos. Si te ayuda en tu día a día, me hace muy feliz.
</details>

<details>
<summary>🇬🇧 English</summary>

ApliArte Link was born out of a real need: I have several computers on my desk and physically swapping a keyboard and mouse between them was always a hassle. I started learning to code in April 2023 and this project is part of that journey. I'm sharing it free because I believe useful tools should be accessible to everyone. If it helps you in your daily work, that makes me very happy.
</details>

<details>
<summary>🇧🇷 Português</summary>

O ApliArte Link nasceu de uma necessidade real: tenho vários computadores na minha mesa e trocar um teclado e mouse fisicamente entre eles sempre foi uma chatice. Comecei a programar em abril de 2023 e este projeto faz parte dessa jornada de aprendizado. Compartilho de graça porque acredito que ferramentas úteis devem estar ao alcance de todos. Se te ajudar no dia a dia, fico muito feliz.
</details>

<details>
<summary>🇫🇷 Français</summary>

ApliArte Link est né d'un besoin réel : j'ai plusieurs ordinateurs sur mon bureau et déplacer physiquement un clavier et une souris de l'un à l'autre a toujours été fastidieux. J'ai commencé à programmer en avril 2023 et ce projet fait partie de ce chemin d'apprentissage. Je le partage gratuitement parce que je crois que les outils utiles doivent être accessibles à tous. Si cela vous aide au quotidien, je suis très content.
</details>

<details>
<summary>🇩🇪 Deutsch</summary>

ApliArte Link entstand aus einem echten Bedürfnis: Ich habe mehrere Computer auf meinem Schreibtisch und das physische Hin-und-herreichen einer Tastatur und Maus war immer umständlich. Ich habe im April 2023 angefangen zu programmieren und dieses Projekt ist Teil dieses Lernwegs. Ich teile es kostenlos, weil ich glaube, dass nützliche Werkzeuge für alle zugänglich sein sollten. Wenn es dir im Alltag hilft, freut mich das sehr.
</details>

<details>
<summary>🇮🇹 Italiano</summary>

ApliArte Link è nato da un'esigenza reale: ho diversi computer sulla scrivania e spostare fisicamente una tastiera e un mouse da uno all'altro è sempre stato scomodo. Ho iniziato a programmare nell'aprile 2023 e questo progetto fa parte di quel percorso di apprendimento. Lo condivido gratuitamente perché credo che gli strumenti utili debbano essere accessibili a tutti. Se ti aiuta nel quotidiano, ne sono molto felice.
</details>

---

## Validación manual

La guía operativa para probar el MVP real entre equipos está en `docs/validacion-manual-mac-windows.md`.

Si vas a probar tres equipos a la vez, usa igualmente los alias `agenmac`, `agenpc` y `agenlinux` al reportar resultados.

## Contribuir

Este proyecto es open source y acepta contribuciones.
Abre un issue antes de cambios grandes para coordinar el alcance.

## 💥 Compártelo

Si este proyecto te ahorra cables, tiempo o dolores de cabeza, compártelo.

[𝕏 Twitter / X](https://twitter.com/intent/tweet?text=ApliArte Link%20%E2%80%94%20KVM%20virtual%20por%20red%20local%20para%20compartir%20teclado%20y%20rat%C3%B3n%20entre%20Mac%2C%20Windows%20y%20Linux.&url=https%3A%2F%2Fgithub.com%2Ferbolamm%2Fapliarte-link) · [💼 LinkedIn](https://www.linkedin.com/sharing/share-offsite/?url=https%3A%2F%2Fgithub.com%2Ferbolamm%2Fapliarte-link) · [💬 WhatsApp](https://api.whatsapp.com/send?text=ApliArte Link%20%E2%80%94%20KVM%20virtual%20por%20red%20local%3A%20https%3A%2F%2Fgithub.com%2Ferbolamm%2Fapliarte-link) · [✈️ Telegram](https://t.me/share/url?url=https%3A%2F%2Fgithub.com%2Ferbolamm%2Fapliarte-link&text=ApliArte Link%20%E2%80%94%20KVM%20virtual%20por%20red%20local%20para%20compartir%20teclado%20y%20rat%C3%B3n%20entre%20Mac%2C%20Windows%20y%20Linux.) · [🟠 Reddit](https://www.reddit.com/submit?url=https%3A%2F%2Fgithub.com%2Ferbolamm%2Fapliarte-link&title=ApliArte Link%20%E2%80%94%20KVM%20virtual%20por%20red%20local) · [🔵 Facebook](https://www.facebook.com/sharer/sharer.php?u=https%3A%2F%2Fgithub.com%2Ferbolamm%2Fapliarte-link) · [🧵 Threads](https://www.threads.net/intent/post?text=ApliArte Link%20%E2%80%94%20KVM%20virtual%20por%20red%20local.%20https%3A%2F%2Fgithub.com%2Ferbolamm%2Fapliarte-link) · [📧 Email](mailto:?subject=ApliArte Link%20%E2%80%94%20KVM%20virtual%20por%20red%20local&body=Te%20comparto%20ApliArte Link%3A%20KVM%20virtual%20por%20red%20local%20para%20compartir%20teclado%20y%20rat%C3%B3n%20entre%20Mac%2C%20Windows%20y%20Linux.%0A%0Ahttps%3A%2F%2Fgithub.com%2Ferbolamm%2Fapliarte-link)

## 💖 Apoya el proyecto

Herramienta gratuita y open source. Si te ahorra tiempo, un café ayuda a mantener el desarrollo.

| Plataforma | Enlace |
|-----------|--------|
| PayPal | [paypal.me/erbolamm](https://paypal.me/erbolamm) |
| Ko-fi | [ko-fi.com/C0C11TWR1K](https://ko-fi.com/C0C11TWR1K) |
| Twitch Tip | [streamelements.com/apliarte/tip](https://streamelements.com/apliarte/tip) |

🌐 [apliarte.com](https://apliarte.com) · 📦 [GitHub](https://github.com/erbolamm/apliarte-link)

## Licencia

MIT — © 2026 ApliArte
