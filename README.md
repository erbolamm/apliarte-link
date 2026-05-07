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
[agenmac]  ←──── UDP 4242 ────→  [agenpc]
   \__________________________/
     un solo teclado y ratón
```

---

## Estado actual

| Plataforma | Motor (Rust) | Estado |
|---|---|---|
| macOS 12+ arm64 / x64 | ✅ Base implementada | 🟡 Foco MVP — app usable + ratón/teclado |
| Windows 10+ x64 | ✅ Base implementada | 🟡 Foco MVP — app usable + ratón/teclado |
| Linux x64 | 🔵 Esqueleto | ⏸️ Después del MVP macOS/Windows |

> Los binarios descargables estarán disponibles en [apliarte-link.web.app](https://apliarte-link.web.app) en cuanto pase el primer test end-to-end.

> El foco inmediato no es meter extras. El foco es cerrar una primera app usable en macOS y Windows donde ya se pueda trabajar con ratón y teclado entre dos equipos.

## Roadmap inmediato

- Sacar una app de macOS que arranque bien, pida permisos y pueda usarse sin pasos raros.
- Sacar una app de Windows que arranque bien y pueda enlazarse con la de macOS.
- Cerrar el flujo básico entre ambos: mover ratón, clicks, scroll y teclado esencial.
- Hacer que el cambio de control sea explícito y configurable por comando, sin depender del borde de pantalla en la primera versión.
- Mantener teclado y ratón unidos en el MVP: el usuario asigna ambos a la vez al equipo activo.
- Mostrar una confirmación visual clara tanto en el equipo origen como en el destino cuando cambie la asignación.
- Resolver estabilidad del enlace local, selección de equipo activo y empaquetado descargable.
- Dejar para después del MVP: clipboard, DTLS, Linux, cuentas y extras de periféricos.

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
│       │   ├── windows.rs        Captura e inyección Windows
│       │   ├── linux.rs          uinput — inyección Linux (esqueleto)
│       │   ├── protocol.rs       InputEvent (MouseMove, KeyPress…)
│       │   └── mod.rs            Orquestación SENDER/RECEIVER
│       ├── network/mod.rs        UDP port 4242 — UdpSender + receiver
│       ├── ipc/mod.rs            TCP 127.0.0.1:4243 — comunicación con UI
│       └── config.rs             JSON en AppData / Library
├── apps/apliarte_link/              ← App Flutter Desktop (panel de configuración)
├── packages/apliarte_link_core/     ← Modelos y contratos compartidos entre UI e IPC
├── web/                             ← Landing page (Firebase Hosting)
└── .github/workflows/
  └── release.yml                  ← Publica binarios y apps empaquetadas en GitHub Releases
```

**Core (Rust):** corre de fondo, captura input del SO, lo envía por UDP al otro PC y lo inyecta.  
**Modos:** cada instancia puede ser SENDER (captura y envía) o RECEIVER (recibe e inyecta), según la configuración.

### Protocolo de red

- Input y control entre equipos: **UDP 4242**.
- Discovery y presencia de peers: **UDP 4244**.
- UI Flutter contra daemon local: **TCP 127.0.0.1:4243**.
- Serialización: **JSON**.

### Cambio de control actual

- MVP actual: **cambio explícito configurable**.
- Comandos por defecto:
  - teclado: **triple Shift derecho**
  - ratón: **triple clic derecho**
- El gesto por borde existe como opción, pero el flujo recomendado del MVP es con comando explícito.
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
