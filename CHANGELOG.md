# Progreso

## Fase 1: MVP

| # | Módulo | Estado |
|---|---|---|
| 1 | Workspace de Cargo + modelo de dominio (`finder-core::model`) | ✅ Terminado |
| 2 | Palabras clave y puntaje ATS (`core::ats`) | ⏳ Pendiente |
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
