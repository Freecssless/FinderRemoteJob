# FinderRemoteJob

Aplicación de escritorio (y, más adelante, móvil) para buscar empleo **remoto en México y LATAM**:

- Varios perfiles profesionales, cada uno con su CV maestro.
- Búsqueda agregada en portales con API o RSS oficial.
- Puntaje de compatibilidad entre CV y vacante.
- Optimización de CV para ATS (palabras clave, puntaje antes y después, exportación a PDF y DOCX) **sin inventar información**.
- Tablero de seguimiento de postulaciones.

El diseño completo está en [`docs/ARQUITECTURA.md`](docs/ARQUITECTURA.md) y el avance en [`CHANGELOG.md`](CHANGELOG.md).

## Estructura

| Carpeta | Paquete | Qué contiene |
|---|---|---|
| `crates/core` | `finder-core` | Dominio puro: modelos, ATS, compatibilidad, guardián. **Sin IO**; se reutiliza en móvil |
| `crates/infra` | `finder-infra` | SQLite, clientes LLM, fuentes de vacantes, PDF/DOCX |
| `src-tauri`, `src` | — | App Tauri 2 + Svelte (módulo 10, pendiente) |

## Requisitos

- Rust estable (1.85 o posterior, edición 2024): `rustup default stable`
- Para la app de escritorio (módulo 10), en Arch/CachyOS:
  ```bash
  sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file openssl \
    appmenu-gtk-module libappindicator-gtk3 librsvg xdotool nodejs npm
  ```
- Opcional: [Ollama](https://ollama.com) para la IA local (módulo 7).

## Compilar y probar

```bash
cargo build
cargo test
cargo clippy --all-targets
```

## Claves de API y configuración

Las claves **nunca** van en el código ni en el repositorio. Se leerán de variables de entorno o de un archivo `.env` en la raíz, que ya está excluido en `.gitignore`. Las variables exactas se definen en el módulo 7 (LLM) y se documentarán aquí.

## Fuentes de vacantes y atribución

Las vacantes automáticas vienen de Remote OK, Remotive, We Work Remotely, Get on Board y Jobicy. Sus términos piden mencionar la fuente y enlazar al anuncio original; la app guarda ambos datos en cada vacante. Además, respeta un tiempo mínimo entre consultas (Remotive: 6 horas).

## Portales sin API oficial

LinkedIn, Indeed, Glassdoor, OCC y Computrabajo **no** se consultan automáticamente: sus términos de servicio lo prohíben y hacerlo pone en riesgo tus cuentas. En el MVP solo se pueden importar a mano (pegar el enlace o el texto de la vacante), y la postulación siempre la confirmas tú.
