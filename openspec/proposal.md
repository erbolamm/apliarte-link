# Fase 1 — Protocolo Confiable

## Summary

Migrar el transporte de input entre peers de **UDP + JSON** a **TCP + TLS + MessagePack**, agregar handshake de versión, keep-alive, y canal de control separado.

## Motivation

- **UDP pierde paquetes**: un `MouseUp` perdido deja el botón pegado en el destino. Deskflow y Lan Mouse usan TCP por este motivo desde sus inicios.
- **JSON es verboso**: cada `MouseMove` son ~80 bytes en JSON vs ~9 en MessagePack/binario. Para input en tiempo real (cientos de eventos/segundo), la diferencia es significativa.
- **Sin cifrado**: los datos de input viajan en texto plano por la red local.
- **Sin detección de desconexión**: si un peer se cae, no se detecta hasta que expira el TTL de discovery (15s).

## Scope

| In scope | Out of scope |
|----------|--------------|
| TCP + TLS + MessagePack para input/control | Cambiar discovery (sigue en UDP+JSON) |
| Handshake de versión de protocolo | Modelo de pantalla / edge-switching |
| Keep-alive cada 3s | Clipboard sharing |
| Canal de control multiplexado | Mapeo de layouts de teclado |
| Pool de conexiones persistentes | Amigos por correo (Fase 2) |
| Compatibilidad hacia atrás con peers viejos | Linux / Wayland |

## Risks

- **Regresión**: romper el flujo actual macOS↔Windows que ya funciona con UDP
- **Complejidad TLS**: certificados autofirmados requieren manejo correcto en macOS (keychain) y Windows (cert store)
- **Latencia de conexión TCP**: al cambiar de peer activo, el handshake TCP+TLS agrega ~50-100ms

## Acceptance Criteria

1. Dos daemons se conectan por TCP en puerto 4242
2. La conexión va cifrada con TLS (certificados autofirmados)
3. Los eventos de input se serializan con MessagePack
4. El handshake de versión negocia la máxima versión común
5. Keep-alive detecta peers caídos en <10 segundos
6. `cargo build` compila sin errores en macOS
7. Los tests existentes siguen pasando
