---
name: revision-seguridad
description: Revisión completa de seguridad, calidad y rendimiento de FinderRemoteJob antes de publicar o tras cambios grandes. Pruebas unitarias, pruebas de ataque (pentest), SAST, escaneo de dependencias y secretos, benchmarks y la lista de endurecimiento de 19 puntos. Úsala cuando pidan "revisión de seguridad", "auditoría", "SAST", "pentest", "¿está lista para publicar?", "escanea dependencias" o "revisa antes de subir a GitHub".
---

# Revisión de seguridad de FinderRemoteJob

App de escritorio Tauri 2 + Svelte 5 + Rust (`crates/core`, `crates/infra`, `src-tauri`), SQLite
local, sin servidor ni cuentas. Todo en español. Corrige lo que encuentres (cambio mínimo + prueba
que lo demuestre), no hagas commit hasta que el usuario pruebe la app y lo apruebe.

Trabaja en este orden y no te saltes pasos: cada uno ya encontró fallos reales.

## 1. Pruebas existentes

```sh
cargo test --workspace                      # todas (incluye tests/seguridad.rs)
cargo clippy --workspace --all-targets      # debe dar 0 avisos
npx svelte-check && npm run build           # 0 errores, 0 avisos
```

## 2. Dependencias

```sh
cargo install cargo-audit --locked          # si falta (tarda ~3 min, en segundo plano)
cargo audit                                 # 0 vulnerabilidades
npm audit --omit=dev                        # 0 en lo que va dentro de la app
```

- Vulnerable y actualizable → actualiza (así pasó `zip` 8 → 9).
- No actualizable (la librería padre ya está en su última versión) → demuestra que el código
  vulnerable no es alcanzable (`cargo tree -i <crate>@<versión> -e normal` + leer cómo se usa) y
  añade el ID a `.cargo/audit.toml` **con el motivo**. Nunca ignores sin justificar.
- `npm audit` sin `--omit=dev`: los avisos de herramientas de desarrollo no entran al binario;
  repórtalos, no los fuerces con `--force` (rompe SvelteKit).

## 3. Secretos

```sh
git log -p --all | grep -nE "sk-ant-[A-Za-z0-9_-]{10,}|sk-(proj-)?[A-Za-z0-9]{20,}|AIza[0-9A-Za-z_-]{30,}|ghp_[A-Za-z0-9]{30,}|-----BEGIN (RSA|EC|OPENSSH|PRIVATE)"
git ls-files | grep -iE "\.env$|\.db$|\.sqlite|\.pem$|\.key$|secret|credential"
git log --format='%an <%ae>' | sort -u      # el correo del autor será público en GitHub
```

Revisa que los fixtures (`crates/*/tests/fixtures`) usen datos inventados, no CVs reales.

## 4. SAST

Semgrep en un entorno aislado del scratchpad (no lo añadas al proyecto):

```sh
python3 -m venv $SCRATCH/sg && $SCRATCH/sg/bin/pip install -q semgrep
$SCRATCH/sg/bin/semgrep scan --metrics=off \
  --config p/rust --config p/javascript --config p/typescript --config p/secrets \
  --config p/owasp-top-ten --config p/security-audit --config p/gitleaks --config p/github-actions \
  --exclude node_modules --exclude target --exclude build --exclude .svelte-kit --json -o $SCRATCH/semgrep.json .
```

Falsos positivos conocidos (no los "arregles"): `temp-dir` dentro de pruebas, `args` en
`crates/core/examples/ats_demo.rs`. Las acciones del workflow deben seguir fijadas a un SHA
(`uses: x/y@<sha> # vN`); si Dependabot sube una, revisa que conserve el SHA.

Clippy con lints de pánico (un pánico = la app se cierra):

```sh
cargo clippy --workspace --lib --bins -- -A clippy::all -W clippy::string_slice -W clippy::indexing_slicing -W clippy::expect_used
```

Cada corte de texto por bytes (`&s[a..b]`) debe tener índices que caen en límite de carácter
(vienen de `find` de ASCII, de `char_indices` o del mapa de `Normalized`). Con texto en español
un índice mal calculado sobre "á" o "ñ" tumba la app.

