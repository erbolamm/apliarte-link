# Validación manual Mac + Windows

Esta guía sirve para validar el flujo real de ApliArte Link entre un Mac y un PC Windows sin compilar nada durante la prueba.

Objetivo de esta validación:

- Confirmar que la app Flutter arranca el daemon en ambos equipos.
- Confirmar discovery y conexión de peers.
- Confirmar cambio explícito de control.
- Confirmar captura local en origen y recepción/inyección en destino.
- Confirmar retorno del control al equipo local.

## Alcance real a día de hoy

Lo que esta prueba sí debe validar:

- Estado del motor en la UI.
- Visibilidad del otro equipo en la red local.
- Cambio explícito de teclado y ratón juntos.
- Movimiento de ratón, clicks, scroll y teclado básico entre Mac y Windows.
- Aviso visual en origen y destino.

Lo que sigue siendo riesgo conocido:

- En macOS siguen quedando huecos en scroll y movimiento absoluto dentro del backend de inyección.
- Linux no entra en esta validación.
- La prueba debe hacerse con los binarios empaquetados o con una ejecución local ya preparada; esta guía no cubre build.

## Preparación

## Alias para pruebas paralelas

Para que los reportes no se mezclen cuando probemos varios equipos a la vez:

- `agenmac`: Mac.
- `agenpc`: Windows.
- `agenlinux`: Linux.

En esta guía principal el flujo detallado sigue centrado en Mac + Windows, pero usaremos esos alias también cuando sumemos Linux a la ronda de debug.

Antes de empezar:

- Los dos equipos deben estar en la misma red local.
- Abrí ApliArte Link en los dos equipos.
- Dejá una app de texto abierta en el destino, por ejemplo Bloc de notas en Windows.
- Dejá una superficie clara para probar ratón y scroll, por ejemplo un navegador con una página larga.

Configuración de prueba recomendada para evitar activaciones accidentales:

- Comando teclado: triple Escape.
- Comando ratón: triple clic central.
- Switch por borde: desactivado.

Después de validar ese perfil seguro, repetí una pasada corta con el comando que querés usar de verdad, por ejemplo triple Shift derecho.

## Permisos en macOS

En macOS no alcanza con que la app abra. El proceso que captura input global necesita permisos del sistema.

Revisá en Ajustes del Sistema > Privacidad y seguridad:

- Accesibilidad:
  - Activado para ApliArte Link.
  - Si aparece el helper o binario apliarte-link por separado, activarlo también.
- Monitorización de entrada:
  - Activado para ApliArte Link.
  - Si aparece apliarte-link por separado, activarlo también.
- Red local o firewall:
  - Si macOS pide permitir conexiones entrantes para la app o el daemon, aceptar.
  - Si el firewall está activo y no hubo prompt, revisar manualmente que no esté bloqueando la app o el binario helper.

Señal de problema típica en Mac:

- La UI muestra el peer y hasta cambia el estado, pero el teclado/ratón siguen actuando localmente y no llegan al otro equipo.

Eso casi siempre apunta a permisos de captura global, no a la UI.

## Permisos en Windows

En Windows normalmente no hace falta permiso de privacidad específico para hooks globales, pero sí hay dos puntos prácticos:

- Firewall de Windows:
  - Si aparece un aviso para permitir la app o el daemon en red privada, aceptar.
  - Si no aparece el peer remoto, revisar reglas para la app Flutter y para apliarte-link.exe.
- Elevación:
  - Si la captura global no se comporta bien en una máquina concreta, hacer una pasada comparativa ejecutando la app como administrador para descartar limitaciones del entorno.

## Protocolo de prueba

### 1. Verificar arranque del motor

En ambos equipos:

- Abrir la app.
- Confirmar que la insignia de la UI dice Motor conectado.

Resultado esperado:

- Ambos lados muestran Motor conectado.

Si falla:

- El daemon no arrancó o la UI no pudo conectar al IPC local en 127.0.0.1:4243.

### 2. Verificar discovery de peers

En ambos equipos:

- Esperar unos segundos.
- Comprobar que el otro equipo aparece como peer conectado.

Resultado esperado:

- El Mac ve el Windows.
- El Windows ve el Mac.

Si falla:

- Revisar firewall.
- Revisar que ambos estén en la misma LAN.
- Probar añadir peer manualmente por IP desde la UI.

### 3. Verificar asignación explícita de control

