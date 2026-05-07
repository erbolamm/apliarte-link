Documento generado para AI Studio de Google
Proyecto App Universal de Teclado/Ratón:
- Bluetooth: teclado, ratón, cualquier dispositivo físico compatible.
- Compatibilidad inicial: Linux, Mac, PC.
- Alcance local (red).
- La aplicación actúa como intermediaria convirtiéndose en teclado/ratón universal.
- Implementar switch para activar/desconectar dispositivos de su host original.
- Configuraciones mapeadas: para futuras iteraciones.
Preguntas pendientes:
1) ¿Registro y estado de dispositivos conectados?
2) ¿Switch automático o manual?
3) ¿Notificaciones emergentes para estados y conexión?
- Registro de dispositivos conectados: Sí.
- Gestión del switch: Pulsando una tecla (Control) o pasando el ratón por una esquina, dependiendo del dispositivo conectado. También podría haber una app maestra para cambiar configuraciones.
- Notificaciones emergentes: Muy importante incluir para avisos de estado, conexión y usos.
- Perfiles preconfigurados: Es posible.
- Interfaz visual: Acceso organizado con diferentes funciones agrupadas en pantallas separadas para evitar saturación. Diseño enfocado en arquitectura limpia.
- Seguridad: Requiere intervención directa del teclado/ratón conectado para emparejamiento o autenticación.
- Actualizaciones: El usuario elegirá cuándo actualizar.
- Estadísticas de uso: Podrían incluirse siempre que no sobrecarguen la app.
- Sistema de permisos: Implementar si se considera necesario para limitar accesos avanzados o funciones específicas.
- Activación: Funciona en segundo plano automáticamente.
- Dispositivos conectados: Ilimitados.
- Configuraciones: Exportables como JSON y posibilidad de guardarlas en Firebase.
- Accesos rápidos: Serían útiles para moverse fácilmente entre los dispositivos donde esté instalada la app (ordenador, móvil, etc.).
- Control por voz: Se deja como pendiente.
- Consumo de batería/recursos: Pregunta pendiente de reformular.
- Logs/Historial: Incluir para registrar acciones, especialmente útil en caso de error, y permitir enviarlos adjuntos por correo a info@apliarte.com.
- Personalización visual: Incluir temas (claro/oscuro).
- Soporte técnico/Guía: Incluir soporte técnico básico y guía rápida dentro de la app.
- Accesibilidad: Sí, incluir opciones como teclas más grandes o lectura de texto en voz.
- Notificaciones push: Sí, incluir alertas sobre dispositivos conectados/desconectados.
- Escenario prioritario: Mostrar claramente que la app es creada por ApliArte.com, incluyendo botones como Ko-fi, PayPal y enlaces a los proyectos actuales en GitHub.
- Multilenguaje: Soporte desde la primera versión para español e inglés.
- Respaldo/Restauración: Sí, incluir opción de respaldar y restaurar configuraciones automáticamente.
- Análisis de rendimiento: Sí, para medir velocidad de conexiones y consumo de recursos.


# Actualizaciones basadas en nuevas decisiones:

4. Consumo de recursos:
• Optimización de uso de batería y CPU pendiente de BLE y reconexión bajo demanda.
• Políticas de desconexión para dispositivos inactivos.

5. Perfiles preconfigurados:
• Perfiles localmente (JSON, almacenamiento simple).
• Migración futura a Firebase para backups y sincronización automática.

---

# Actualización — 2026-03-27 (Estado + Estudio)

## Avance en UI (Flutter)
- Implementada una vista de **“Buscar conexiones cercanas”** en `apps/apliarte_link/` accesible desde el botón de **Settings**.
- La vista realiza **escaneo BLE** con `flutter_blue_plus` y muestra resultados filtrados por **RSSI** como aproximación de “radio corto”.
- Se añadieron textos de uso en `apps/apliarte_link/macos/Runner/Info.plist` para que macOS entienda el motivo del acceso Bluetooth.

## Estudio / Observaciones (macOS)
- En algunos casos, el sistema **no solicita el prompt de permisos** si faltan *entitlements* de Bluetooth.
- Para corregirlo, se añadieron entitlements de Bluetooth en:
  - `apps/apliarte_link/macos/Runner/DebugProfile.entitlements`
  - `apps/apliarte_link/macos/Runner/Release.entitlements`

## Estado funcional actual
- El core Rust actual sigue enfocado en comunicación **por red local (LAN/UDP)** e inyección de input.
- La parte de “switch” basada en dispositivos Bluetooth (pairing/autenticación + enrutado del input) **todavía no está implementada** extremo a extremo.

# Actualización — 2026-05-06 (Roadmap MVP macOS + Windows)

## Decisión de foco
- El foco inmediato deja de ser Bluetooth, cuentas o extras.
- El objetivo pasa a ser una **app usable en macOS y Windows** lo antes posible.
- La definición mínima del MVP es: **dos equipos conectados por red local, con teclado y ratón funcionando juntos entre ambos mediante un cambio explícito de equipo**.

## Qué entra en este MVP
- App Flutter + daemon Rust arrancando en macOS.
- App Flutter + daemon Rust arrancando en Windows.
- Conexión local simple entre ambos equipos.
- Movimiento, clics, scroll y teclado esencial.
- Cambio de equipo por comando explícito configurable por el usuario.
- Teclado y ratón asignados siempre juntos en la primera versión.
- Confirmación visual clara en origen y destino cuando el control cambie de equipo.
- Permisos, estabilidad básica y empaquetado descargable.

## Qué no entra todavía
- Linux.
- Portapapeles compartido.
- Audio, archivos y extras de periféricos.
- Sistema de cuentas o pairing sofisticado.
- Bluetooth como eje principal del producto.
- Cambio automático por acercar el ratón al borde como flujo principal del MVP.

