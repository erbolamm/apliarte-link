# 🚀 Pasos para Publicar — ApliArte Link

**Versión actual**: 0.2.7  
**Última actualización**: 2026-10-29  
**Mantenedor**: Javier Mateo (ApliArte / ErBolamm)  
**Licencia**: MIT — © 2026 ApliArte  

---

> ⚙️ **Modo agente / CI**: Este repositorio publica automáticamente sus releases oficiales multiplataforma en **GitHub Releases** mediante GitHub Actions al crear y subir un tag `vX.Y.Z`.
> Cumple con la normativa canónica de `INBOX.md` (Paso 3.5, Paso 3.10 y Paso 3.13) y la cuota gratuita de GitHub (cero almacenamiento intermedio de artifacts, subida directa al Release).

---

## 1️⃣ Publicación Automática vía GitHub Actions (Recomendada)

1. **Asegurar árbol limpio**:
   ```bash
   git status
   # Comprobar 0 cambios pendientes, 0 stashes y rama sincronizada
   ```

2. **Crear y subir el tag de versión**:
   ```bash
   git tag v0.2.8
   git push origin v0.2.8
   ```

3. **Seguimiento del pipeline**:
   El workflow `.github/workflows/release.yml` se encarga de:
   - Crear el Release oficial en estado borrador (draft).
   - Compilar el daemon Rust en matriz (macOS universal/arm64/x64, Windows x64, Linux x64).
   - Compilar las aplicaciones completas Flutter con el daemon embebido:
     - `apliarte-link-macos-app.zip` (macOS `.app` ad-hoc signed).
     - `apliarte-link-windows-app.zip` (Windows bundle con binario `.exe`).
     - `apliarte-link-linux-app.tar.gz` y paquete `.deb`.
   - Subir todos los instaladores directamente a **GitHub Releases** mediante `gh release upload --clobber` (sin consumir almacenamiento de artifacts de GitHub Actions).
   - Publicar el release retirando el estado draft.

---

## 2️⃣ Compilación y Validación Local

### A. Daemon Rust
```bash
cargo check --workspace
cargo test --workspace
cargo build --release
```

### B. Aplicación Flutter (macOS)
```bash
cd apps/apliarte_link
flutter pub get
flutter build macos --release
```

---

## 3️⃣ Canales de Distribución

- **Binarios e Instaladores**: [GitHub Releases](https://github.com/erbolamm/apliarte-link/releases) (macOS, Windows, Linux).
- **Código Fuente**: [github.com/erbolamm/apliarte-link](https://github.com/erbolamm/apliarte-link).
- **Landing Web**: [erbolamm.github.io/apliarte-link](https://erbolamm.github.io/apliarte-link/) (GitHub Pages).