Semgrep **no lee `.svelte`**: revisión manual obligatoria.

```sh
grep -rnE "\{@html|innerHTML|outerHTML|insertAdjacentHTML|document\.write|eval\(|new Function" src
grep -rnE "(href|src|action)=\{" src          # solo destinos fijos
grep -rnE "window\.open|postMessage|fetch\(|WebSocket" src   # todo pasa por invoke()
grep -rnE "localStorage|sessionStorage" src   # solo el perfil elegido, nunca datos del CV ni claves
```

## 5. Pruebas de ataque (pentest)

Están en `crates/infra/tests/seguridad.rs`, `src-tauri/src/backup.rs` (respaldo hostil),
`src-tauri/src/settings.rs`, `src-tauri/src/lib.rs` (subida) y `crates/core/src/guard.rs`. Si hay
una superficie nueva, añade su prueba ahí. Superficies de esta app y lo que debe cumplirse:

| Superficie | Debe cumplirse |
|---|---|
| CV → PDF (Typst) | Los textos llegan como JSON en `sys.inputs`, nunca como marcado: `#read("/etc/passwd")` sale literal. |
| CV → DOCX | XML escapado (`<w:t>` sale literal). |
| PDF/DOCX importados | 600 archivos dañados al azar sin pánico (`catch_unwind` en `cvfile::extract_text`); DOCX que descomprimido pasa de 50 MB se rechaza; DOCX con data descriptors (Google Docs) se sigue aceptando. |
| Subida de archivos | Máx. 20 MB y cabecera real (`%PDF-` / `PK\x03\x04`), no solo la extensión. |
| Vacantes (portales, enlace, texto pegado) | SQL literal (`'); DROP TABLE`), HTML convertido a texto sin `<script>`/`onerror`, respuesta de red ≤ 20 MB, texto pegado ≤ 200 000 caracteres. |
| Texto Unicode | 3000 textos al azar (acentos, emoji, ligaduras, RTL) por todo el análisis sin pánico. |
| Respaldo `.zip` restaurado | Entradas `../`, rutas absolutas y enlaces simbólicos no escapan de la carpeta de datos. Tras restaurar se borran `FRJ_WRITER`, `FRJ_LOCAL_URL` y `FRJ_OPENAI_URL` (un respaldo ajeno no desvía claves ni CV) aunque el zip no traiga `carpeta.txt`. |
| Rutas guardadas en la base | Corregir un CV solo escribe dentro de `cvs/` (`optimizer::update_saved`). |
| Enlaces que abre la app | Solo http/https/mailto/tel (permiso `opener:default`); archivos solo en `$APPDATA/cvs/**`. |
| IA engañada por un anuncio (prompt injection) | El guardián (`core::guard::validate`) rechaza cifras, habilidades, años y fuentes inventadas. |
| Proveedor de IA externo | `https://` obligatorio; `http://` solo localhost / 127.0.0.1 / [::1] exactos (`localhost.evil.com` se rechaza). |
| Instalador de Ollama | Se descarga en la caché de la app, no en la carpeta temporal compartida. |

## 6. Rendimiento

```sh
cargo test --release -p finder-infra --test rendimiento -- --ignored --nocapture
cargo test --release -p finder-core banco -- --ignored --nocapture
```

Referencia (nov 2026, 12 núcleos). Si algo empeora más del doble, investígalo:

| Proceso | Tiempo |
|---|---|
| Generar PDF / DOCX | 2,5 ms / 0,14 ms |
| Leer PDF / DOCX | 1,2 ms / 0,18 ms |
| Importar CV sin IA (normal / 555 KB) | 0,1 ms / 24 ms |
| Compatibilidad de 500 vacantes | 36 ms |
| Guardar 5000 vacantes / listar 5000 | 0,17 ms c/u / 48 ms |
| Importar CV con IA local (qwen3:8b, CPU) | ~30 s |

Compila también la app completa: `cargo build --release -p finder-app` (debe terminar sin avisos).

## 7. Lista de endurecimiento (19 puntos)

Revisa cada punto contra el código actual; no lo des por bueno solo porque esta tabla lo diga.

