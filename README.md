# ApliArte Link

> **KVM virtual por red local** — Comparte un teclado y ratón entre Mac y Windows sin cables.

🌐 **[apliarte-link.web.app](https://apliarte-link.web.app)**

[![Estado](https://img.shields.io/badge/estado-MVP%20v0.1%20en%20desarrollo-orange)](#estado-actual)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-blue)](#plataformas)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## ¿Qué es?

ApliArte Link es un **KVM virtual por red local**. Instálalo en dos ordenadores de la misma red WiFi y contrólalos todos con un solo teclado y ratón.

Mueve el ratón hasta el borde de la pantalla → salta al siguiente PC.  
Sin cables. Sin dongles. Sin hardware extra.

```
[Tu Mac]  ←──── UDP 4242 ────→  [PC Windows]
     ☝️ Un solo teclado y ratón para los dos
```

---

## Estado actual

| Plataforma | Motor (Rust) | Estado |
|---|---|---|
| macOS 12+ arm64 / x64 | ✅ Implementado | 🟡 MVP — compilando, test pendiente |
| Windows 10+ x64 | ✅ Implementado | 🟡 MVP — compilando, test pendiente |
| Linux x64 | 🔵 Esqueleto | 🔵 Fase 2 — planificado |

> Los binarios descargables estarán disponibles en [apliarte-link.web.app](https://apliarte-link.web.app) en cuanto pase el primer test end-to-end.

---

## Arquitectura

```
apliarte-switch/
├── core/                  ← Servicio Rust (motor, corre en segundo plano)
│   └── src/
│       ├── input/
│       │   ├── macos.rs          CGEventTap — captura e inyección macOS
│       │   ├── capture_windows.rs WH_MOUSE_LL + WH_KEYBOARD_LL — captura Windows
│       │   ├── windows.rs        SendInput — inyección Windows
│       │   ├── linux.rs          uinput — inyección Linux (esqueleto)
│       │   ├── protocol.rs       InputEvent (MouseMove, KeyPress…)
│       │   └── mod.rs            Orquestación SENDER/RECEIVER
│       ├── network/mod.rs        UDP port 4242 — UdpSender + receiver
│       ├── ipc/mod.rs            TCP 127.0.0.1:4243 — comunicación con UI
│       └── config.rs             JSON en AppData / Library
├── ui/                    ← App Flutter Desktop (panel de configuración)
├── web/                   ← Landing page (Firebase Hosting)
└── .github/workflows/
    └── release.yml        Compila los 4 binarios y los sube a Firebase al crear un tag
```

**Core (Rust):** corre de fondo, captura input del SO, lo envía por UDP al otro PC y lo inyecta.  
**Modos:** cada instancia puede ser SENDER (captura y envía) o RECEIVER (recibe e inyecta), según la configuración.

### Protocolo de red

- Transporte: **UDP port 4242**
- Serialización: **JSON** (serde_json)
- Futuro: **DTLS** para tráfico cifrado

### Switch automático

- Detección de borde configurable (`edge_side`: `left` / `right`)
- Margen de 2px — cuando el cursor llega al borde se activa el switch
- Hotkey manual: **Ctrl+Alt+Tab** (configurable)

---

## Clonar el repositorio

### Mac

```bash
cd /Users/apliarte/trabajo
git clone https://github.com/erbolamm/apliarte-switch.git apliarte-switch
cd apliarte-switch
git checkout feat/core-mac-windows-mvp
```

### Windows (PowerShell)

```powershell
cd C:\trabajoPc
git clone https://github.com/erbolamm/apliarte-switch.git apliarte-switch-main
cd apliarte-switch-main
git checkout feat/core-mac-windows-mvp
```

> Tras clonar, la rama activa de desarrollo es `feat/core-mac-windows-mvp`. La rama `main` es la web y los releases.

---

## Compilar desde código fuente

### Requisitos

- [Rust](https://rustup.rs/) >= 1.75
- En macOS: Xcode Command Line Tools (`xcode-select --install`)
- En Windows: Visual Studio Build Tools con MSVC

### Core

```bash
# Desde la raíz del proyecto (workspace Cargo)
cargo build --release

# El binario queda en:
# macOS/Linux:  target/release/apliarte-switch
# Windows:      target\release\apliarte-switch.exe
```

### UI (Flutter Desktop)

```bash
cd ui
flutter pub get
flutter run -d macos     # Mac
flutter run -d windows   # Windows
flutter run -d linux     # Linux
```

---

## Configuración

Al arrancar por primera vez se crea el archivo de configuración automáticamente:

- **macOS/Linux:** `~/Library/Application Support/apliarte-switch/config.json`
- **Windows:** `%APPDATA%\apliarte-switch\config.json`

```json
{
  "network": {
    "port": 4242,
    "encrypted": false,
    "peer_ip": "192.168.1.X"
  },
  "input": {
    "hotkey": "ctrl+alt+tab",
    "edge_switch": true,
    "edge_side": "right"
  },
  "ui": {
    "theme": "dark",
    "language": "es"
  }
}
```

> `peer_ip` es la IP del otro PC en la red local. El PC con `edge_side: "right"` envía cuando el ratón llega al borde derecho. El otro pone `"left"`.

---

## Publicar una nueva versión

Los binarios se compilan automáticamente con GitHub Actions al crear un tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Esto lanza 4 compilaciones en paralelo (macOS arm64, macOS x64, Windows, Linux) y sube los binarios a [apliarte-link.web.app/downloads](https://apliarte-link.web.app).

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

## ¿Qué es ApliArteSwitch?

ApliArteSwitch es un **KVM virtual por red local**. Instálalo en dos o más ordenadores de la misma red WiFi y contrólalos todos con un solo teclado y ratón.

Mueve el ratón hasta el borde de la pantalla → saltas al siguiente PC.  
Sin cables. Sin dongles. Sin hardware extra.

```
[Tu Mac]  ←→  [PC Windows]  ←→  [Portátil Linux]
     ☝️ Un solo teclado y ratón para los tres
```

---

## ✨ Características

- 🖱️ **Switch automático** al mover el ratón al borde de la pantalla
- ⌨️ **Hotkey manual** para cambiar de PC (Ctrl+Alt+Tab por defecto)
- 📋 **Portapapeles compartido** *(próximamente)*
- 🔒 **Tráfico encriptado** (DTLS)
- 🌐 **Descubrimiento automático** en la red local (mDNS)
- 🎨 **App de configuración** con tema claro/oscuro
- 📦 **Perfiles exportables** en JSON

---

## 🖥️ Plataformas soportadas

| Plataforma | Core (motor) | UI Flutter | Estado |
|---|---|---|---|
| macOS 12+ | ✅ | ✅ | 🟡 Fase 1 — En desarrollo |
| Windows 10+ | ✅ | ✅ | 🔵 Fase 3 — Planificado |
| Linux (X11/Wayland) | ✅ | ✅ | 🔵 Fase 3 — Planificado |

---

## 📦 Instalación

### macOS

```bash
# Opción 1 — Script automático (recomendado, descarga binario compilado)
curl -fsSL https://raw.githubusercontent.com/erbolamm/apliarte-switch/main/scripts/install.sh | bash

# Opción 2 — Con Cargo (si tienes Rust instalado)
cargo install apliarte-switch

# Opción 3 — Homebrew (próximamente)
# brew install erbolamm/tap/apliarte-switch
```

### Linux

```bash
# Opción 1 — Script automático
curl -fsSL https://raw.githubusercontent.com/erbolamm/apliarte-switch/main/scripts/install.sh | bash

# Opción 2 — Con Cargo
cargo install apliarte-switch

# Permiso para /dev/uinput (necesario una sola vez)
sudo usermod -a -G input $USER
# Cierra sesión y vuelve a entrar para que el grupo se aplique
```

### Windows

```powershell
# PowerShell — Script automático (descarga binario .exe)
irm https://raw.githubusercontent.com/erbolamm/apliarte-switch/main/scripts/install.ps1 | iex

# O con Cargo si tienes Rust instalado
cargo install apliarte-switch
```

---

## 🚀 Uso rápido

```bash
# Arranca el servicio en segundo plano
apliarte-switch

# Arranca con logs visibles (útil para depurar)
apliarte-switch --verbose

# Ver ayuda
apliarte-switch --help
```

Después abre la **app de configuración** (carpeta `ui/`) para añadir los otros PCs.

---

## 🏗️ Arquitectura

El proyecto tiene dos partes separadas:

```
apliarte-switch/
├── core/          ← Servicio Rust (motor, corre en segundo plano)
│   ├── src/input/     Captura e inyección de teclado/ratón
│   ├── src/network/   Comunicación UDP entre PCs
│   ├── src/ipc/       Comunicación con la UI Flutter
│   └── src/config/    Configuración y perfiles
└── ui/            ← App Flutter Desktop (panel de control)
```

**Core (Rust)**: Corre de fondo, captura input, lo envía por red, lo inyecta.  
**UI (Flutter)**: Solo para configurar. Se conecta al Core por socket local.

---

## 🔨 Compilar desde código fuente

### Requisitos
- [Rust](https://rustup.rs/) >= 1.75
- [Flutter](https://flutter.dev/docs/get-started/install) >= 3.22 (para la UI)

### Core (motor Rust)
```bash
cd core
cargo build --release
# El binario queda en: target/release/apliarte-switch
```

### UI (Flutter Desktop)
```bash
cd ui
flutter pub get
flutter run -d macos     # Mac
flutter run -d windows   # Windows
flutter run -d linux     # Linux
```

---

## 🤝 Contribuir

Este proyecto es open source y acepta contribuciones.  
Abre un Issue antes de grandes cambios para coordinar.

---

## Autor
Javier Mateo (ApliArte) — github.com/erbolamm

## 💬 Una nota personal del autor / A personal note from the author
ℹ️ Nota: El texto siguiente es un mensaje personal del autor, escrito en varios idiomas para que pueda leerlo gente de todo el mundo. Esto no implica que el proyecto tenga soporte funcional completo en esos idiomas.

ℹ️ Note: The text below is a personal message from the author, written in several languages so people around the world can read it. This does not imply full multilingual feature support in those languages.

<details>
<summary>🇪🇸 Español</summary>

ApliArteSwitch nació de una necesidad real: tengo varios ordenadores en mi escritorio y pasarme un teclado y un ratón físico de uno a otro siempre fue un engorro. Empecé a programar en abril de 2023 y este proyecto es parte de ese camino de aprendizaje. Lo comparto gratis y como código abierto porque creo que las herramientas útiles deben estar al alcance de todos. Si te ayuda en tu día a día, me hace muy feliz.
</details>

<details>
<summary>🇬🇧 English</summary>

ApliArteSwitch was born out of a real need: I have several computers on my desk and physically swapping a keyboard and mouse between them was always a hassle. I started learning to code in April 2023 and this project is part of that journey. I'm sharing it free and open source because I believe useful tools should be accessible to everyone. If it helps you in your daily work, that makes me very happy.
</details>

<details>
<summary>🇧🇷 Português</summary>

O ApliArteSwitch nasceu de uma necessidade real: tenho vários computadores na minha mesa e trocar um teclado e mouse fisicamente entre eles sempre foi uma chatice. Comecei a programar em abril de 2023 e este projeto faz parte dessa jornada de aprendizado. Compartilho de graça e como código aberto porque acredito que ferramentas úteis devem estar ao alcance de todos. Se te ajudar no dia a dia, fico muito feliz.
</details>

<details>
<summary>🇫🇷 Français</summary>

ApliArteSwitch est né d'un besoin réel : j'ai plusieurs ordinateurs sur mon bureau et déplacer physiquement un clavier et une souris de l'un à l'autre a toujours été fastidieux. J'ai commencé à programmer en avril 2023 et ce projet fait partie de ce chemin d'apprentissage. Je le partage gratuitement et en open source parce que je crois que les outils utiles doivent être accessibles à tous. Si cela vous aide au quotidien, je suis très content.
</details>

<details>
<summary>🇩🇪 Deutsch</summary>

ApliArteSwitch entstand aus einem echten Bedürfnis: Ich habe mehrere Computer auf meinem Schreibtisch und das physische Hin-und-herreichen einer Tastatur und Maus war immer umständlich. Ich habe im April 2023 angefangen zu programmieren und dieses Projekt ist Teil dieses Lernwegs. Ich teile es kostenlos und als Open Source, weil ich glaube, dass nützliche Werkzeuge für alle zugänglich sein sollten. Wenn es dir im Alltag hilft, freut mich das sehr.
</details>

<details>
<summary>🇮🇹 Italiano</summary>

ApliArteSwitch è nato da un'esigenza reale: ho diversi computer sulla scrivania e spostare fisicamente una tastiera e un mouse da uno all'altro è sempre stato scomodo. Ho iniziato a programmare nell'aprile 2023 e questo progetto fa parte di quel percorso di apprendimento. Lo condivido gratuitamente e come open source perché credo che gli strumenti utili debbano essere accessibili a tutti. Se ti aiuta nel quotidiano, ne sono molto felice.
</details>

## 💥 Compártelo. Que se entere todo el mundo.
Si este proyecto te ahorra cables, tiempo o dolores de cabeza, compártelo. Así llega a más gente.

[𝕏 Twitter / X](https://twitter.com/intent/tweet?text=ApliArteSwitch%20%E2%80%94%20KVM%20virtual%20por%20red%20local%20para%20compartir%20teclado%20y%20rat%C3%B3n%20entre%20Mac%2C%20Windows%20y%20Linux.&url=https%3A%2F%2Fgithub.com%2Ferbolamm%2Fapliarte-switch) · [💼 LinkedIn](https://www.linkedin.com/sharing/share-offsite/?url=https%3A%2F%2Fgithub.com%2Ferbolamm%2Fapliarte-switch) · [💬 WhatsApp](https://api.whatsapp.com/send?text=ApliArteSwitch%20%E2%80%94%20KVM%20virtual%20por%20red%20local%3A%20https%3A%2F%2Fgithub.com%2Ferbolamm%2Fapliarte-switch) · [✈️ Telegram](https://t.me/share/url?url=https%3A%2F%2Fgithub.com%2Ferbolamm%2Fapliarte-switch&text=ApliArteSwitch%20%E2%80%94%20KVM%20virtual%20por%20red%20local%20para%20compartir%20teclado%20y%20rat%C3%B3n%20entre%20Mac%2C%20Windows%20y%20Linux.) · [🟠 Reddit](https://www.reddit.com/submit?url=https%3A%2F%2Fgithub.com%2Ferbolamm%2Fapliarte-switch&title=ApliArteSwitch%20%E2%80%94%20KVM%20virtual%20por%20red%20local) · [🔵 Facebook](https://www.facebook.com/sharer/sharer.php?u=https%3A%2F%2Fgithub.com%2Ferbolamm%2Fapliarte-switch) · [🧵 Threads](https://www.threads.net/intent/post?text=ApliArteSwitch%20%E2%80%94%20KVM%20virtual%20por%20red%20local.%20https%3A%2F%2Fgithub.com%2Ferbolamm%2Fapliarte-switch) · [📧 Email](mailto:?subject=ApliArteSwitch%20%E2%80%94%20KVM%20virtual%20por%20red%20local&body=Te%20comparto%20ApliArteSwitch%3A%20KVM%20virtual%20por%20red%20local%20para%20compartir%20teclado%20y%20rat%C3%B3n%20entre%20Mac%2C%20Windows%20y%20Linux.%0A%0Ahttps%3A%2F%2Fgithub.com%2Ferbolamm%2Fapliarte-switch)

## 💖 Apoya el proyecto
Herramienta gratuita y open source. Si te ahorra tiempo, un café ayuda a mantener el desarrollo.

| Plataforma | Enlace |
|-----------|--------|
| PayPal | [paypal.me/erbolamm](https://paypal.me/erbolamm) |
| Ko-fi | [ko-fi.com/C0C11TWR1K](https://ko-fi.com/C0C11TWR1K) |
| Twitch Tip | [streamelements.com/apliarte/tip](https://streamelements.com/apliarte/tip) |

🌐 [apliarte.com](https://apliarte.com) · 📦 [GitHub](https://github.com/erbolamm/apliarte-switch)

## Licencia
MIT — © 2026 ApliArte

## About
KVM virtual por red local — comparte teclado y ratón entre Mac, Windows y Linux con un solo comando de instalación.
