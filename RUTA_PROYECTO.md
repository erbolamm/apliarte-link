# Ruta Del Proyecto — ApliArte Link

Este archivo existe para evitar confusiones durante la transición de nombre del repositorio.

## Estado Actual

Ruta local actual:

```text
/home/javier/Documentos/repos/apliarte-link
```

La carpeta raíz ya usa el nombre nuevo `apliarte-link`.

Slug remoto actual en GitHub:

```text
erbolamm/apliarte-link
```

El repositorio remoto ya usa el slug nuevo. Los enlaces operativos de GitHub, releases y scripts deben apuntar a ese slug.

## Nombre Objetivo

Nombre definitivo del proyecto:

```text
ApliArte Link
```

ID de proyecto Firebase:

```text
apliarte-link
```

Ruta local recomendada cuando se haga el cambio controlado:

```text
/home/javier/Documentos/repos/apliarte-link
```

## Identidad Técnica Objetivo

```text
Flutter package/app: apliarte_link
Bundle ID Apple: com.erbolamm.apliarteLink
Nombre público: ApliArte Link
Firebase macOS nickname: ApliArte Link (macOS)
Firebase Windows/Web nickname: ApliArte Link (Windows/Web)
```

## Arquitectura Objetivo

```text
apps/apliarte_link/              # App Flutter principal
packages/apliarte_link_core/     # Contratos/modelos Dart: IPC, peers, comandos, status
native/daemon/                   # Daemon Rust desktop
web/                             # Web pública
```

## Decisión

La transición es de corte limpio: no se migrará configuración vieja del nombre anterior.

El rename de la carpeta raíz local ya está aplicado. Si algún terminal, script o configuración local sigue apuntando al path anterior, hay que actualizar esa referencia manualmente.
