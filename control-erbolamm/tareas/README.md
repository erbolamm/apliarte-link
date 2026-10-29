# Normativa de Tareas Locales

Estructura de ciclo de vida de tareas y bitacora para este proyecto.

## Carpetas de Ciclo de Vida

- por-hacer/: Tareas aprobadas pendientes de inicio.
- haciendo/: Maximo una tarea activa en curso.
- hecho/: Tareas completadas con su bitacora de cambios documentada.
- futuras/: Tareas en planificacion o backlog a largo plazo.

## Reglas Obligatorias

1. Bitacora Progresiva: Cada tarea debe incluir la seccion "Bitacora de cambios" documentando los cambios paso a paso.
2. Nomenclatura: Archivos en minusculas separados por guiones con extension .md (ej. rol--proyecto--slug--creador--prioridad--fecha.md).
3. Higiene de Git: Verificar git status antes de iniciar y no mezclar cambios ajenos a la tarea.

## Plantilla Canónica Simplificada

```md
# Título de la tarea

- Hora: 04:35
- Destino: ruta o repositorio (opcional)
- Depende de: ninguna
- No tocar: fuera del objetivo descrito; secretos, publicación y Git remoto

## Objetivo

Una sola entrega comprobable.

## Comprobación

- Comando o evidencia esperada.

## Bitácora de cambios

- AAAA-MM-DD HH:MM (Responsable): Inicio del trabajo.

## Recibo

### Estado
Completo

### Resultado
Qué se hizo.

### Pruebas
Salida real pegada.

### Bloqueos
Ninguno
```
