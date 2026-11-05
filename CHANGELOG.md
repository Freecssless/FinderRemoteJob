# Progreso

## Fase 1: MVP

| # | Módulo | Estado |
|---|---|---|
| 1 | Workspace de Cargo + modelo de dominio (`finder-core::model`) | ✅ Terminado |
| 2 | Palabras clave y puntaje ATS (`core::ats`) | ✅ Terminado |
| 3 | Puntaje de compatibilidad (`core::matching`) | ✅ Terminado |
| 4 | Guardián antiinvención (`core::guard`) | ✅ Terminado |
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

### Módulo 3: puntaje de compatibilidad
- `matching::match_score`: habilidades 45 (un sinónimo cuenta completo), similitud semántica 15 (opcional; sin embeddings se reescala a 100), puesto 10, seniority 5, años 10, logística 15 (remoto 6, región 5, idiomas 4).
- Bloqueantes ("US only", ciudadanía, idioma requerido que no está en el CV, o los que detecte el LLM) limitan el total a 20.
- Desviación aprobada (opción b): si cubres menos del 20 % de las habilidades pedidas, el total se limita a 30 y se marca `low_relevance` ("Poco relevante").
- `years_of_experience` fusiona periodos traslapados; la fecha actual se recibe como parámetro (`core` no lee el reloj).
- `ats::score` ahora expone `title_overlap` y `Evidence`, que comparten el puntaje ATS y el de compatibilidad.
- Pendiente: salario (no hay preferencia salarial en el perfil todavía), comparar el nivel del idioma (B2 vs. "avanzado") y calibrar el reescalado del coseno con embeddings reales (módulo 7).

### Módulo 4: guardián antiinvención
- `guard::Draft`: el único formato que la IA puede proponer (resumen, bullets con `source_ids` y orden de habilidades). Empresa, puesto y fechas no están en el borrador; la plantilla los toma del maestro.
- `guard::validate` rechaza: texto vacío, falta de fuentes, ids inexistentes o de otra experiencia, cifras que no están en las fuentes (canonicaliza "1,200" = "1.200"; el resumen puede citar tus años reales) y habilidades que no están en `maestro.skills`, en el titular, en los puestos ni en las fuentes citadas.
- Un bullet rechazado vuelve al texto original de sus fuentes válidas (`Checked::final_texts`), y cada rechazo trae su motivo en español para el diff.
- Límites conocidos: no detecta cifras escritas con letra ni habilidades fuera del diccionario. Por eso siempre hay revisión humana en el diff.
