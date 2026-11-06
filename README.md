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
| `src-tauri` | `finder-app` | App de escritorio Tauri 2: comandos delgados sobre `finder-infra` |
| `src` | — | Interfaz SvelteKit 5 + TypeScript (`adapter-static`) |

## Requisitos

- Rust estable (1.85 o posterior, edición 2024): `rustup default stable`
- Para la app de escritorio (módulo 10), en Arch/CachyOS:
  ```bash
  sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file openssl \
    appmenu-gtk-module libappindicator-gtk3 librsvg xdotool nodejs npm
  ```
- [Ollama](https://ollama.com) para la IA local (ver "IA y claves de API").

## Compilar y probar

```bash
cargo build
cargo test
cargo clippy --all-targets
```

### App de escritorio

```bash
npm install
npm run tauri dev     # desarrollo con recarga en caliente
npm run check         # tipos de la interfaz
```

- La base de datos vive en `~/.local/share/com.finderremotejob.app/finder.db` (Linux).
- El `.env` se busca en `~/.config/com.finderremotejob.app/.env` y, en desarrollo, en el directorio actual y sus padres (la raíz del repo). Una variable real del entorno gana sobre el archivo.
- Si la ventana sale en blanco en Linux, prueba `WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run tauri dev`.

## IA y claves de API

La app usa dos "papeles" de IA:

| Papel | Para qué | Por defecto |
|---|---|---|
| **Local** | Importar tu CV, clasificar requisitos de vacantes, similitud semántica | Ollama con `qwen3:8b` y `bge-m3`. Tu CV no sale del equipo |
| **Redactor** | Reescribir CV y cartas (módulo 9) | Claude si defines `ANTHROPIC_API_KEY`; si no, el modelo local |

Instalar la IA local (Arch/CachyOS):
```bash
sudo pacman -S ollama            # con GPU: ollama-cuda (NVIDIA), ollama-vulkan u ollama-rocm (AMD); en CPU el optimizador tarda minutos
sudo systemctl enable --now ollama
ollama pull qwen3:8b
ollama pull bge-m3
```

Las claves **nunca** van en el código: se leen de variables de entorno. Copia `.env.example` como `.env` (excluido de git) y ajusta lo que necesites; ahí están todas las variables documentadas.

Con Claude, cada petición incluye `fallbacks: "default"`: si los filtros de seguridad del modelo rechazan una petición, la API la repite con otro modelo dentro de la misma llamada.

Prueba en vivo de la IA (requiere Ollama corriendo):
```bash
cargo test -p finder-infra llm -- --ignored --nocapture
```

## Fuentes de vacantes y atribución

Las vacantes automáticas vienen de Remote OK, Remotive, We Work Remotely, Get on Board, Jobicy y Himalayas, y de las empresas que sigas en Greenhouse, Lever o Ashby (Ajustes). Sus términos piden mencionar la fuente y enlazar al anuncio original; la app guarda ambos datos en cada vacante. Además, respeta un tiempo mínimo entre consultas (Remotive: 6 horas).

## Licencias de terceros

- Fuente Liberation Sans: SIL Open Font License 1.1 (`crates/infra/assets/fonts/LICENSE-LiberationSans.txt`).

## Portales sin API oficial

LinkedIn, Indeed, Glassdoor, OCC y Computrabajo **no** se consultan automáticamente: sus términos de servicio lo prohíben y hacerlo pone en riesgo tus cuentas. En el MVP solo se pueden importar a mano (pegar el enlace o el texto de la vacante), y la postulación siempre la confirmas tú.