| Punto | Aplica | Dónde se cumple |
|---|---|---|
| Ocultar API Key | Sí | Llavero del sistema (`settings.rs`); la interfaz solo ve `saved_keys`; nunca en DB ni registros. |
| Purgar secrets de git | Sí | Paso 3. |
| Key pública de BD / RLS | No | SQLite local, sin Supabase/Postgres. |
| Encriptación de datos sensibles | Parcial | Carpeta de datos 0700 y respaldo 0600 en Unix (AppData en Windows). Sin SQLCipher a propósito (riesgo en builds de Windows; basta el cifrado de disco del sistema). |
| Autenticación del servidor | No | No hay servidor. Las conexiones salientes verifican TLS (no hay `danger_accept_invalid_certs`). |
| Restringir acceso a registros | Sí | Capacidades de Tauri mínimas (`src-tauri/capabilities/default.json`). |
| Bloquear manipulación de campos | Sí | Ajustes con lista blanca (`PLAIN`, `SECRETS`); guardián del CV. |
| Cookies de sesión / hash de contraseñas / límite de login / antibots | No | Sin cuentas ni login ni formularios públicos. |
| Parametrizar SQL | Sí | Todo con `.bind()`; búsqueda FTS entrecomillada (`fts_query`). Busca `format!` dentro de `sqlx::query`. |
| Validar inputs | Sí | Topes de tamaño (paso 5). |
| Escapar contenido del usuario | Sí | Svelte escapa; sin `{@html}`; Typst/DOCX probados. |
| Restringir subida de archivos | Sí | `check_upload` (backend) + tope en `api.ts` (evita congelar la ventana). |
| Recortar respuestas de la API | Sí | Comandos devuelven solo lo que la vista usa. |
| Headers de seguridad | Sí | CSP en `tauri.conf.json`: `object-src`, `base-uri`, `form-action`, `frame-src`, `frame-ancestors` en `'none'`. Todo `<form>` debe llamar `preventDefault`. |
| Forzar HTTPS | Sí | `secure_url` en `crates/infra/src/llm/mod.rs`. |
| Escanear dependencias | Sí | Paso 2 + `.github/dependabot.yml`. |

## 8. Privacidad y licencias al día

Los documentos legales prometen cosas concretas: si el código cambió, revisa que sigan siendo ciertas.

- **Servicios externos:** `grep -rhoE "https://[a-z0-9.-]+\.[a-z]{2,}" crates/infra/src src-tauri/src src | sort -u`.
  Todo destino nuevo debe aparecer en `PRIVACIDAD.md` §2 y en la tabla de `src/routes/acerca/+page.svelte`.
- **Qué recibe la IA externa:** `for_writer` y `writer_request` en `crates/infra/src/optimizer.rs`
  deben coincidir con el texto del consentimiento en Ajustes y con `PRIVACIDAD.md`.
- **Licencias:** sin GPL, LGPL, AGPL ni licencias de uso personal en lo que entra al binario
  (`cargo metadata` desde `finder-app`, solo dependencias normales y de compilación, más
  `node_modules`). Si cambian las dependencias, regenera la lista de `LICENCIAS-TERCEROS.md`.
- **Recursos:** ningún icono, imagen o fuente sin licencia clara (el icono de la app sale de
  `src-tauri/icons/icon.svg`; nunca los logos de Tauri o Svelte).
- **Si cambian la política o los términos:** sube `LEGAL_VERSION` en `src/lib/legal.ts` y la
  versión/fecha de los dos documentos.

## 9. Informe

En español, para alguien no técnico:

1. Una línea de veredicto y los totales (pruebas, avisos, vulnerabilidades).
2. Tabla **Corregido**: qué había, riesgo, qué hice.
3. **Ya se cumplía (verificado)** y **No aplica (por qué)**.
4. Rendimiento en tabla.
5. Al final: qué no revisaste y qué riesgo queda (p. ej. Windows sin probar, SQLCipher no aplicado).

Añade una entrada al `CHANGELOG.md`. No hagas commit hasta que el usuario diga que lo probó; luego
un solo commit con `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` y sin push.