Desde el equipo origen, por ejemplo el Mac:

- Tocar la tarjeta del Windows.
- Confirmar la acción de conectar.

Resultado esperado:

- En el origen aparece un aviso de que teclado y ratón fueron asignados.
- En el destino aparece un aviso de que está recibiendo control desde el origen.

Si falla:

- Si el peer aparece pero no cambia el estado, el problema está en IPC o en el estado del daemon.

### 4. Verificar ratón remoto

Con el Windows ya activo como destino:

- Mover el ratón físicamente en el Mac.
- Probar clic izquierdo.
- Probar clic derecho.
- Probar scroll vertical.

Resultado esperado:

- El cursor y los clicks afectan al Windows.
- El Mac no debería seguir actuando localmente con esos mismos eventos mientras el control remoto esté activo.

Si falla:

- Si hay aviso visual pero no se mueve nada en destino: problema de captura local o de envío UDP 4242.
- Si el destino recibe algo pero el origen también actúa localmente: el hook está capturando pero no está bloqueando el evento original.
- Si movimiento y clicks funcionan pero scroll no: el hueco está en el backend de inyección, especialmente probable en macOS.

### 5. Verificar teclado remoto

Con el foco puesto en una caja de texto del Windows:

- Escribir texto simple.
- Probar Backspace.
- Probar Enter.
- Probar mayúsculas.

Resultado esperado:

- El texto aparece en Windows.
- No se escribe en el Mac a la vez.

Si falla:

- Si el ratón funciona pero teclado no, revisar de nuevo Monitorización de entrada en macOS.
- Si solo falla el comando triple Shift derecho pero Escape funciona, el problema está en el tratamiento de modificadores y no en el pipeline completo.

### 6. Verificar retorno del control

Con el Windows todavía activo:

- Ejecutar el comando explícito de retorno o de cambio desde el equipo origen.
- Alternativa de respaldo: usar el botón Volver aquí si todavía podés interactuar con la UI local.

Resultado esperado:

- El control vuelve al equipo local.
- El destino deja de reaccionar al input del origen.
- Aparece aviso visual de liberación en destino y de control local recuperado en origen.

### 7. Repetir en sentido inverso

Hacer la misma secuencia al revés:

- Windows como origen.
- Mac como destino.

Esto es obligatorio porque Mac y Windows no capturan igual y el fallo puede ser asimétrico.

## Criterio de aceptación mínimo

La prueba puede darse por válida para el MVP solo si se cumple todo esto:

- Ambas apps muestran Motor conectado.
- Ambos equipos se descubren entre sí o al menos quedan conectables de forma estable.
- El cambio explícito asigna teclado y ratón juntos al destino.
- Hay aviso visual en origen y destino.
- Ratón, clicks, scroll y teclado básico viajan del origen al destino.
- El origen deja de actuar localmente mientras el destino está activo.
- El retorno del control funciona.
- La misma prueba pasa en ambos sentidos: Mac a Windows y Windows a Mac.

## Fallos y lectura rápida

Mapa rápido de síntomas:

- Peer no aparece en ninguno de los dos lados:
  - Discovery o firewall.
- La UI conecta al motor pero no aparece el otro equipo:
  - Discovery UDP 4244 o bloqueo de red local.
- El cambio de peer ocurre en la UI pero no viaja input:
  - Captura local o envío UDP 4242.
- El destino muestra aviso de control recibido pero no responde al ratón/teclado:
  - Inyección en destino o paquetes descartados.
- El origen y el destino reaccionan a la vez:
  - El hook captura pero no consume el evento local.
- El teclado remoto funciona con Escape pero no con Shift derecho:
  - Tratamiento de modificadores en el origen.
- Solo falla Mac como origen:
  - Permisos de Accesibilidad o Monitorización de entrada.
- Solo falla Mac como destino:
  - Backend de inyección macOS.

## Registro de resultados

Conviene anotar por cada pasada:

- Origen.
- Destino.
- Comando usado.
- Movimiento ratón: OK o FAIL.
- Clicks: OK o FAIL.
- Scroll: OK o FAIL.
- Teclado: OK o FAIL.
- Aviso visual origen: OK o FAIL.
- Aviso visual destino: OK o FAIL.
- Retorno del control: OK o FAIL.
- Observaciones.

Con eso ya podés distinguir si el problema es de captura, red, inyección o permisos sin volver a empezar a ciegas.