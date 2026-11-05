# Progreso

## Fase 1: MVP

| # | Módulo | Estado |
|---|---|---|
| 1 | Workspace de Cargo + modelo de dominio (`finder-core::model`) | ✅ Terminado |
| 2 | Palabras clave y puntaje ATS (`core::ats`) | ✅ Terminado |
| 3 | Puntaje de compatibilidad (`core::matching`) | ⏳ Pendiente |
| 4 | Guardián antiinvención (`core::guard`) | ⏳ Pendiente |
| 5 | Base de datos SQLite + migraciones (`infra::db`) | ⏳ Pendiente |
| 6 | Fuentes de vacantes: Remote OK, Remotive, WWR, Get on Board, Jobicy + importación manual (`infra::sources`) | ⏳ Pendiente |
| 7 | Cliente LLM (Ollama / OpenAI-compatible) + importación de CV PDF/DOCX | ⏳ Pendiente |
| 8 | Generación de PDF (Typst) y DOCX + prueba de ida y vuelta (`infra::docgen`) | ⏳ Pendiente |
| 9 | Flujo completo del optimizador de CV | ⏳ Pendiente |
| 10 | App Tauri + interfaz Svelte (perfiles, vacantes, optimizador, tablero) | ⏳ Pendiente |

## Registro

### Módulo 1: workspace y modelo de dominio
- Workspace de Cargo (edición 2024) con `crates/core` (`finder-core`) y `crates/infra` (`finder-infra`).
- Desviación aprobada del plan: los paquetes se llaman `finder-core` y `finder-infra`, porque `core` choca con el crate `core` de la librería estándar.
- Modelos: `Profile`, `CvMaster` (con `id` en cada experiencia, logro y habilidad), `Job`, `JobRequirements`, `Term`, `Status` y `Application`.
- CV de ejemplo en `crates/core/tests/fixtures/cv_editor_video.json`.
- Lint `clippy::unwrap_used` activado en todo el workspace.

### Módulo 2: palabras clave y puntaje ATS
- `ats::keywords`: normalización sin acentos que conserva la forma original del anuncio, diccionario de habilidades con sinónimos (`data/skills/*.csv`, ~140 habilidades, compilado en el binario), búsqueda con Aho-Corasick respetando bordes de palabra, y `extract_terms` (requerido o deseable según la sección; peso 3/1, +2 si está en el título, +ln(frecuencia)).
- `ats::score`: puntaje ATS de 0 a 100 (palabras clave 60 con 0.7 para sinónimos, título 10, secciones y contacto 10, legibilidad 20), sugerencias concretas y `classify_gaps` (huecos con y sin respaldo en el CV maestro).
- Ejemplo: `cargo run -p finder-core --example ats_demo -- vacante.txt cv.txt "Puesto"`.
- Desviaciones menores: la similitud de título usa coincidencia de palabras en lugar de `strsim`, y no se agregó `rust-stemmers` (los sinónimos cubren las variantes por ahora). Ninguno de los dos crates hacía falta todavía.
- Pendiente para módulos posteriores: el diccionario ESCO completo; la clasificación requerido/deseable con LLM (módulo 7), que hoy marca como requerido todo lo que aparece en "Responsabilidades"; la legibilidad con el PDF real (módulo 8).
