# Progreso

## Fase 1: MVP

| # | Módulo | Estado |
|---|---|---|
| 1 | Workspace de Cargo + modelo de dominio (`finder-core::model`) | ✅ Terminado |
| 2 | Palabras clave y puntaje ATS (`core::ats`) | ✅ Terminado |
| 3 | Puntaje de compatibilidad (`core::matching`) | ✅ Terminado |
| 4 | Guardián antiinvención (`core::guard`) | ✅ Terminado |
| 5 | Base de datos SQLite + migraciones (`infra::db`) | ✅ Terminado |
| 6 | Fuentes de vacantes: Remote OK, Remotive, WWR, Get on Board, Jobicy + importación manual (`infra::sources`) | ✅ Terminado |
| 7 | Cliente LLM (Ollama / OpenAI-compatible / Anthropic) + importación de CV PDF/DOCX | ✅ Terminado (probado en vivo con Ollama) |
| 8 | Generación de PDF (Typst) y DOCX + prueba de ida y vuelta (`infra::docgen`) | ✅ Terminado |
| 9 | Flujo completo del optimizador de CV | ✅ Terminado (probado en vivo con qwen3:8b) |
| 10 | App Tauri + interfaz Svelte (perfiles, vacantes, optimizador, tablero) | ✅ Terminado (10a–10d + recordatorios) |

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

### Módulo 5: base de datos SQLite
- `infra/migrations/0001_init.sql`: esquema de la §5.2 del plan. Incluye las fuentes del MVP precargadas e índices para el tablero.
- Detalle técnico de FTS5: la tabla de búsqueda guarda su propia copia del texto (mantenida con triggers) en vez de usar "external content", porque el rowid de `jobs` (clave TEXT) puede cambiar con VACUUM.
- `infra::db::Db`: perfiles (crear, editar, listar, borrar con todo lo suyo), CV maestro (conserva el texto del archivo importado), vacantes (sin duplicados por `dedupe_key`), búsqueda de texto completo segura, tablero con filtros por perfil, estado, portal y fechas, cambios de estado con historial, notas y próximo paso, y versiones de CV vinculadas a la postulación.
- `core`: `Status::as_str/parse`, `RemotePref::as_str/parse`, `dedupe_key`, `CvVersion`, `StatusChange` y `pipeline::transition` (fija `applied_at` y `first_response_at`).
- Decisiones confirmadas: cada combinación de vacante y perfil tiene su fila en `applications` desde "Encontrada"; la ruta del archivo la decide la app.
- La fecha actual se toma de SQLite (`strftime`), sin agregar un crate de fechas.
- Pendiente: recordatorios (tabla creada, se usará en la fase 2) y estadísticas.

### Módulo 6: fuentes de vacantes
- 5 fuentes automáticas con API o RSS públicos, verificadas en vivo el 2026-11-05: Remote OK (99 vacantes), Remotive (10), We Work Remotely (100), Get on Board (15 por búsqueda) y Jobicy (100, filtro LATAM).
- Se respetan las condiciones de cada una: se guarda el enlace original y la fuente (atribución), y hay un intervalo mínimo entre consultas (Remotive: 6 h, por su límite de 4 al día; las demás: 1 h), controlado con `sources.last_sync_at`.
- `sources::sync`: descarga, filtra por los puestos del perfil ("Editor de video" acepta "Video Editor"), guarda sin duplicados y devuelve los ids nuevos.
- `sources::manual`: importar por texto pegado o por URL (una sola petición; lee el JSON-LD `JobPosting` de schema.org o el texto principal; si el portal bloquea, pide pegar el texto).
- `ASSISTED_PORTALS`: LinkedIn, Indeed, OCC, Computrabajo, Bumeran y Glassdoor solo generan el enlace de búsqueda para abrirlo en el navegador. Sin scraping ni postulación automática.
- Desviación aprobada: WWR se lee con `quick-xml` en lugar de `feed-rs`, para conservar `<region>` ("USA Only"), que se usa como bloqueante.
- `ego-tree` se declara como dependencia explícita (ya venía con `scraper`) para recorrer el HTML.
- Pruebas con respuestas reales guardadas en `crates/infra/tests/fixtures/`, así que `cargo test` no usa la red. La prueba en vivo se corre a mano: `cargo test -p finder-infra -- --ignored --nocapture`.
- Limitaciones: la API pública de Remotive devuelve solo 10 vacantes; no se pudieron leer los términos de Get on Board (su página de documentación usa JavaScript); los enlaces de búsqueda de OCC, Computrabajo y Bumeran siguen el formato actual de sus sitios y pueden cambiar.

### Módulo 7: cliente de IA e importación de CV
- `infra::llm`: configuración por variables de entorno (`.env.example`), dos papeles (local y redactor) y dos backends: compatible con OpenAI (Ollama, OpenAI, Gemini, Groq, OpenRouter) y Anthropic nativo.
- Anthropic por HTTP directo (no hay SDK de Rust): API `2026-10-21`, `output_config.format` (json_schema), modelo por defecto `claude-opus-5-5` con esfuerzo "medium", y `fallbacks: "default"` activado por defecto. Se revisa `stop_reason` (rechazo o corte) antes de leer la respuesta.
- `infra::cvfile`: texto de PDF (`pdf-extract`, con protección contra pánicos) y DOCX (`docx-rs`, incluidas tablas).
- `core::import::finalize`: los ids los asigna Rust (no la IA); calcula las cifras y normaliza las fechas; avisa de todo dato que no aparece tal cual en el archivo original.
- `core::ats::requirements::merge`: la IA reclasifica requerido/deseable y aporta habilidades extra, seniority, años, idiomas y bloqueantes; todo se verifica contra el texto del anuncio y se descarta lo que no esté.
- `matching::cosine` y `tasks::semantic_similarity` (embeddings con Ollama).
- Desviación menor: el plan mostraba un `trait Llm`; se usa un `enum Backend`, porque los métodos async en traits no admiten `dyn` y con dos backends basta.
- Pendiente: probar en vivo con Ollama (no estaba instalado); reintentos automáticos ante errores 429/529 (hoy se marcan como reintentables para la interfaz); la anonimización de datos personales antes de enviar a una API externa (fase 2 del plan; por eso la importación usa el modelo local).

### Módulo 8: generación de PDF y DOCX
- `core::document::CvDocument`: el CV listo para imprimir (orden de secciones, encabezados estándar en español o inglés, fechas uniformes "ene 2022 – actual", nombre de archivo `Nombre_Apellido_Puesto_Empresa`) y detección simple del idioma de la vacante.
- Plantilla Typst **Clásica** (`infra/templates/clasica.typ`): una columna, sin tablas, íconos ni encabezado o pie de página; los datos entran como JSON (`sys.inputs.data`), así que `#`, `*`, `$`, `<` o `@` del CV se imprimen literalmente.
- Fuente Liberation Sans (Regular, Bold, Italic; licencia OFL en `infra/assets/fonts/`) incluida en el binario: el PDF sale igual en cualquier sistema.
- DOCX con `docx-rs` usando estilos nativos ("Heading 1" y viñetas reales); fuente Arial, que tiene la misma métrica que Liberation Sans.
- `docgen::verify`: prueba de ida y vuelta (vuelve a leer el archivo y comprueba palabras y orden de secciones); `ats::score::with_roundtrip` aplica el resultado a la legibilidad del puntaje ATS.
- Dependencias: `typst-as-lib` 0.17, `typst`/`typst-pdf`/`typst-layout` 0.15 (`typst-layout` ya venía con `typst`; se declara por `PagedDocument`).
- Muestra para revisar a ojo: `cargo test -p finder-infra muestras -- --ignored` → `/tmp/finder-muestras/`.
- Pendiente para la versión completa: plantillas Compacta y Técnica (fase 2 del plan).

### Módulo 9: optimizador de CV
- `infra::optimizer`: `prepare` (requisitos con IA local o, si no responde, solo diccionario; ATS antes; huecos; idioma) → `optimize` (el redactor propone un `Draft` con esquema JSON estricto) → `apply_draft` (guardián, CV final, PDF, ida y vuelta, ATS después) → `save` (decisiones del usuario, PDF y DOCX en la carpeta indicada, `cv_versions` vinculada a la postulación y estado "CV adaptado" sin retroceder).
- `core::document::CvDocument::from_report` + `Choices`: los rechazados por el guardián o por el usuario vuelven al texto original; las habilidades se reordenan sin quitar ninguna; empresa, puesto y fechas siempre salen del maestro.
- Privacidad: el redactor no recibe nombre, correo, teléfono, ubicación ni enlaces (los pone la plantilla).
- Nombre de archivo con sufijo del id de versión, para no sobrescribir versiones anteriores.
- `master_hash` (FNV-1a sobre el JSON del maestro) identifica de qué versión del maestro salió cada CV.
- Ejemplo de prueba: ATS 67 → 69 con un borrador honesto; lo que sigue faltando (TikTok, Instagram…) no está en el maestro y se queda como hueco.
- Carta de presentación: pasa a la fase 2 (decisión del usuario).
- Pendiente: probar en vivo con Ollama o Claude (`cargo test -p finder-infra optimizer -- --ignored --nocapture`).

### Módulo 10a: esqueleto de la app y pantalla Perfiles
- `src-tauri` (`finder-app`) en el workspace; interfaz SvelteKit 5 + TypeScript en `src/` con `adapter-static`.
- Base de datos en la carpeta de datos de la app; `.env` leído sin tocar el entorno del proceso (`envfile.rs`, con prueba) y pasado a `LlmConfig::from_lookup`.
- Comandos: perfiles (listar, crear, editar, borrar), CV maestro (leer, guardar) e importar PDF/DOCX con la IA **local**. Al guardar se recalculan las cifras de cada logro.
- Pantalla Perfiles: formulario del perfil, borrado con confirmación, importación con avisos de revisión, editor completo del CV maestro (ids estables para el guardián) y aviso de cambios sin guardar.
- Probado por el usuario en la app: importar su CV con Ollama (qwen3:8b) y guardarlo.
- `LlmError::ModelNotFound`: el 404 de Ollama por modelo no descargado se explica en español con el comando `ollama pull`.

### Módulo 10b: pantalla Vacantes
- `src-tauri/src/jobs.rs`: sincronizar las cinco fuentes con los puestos buscados del perfil (cada una respeta su intervalo; si una falla, las demás siguen), importar por enlace o texto pegado, lista y ficha con compatibilidad, y análisis de requisitos con la IA local, que se guarda en la vacante.
- La compatibilidad se calcula al vuelo para el perfil elegido (hasta 500 vacantes). Si una vacante no se ha analizado con IA, se usan los términos del diccionario.
- Filtros: búsqueda de texto completo, fuente y "ocultar poco relevantes y bloqueadas". Enlaces de búsqueda a LinkedIn, Indeed, OCC, Computrabajo y Bumeran (se abren en el navegador).
- El perfil activo se recuerda entre pantallas (`localStorage`, solo como comodidad).
- Probado por el usuario en la app.
- `finder_infra::sources::Client` se reexporta para que la app no dependa de `reqwest` directamente.

### Módulo 10c: pantalla Optimizador
- `src-tauri/src/optimizer.rs`: `run_optimizer` (requisitos con IA local, que se guardan en la vacante si no estaban; ATS antes; borrador del redactor; guardián; ATS después), `get_optimization` y `save_optimized`.
- La propuesta queda en memoria por vacante: la interfaz solo envía las decisiones al guardar y se puede guardar otra versión con otras decisiones sin volver a llamar a la IA. Se pierde al cerrar la app.
- Al guardar, la vacante entra al tablero del perfil (con su compatibilidad) y pasa a «CV adaptado». PDF y DOCX en `<datos de la app>/cvs/`.
- Pantalla: ATS antes → después, huecos con y sin respaldo, sinónimos a cambiar por la forma del anuncio, diff por logro (tu texto / propuesta) con casillas, rechazos del guardián con su motivo, y botones para abrir el PDF/DOCX o mostrar la carpeta.
- Permiso `opener:allow-open-path` limitado a `$APPDATA/cvs/**`.
- Botón «Adaptar mi CV» en la ficha de la vacante.
- Prueba en vivo del optimizador con `qwen3:8b` como redactor (7 min 16 s): el guardián rechazó 3 de 5 logros por habilidades inventadas (TikTok, Instagram, motion graphics, edición de audio, redes sociales). ATS 67 → 67.
- Corregido en `CvDocument::from_report`: un logro rechazado ya no devuelve un original que otra propuesta aceptada ya incluye (salía repetido en el CV). Prueba `rechazo_no_duplica_un_logro_ya_usado`.
- Optimizador más rápido y que no baja el puntaje (probado con el CV real del usuario: antes bajaba 62 → 56; ahora 74 → 76 y unos 5 min en CPU, antes más de 7):
  - IA local por la API nativa de Ollama (`llm::ollama`): sin razonamiento (`think: false`, ~8× más rápido con qwen3) y contexto de 8192 tokens (con 4096 Ollama recortaba en silencio el prompt del redactor).
  - Prompt del redactor compacto (sin métricas, educación ni idiomas) y como máximo 3 logros reescritos por experiencia; el resumen es obligatorio en el esquema.
  - `from_report`: los logros que ninguna propuesta usada cubre se conservan (el redactor resumía 7 logros en 4 y se perdían).
  - `CvDocument::align_skills`: cada habilidad se escribe como la nombra la vacante si el diccionario dice que es la misma ("Corrección de color (Color Grading)").
  - Por defecto se desmarca una propuesta que pierde términos de la vacante que su original tenía.
  - Requisitos: se descartan "habilidades" que son frases del anuncio (`is_skill_like`), también en vacantes ya analizadas.
  - Guardián: el resumen se respalda con todo el CV, y las empresas cuentan como respaldo ("Canal de YouTube" → YouTube).
  - Ida y vuelta del PDF: pdf-extract pega líneas ("VillegasEditor"); ya no cuenta como palabra perdida ni se puntúa ese texto (costaba 8 puntos de legibilidad).

### Módulo 10d: pantalla Tablero
- `src-tauri/src/board.rs`: `list_board`, `set_application_status` (fija «postulada el» y «primera respuesta» según `core::pipeline`, con historial), `save_application_notes`, `application_detail` (historial y CV enviado) y `add_to_board`.
- Kanban por perfil con las 10 columnas de estado: arrastrar una tarjeta cambia su estado (también desde un selector en la ficha, accesible con teclado). Búsqueda por puesto o empresa.
- Ficha: anuncio, compatibilidad, fechas, CV enviado (ATS antes → después, abrir PDF/DOCX), notas, próximo paso con fecha (en rojo si ya venció) e historial.
- Vacantes: botón «Agregar al tablero» (entra como «Encontrada» con su compatibilidad).
- `tauri.conf.json`: `dragDropEnabled: false` (la app no recibe archivos arrastrados; así funciona el arrastre del Kanban en todas las plataformas).

### Recordatorios (plan §5.3)
- `core::pipeline::reminders_on`: al pasar a «Postulada», seguimiento a los 7 días y «¿Sin respuesta?» a los 21; al pasar a «Entrevista», agradecimiento a las 24 h. Plazos configurables (`ReminderDays`, tabla `settings`).
- `Db::set_status` cierra los recordatorios pendientes del estado anterior y crea los del nuevo; la fecha del próximo paso de la ficha también es un recordatorio (se reemplaza al cambiarla). `list_due_reminders`, `pending_reminders`, `complete_reminder`, `reminder_days`.
- La app revisa los vencidos al arrancar y cada hora, y los avisa con `tauri-plugin-notification` (uno por sesión; varios se agrupan en una notificación).
- Tablero: lista de vencidos con «Abrir», «Hecho» y «Marcar sin respuesta»; recordatorios pendientes en la ficha; plazos editables.
- Las postulaciones que ya estaban en «Postulada» antes de este cambio no tienen recordatorios (se crean al cambiar de estado).

### Estadísticas (plan §5.5)
- `core::stats::compute`: solo cuentan las postulaciones con ≥ 14 días; tasa de respuesta global y por portal, perfil, plantilla de CV y rango de ATS (<60, 60–79, ≥80); días medianos hasta la primera respuesta; embudo Postulada → Entrevista → Oferta (según el historial, no el estado actual); compatibilidad promedio de las que respondieron y de las que no. Cada grupo con su n; con menos de 10, «pocos datos» y al final.
- `Db::stat_inputs` y comando `get_stats`.
- Pantalla Estadísticas: tarjetas de cifras, embudo y tablas con barra de tasa por grupo; filtro por perfil o todos. Color de barra validado con el script de dataviz (claro y oscuro: token `--chart-bar`).

### Vista de lista y filtros del Tablero (plan §5.4)
- Tablero: botón Kanban / Lista. La lista es una tabla (puesto, empresa, portal, estado, fecha, compatibilidad); al pulsar el puesto se abre la misma ficha.
- Filtros por portal, rango de fechas (de postulación, o de hallazgo si aún no te postulas) y compatibilidad mínima, en las dos vistas; el filtro por estado solo en la lista (en el Kanban ya son las columnas). «Quitar filtros» los limpia.

### Plantillas Compacta y Técnica (plan §4.6)
- `core::document::Template` (Clásica, Compacta, Técnica) decide el orden de las secciones (`CvDocument::sections`); PDF, DOCX, texto plano y prueba de ida y vuelta usan ese mismo orden.
- Compacta: letra de 9.5 pt y márgenes y espacios más chicos para caber en una página. Técnica: Habilidades justo después del Resumen. Como el CV maestro no tiene sección de proyectos, la Técnica no puede ponerlos antes que Educación, como pedía el plan.
- Una sola plantilla Typst (`templates/cv.typ`, antes `clasica.typ`) con el orden que le pasa Rust.
- Optimizador: eliges la plantilla al guardar; se guarda como `variant` (sale en la ficha del Tablero y en Estadísticas «Por plantilla de CV»).
- Pruebas de ida y vuelta para las tres plantillas en PDF y DOCX, español e inglés.

### Carta de presentación (plan §4.6)
- `core::guard::validate_paragraphs`: cada párrafo de la carta pasa por las mismas reglas que el resumen (fuentes citadas, sin cifras ni habilidades que no estén en el CV).
- `infra::letter`: el redactor escribe 3 párrafos (por qué el puesto, 2 o 3 logros, cierre) sin recibir tu nombre ni tus datos de contacto; el saludo («Estimado equipo de …») y la firma se agregan localmente. Si el guardián rechaza algún párrafo, hay una segunda vuelta con la lista de términos prohibidos y se usa el párrafo nuevo solo si pasa.
- `Db::set_cover_letter`; comandos `write_cover_letter` y `save_cover_letter` (este último agrega la vacante al tablero si no estaba).
- Optimizador: tarjeta «Carta de presentación» para generar, ver los párrafos rechazados con su motivo, editar, guardar y copiar. Tablero: la ficha muestra la carta guardada con «Copiar carta».
- Probado en vivo con qwen3:8b (CPU): unos 40–70 s; 2 de 3 párrafos aprobados tras la segunda vuelta.

### Ajustes de IA y claves en el llavero (Fase 2)
- Pantalla Ajustes: IA local (URL, modelo, embeddings) y redactor (local, Claude o compatible con OpenAI) con modelo, esfuerzo y URL. Al guardar, la configuración se recarga sin reiniciar la app.
- Las claves de API van al llavero del sistema (`keyring`: KWallet / GNOME Keyring vía Secret Service, Credential Manager en Windows, Keychain en macOS), nunca a la base de datos. Se pueden borrar desde la pantalla.
- Prioridad: variable de entorno > `.env` > Ajustes > valor por defecto. Los campos fijados por el entorno o el `.env` aparecen bloqueados con el aviso.
- `Db::settings_with_prefix` y `Db::set_setting` (tabla `settings`, prefijo `llm.`).
- La pantalla recuerda qué se envía a una API externa: vacante y CV maestro sin nombre, correo, teléfono, ubicación ni enlaces.

### Costo estimado del redactor externo (Fase 2)
- Con Claude o un proveedor compatible con OpenAI, el Optimizador muestra antes de generar los tokens aproximados y el costo en USD de la propuesta de CV y de la carta (`optimizer::estimate_tokens`, `letter::estimate_tokens`, comando `estimate_cost`).
- Los precios (USD por millón de tokens de entrada y salida) los escribe el usuario en Ajustes, por proveedor; la app no trae precios fijos que se queden viejos. Sin precios, solo se muestran los tokens.
- Estimado simple: ~3 caracteres por token y salida fija (1.5 k el CV, 0.6 k la carta); no cuenta el razonamiento de Claude ni la segunda vuelta de la carta.

### Cola de postulación asistida y respuestas frecuentes (Fase 2)
- Pantalla Postular: las 20 vacantes más compatibles (≥ 70) del perfil que aún no enviaste (sin postulación, o en Encontrada / Evaluada / CV adaptado), con si ya tienen CV adaptado y carta.
- «Abrir y postular» abre el anuncio en el navegador, copia la carta al portapapeles y muestra el PDF en su carpeta para arrastrarlo. «Ya me postulé» la agrega al tablero si hace falta y la pasa a Postulada (fecha y recordatorio de seguimiento como en el Tablero).
- Respuestas frecuentes por perfil: las calculadas del CV maestro (años de experiencia, años por habilidad, idiomas) y las tuyas (salario, disponibilidad, visa, horario…), editables y con botón de copiar. Se guardan en `settings` (`faq.<perfil>`) y se borran con el perfil.
- La app no llena formularios ni entra a tus cuentas: tú envías la solicitud.

### Fuente Himalayas (Fase 2)
- Nueva fuente automática `himalayas` (API pública, sin clave): busca por cada puesto objetivo del perfil (máx. 3, 20 resultados cada uno), como Get on Board. Migración `0002_himalayas.sql`.
- La URL guardada es su `applicationLink` y la fuente se muestra en cada vacante, como piden sus condiciones.
- Salario mensual convertido a anual (×12); otros periodos (por hora…) se omiten.
- `sources::search` reúne la búsqueda por palabra clave que usaban Get on Board y ahora Himalayas.

### Empresas que sigues: Greenhouse, Lever y Ashby (Fase 2)
- En Ajustes, «Empresas que sigues»: pegas la URL de la página de empleos de una empresa (boards.greenhouse.io, job-boards.greenhouse.io, jobs.lever.co, jobs.ashbyhq.com) o `sistema:slug`. Se guarda como `greenhouse:gitlab` en `settings` (`sources.boards`); una línea no reconocida se rechaza con aviso.
- Tres fuentes nuevas (`sources::boards`, migración `0003_company_boards.sql`) con sus APIs públicas de publicaciones, sin clave. Solo se consultan si sigues alguna empresa en ese sistema; las vacantes pasan por el mismo filtro de puestos del perfil.
- Una empresa que falla (slug mal escrito) no impide traer las demás.
- Lever y Ashby no dan el nombre de la empresa: se toma del slug. Ashby agrega el rango de compensación a la descripción.
- `Db::setting` para leer un ajuste suelto.

### Fuentes con clave: Adzuna y Jooble (Fase 2)
- Dos fuentes nuevas (`sources::adzuna`, `sources::jooble`, migración `0004_adzuna_jooble.sql`) que solo se consultan si pones tu clave gratuita en Ajustes → «Fuentes de vacantes con clave». Las claves van al llavero del sistema; App ID, país y dominio a `settings`. También se pueden fijar en `.env`.
- Adzuna: búsqueda por puesto (máx. 3) en el país elegido (por defecto `mx`); no se guarda el salario que Adzuna "predice".
- Jooble: la clave gratuita tiene 500 consultas en total, así que se consulta una vez al día con todos los puestos en una sola petición. La clave es por país (dominio `mx.jooble.org` por defecto).
- La clave va dentro de la URL de ambas APIs: los errores nunca muestran la URL (`send_private`), con prueba.
- Los ejemplos de prueba de ambas están armados con el formato de su documentación; falta una prueba con respuesta real.

### Lector de alertas por correo (Fase 2)
- Nueva fuente `email` (`sources::email`, migración `0005_email.sql`): por IMAP con TLS lee los correos de alertas de LinkedIn, Indeed, OCC, Computrabajo y Bumeran de los últimos 14 días y saca los enlaces a vacantes (título del enlace; empresa y ubicación del texto de alrededor). La app nunca entra a esos portales.
- El buzón se abre en solo lectura (`EXAMINE`) y los correos se bajan con `BODY.PEEK[]`: no se marcan como leídos ni se mueven. Solo se buscan esos remitentes (constantes; no entra texto del usuario en la búsqueda IMAP).
- Ajustes → «Alertas por correo»: servidor, puerto, usuario y contraseña de aplicación (al llavero). También en `.env`.
- Dependencias nuevas: `async-imap`, `mail-parser`, `tokio-rustls` y `rustls-platform-verifier` (certificados del sistema).
- La lectura de cada alerta es heurística y se probó solo con un correo armado a mano: falta afinarla con alertas reales.

### Similitud semántica con embeddings (Fase 2)
- La compatibilidad ya usa el componente de similitud semántica (15 de 100 puntos): coseno entre el embedding del CV maestro (titular, habilidades, puestos y logros, sin datos personales) y el de la vacante (puesto + descripción, recortados a 6000 caracteres). Sin embeddings, el puntaje se calcula como antes (los otros 85 puntos escalados a 100).
- Los vectores se calculan con la IA local (`FRJ_EMBED_MODEL`, por defecto `bge-m3`) y se guardan en la tabla `embeddings` (migración `0006_embeddings.sql`) con el hash del texto: si cambias el CV, se recalcula el suyo.
- Se calculan solos para las vacantes nuevas al sincronizar y al analizar una vacante (si la IA local no responde, no se interrumpe nada). Botón «Calcular similitud semántica» en Vacantes para las 500 más recientes que no la tengan.
- La ficha de compatibilidad muestra «Similitud con tu CV» cuando existe.
- `finder_infra::semantic` (`ensure`, `Vectors`), `Db::embeddings` y `Db::put_embedding`.

### Perfil activo en el menú
- **Selector único:** el perfil se elige una sola vez en el menú lateral («Perfil activo»), en lugar de un selector en Vacantes, Postular y Tablero. Esas pantallas se recargan solas al cambiarlo, y las pestañas de Perfiles también lo cambian. En el riel de ventanas medianas aparece como un círculo con la inicial, que lleva a Perfiles.
- **Avisos más ligeros:** se calculan con un solo comando, `nav_counts`, en vez de cargar todas las vacantes en cada cambio de pantalla. Las vacantes nuevas se cuentan con una consulta SQL (`Db::count_jobs_found_within`).
- **Prueba en vivo de la similitud semántica** con `bge-m3`: la vacante parecida al CV puntúa 0,74 y una sin relación 0,38.

### Más fácil de navegar, sin huecos y con más personalidad
- **Encabezado común en cada pantalla:** un icono de color propio de la sección (con una entrada animada), el título, una línea que dice para qué sirve y las acciones a la derecha.
- **Avisos en el menú:** cuántas vacantes nuevas hay (24 h), cuántas esperan en Postular y cuántos recordatorios vencieron en el Tablero.
- **Onda de Material 3** al pulsar botones, destinos y pestañas.
- **Perfiles:**
  - Los perfiles pasan a pestañas arriba, cada uno con su inicial; ya no hay columna lateral con hueco debajo.
  - El formulario usa dos columnas.
  - El CV se reparte en dos columnas, con Datos básicos y Habilidades lado a lado.
  - La barra «CV maestro» queda fija arriba al desplazarse.
- **Vacantes:** la lista y el detalle llenan la altura de la ventana y se desplazan cada uno por su cuenta.
- **Ajustes:** dos columnas (la IA a la izquierda; respaldo y opciones avanzadas a la derecha), y «Restaurar una copia» como botón.
- **Optimizador:** usa el mismo encabezado con el puesto y la empresa.

### Pantallas mejor seccionadas
- **Perfiles:**
  - La lista va en su propio panel.
  - «Crear paso a paso» es un botón tonal junto a «Perfil vacío».
  - El CV maestro queda separado del formulario del perfil por un divisor.
- **Habilidades:** son fichas compactas. Se agregan escribiendo y pulsando Enter (varias separadas por comas, sin repetir). Se editan tocándolas, una a la vez, y se quitan con su X.
- **Respuestas frecuentes (Postular):** filas compactas con botones para copiar y editar; solo la que editas se abre como formulario.
- **Vacantes:** tres bloques separados.
  - La barra de búsqueda (campo con lupa, fuente, filtro y «Buscar también en») va en su propio panel.
  - La lista, en otro.
  - El detalle, en otro, con sus secciones divididas por líneas en vez de tarjetas dentro de tarjetas.
  - Sin vacante elegida, el detalle invita a elegir una.

### Desplegables, Kanban y menos texto
- **Menús desplegables propios** (`Select.svelte`) en lugar de `<select>`: en Linux la lista salía con el estilo de GTK. Ahora siguen Material 3, con animación, marca en la opción elegida y uso completo con teclado.
- **Paneles expandibles** («Filtros», «Opciones avanzadas», «Plazos», informe de búsqueda): fila con flecha que gira, capa de estado y entrada suave.
- **Kanban:**
  - La rueda del ratón desplaza el tablero de lado.
  - Hay flechas ‹ › y un desvanecido en el borde donde hay más columnas.
  - Las columnas miden 248px y su alto se ajusta al contenido; las vacías muestran solo «Suelta aquí».
- **Menos texto:**
  - Se acortaron o quitaron explicaciones en Vacantes, Postular, Optimizador, Ajustes, Bienvenida y el editor de CV.
  - El resultado de «Buscar vacantes nuevas» es una línea («6 vacantes nuevas · 1 portal falló»), con el detalle por portal plegado.
  - En Vacantes, «Volver a analizar con IA» pasa a botón de texto y «Abrir anuncio» a «Anuncio».
  - En Postular, el estado de cada vacante se muestra con fichas («CV adaptado»).
- **Opciones avanzadas:** sus campos están en dos columnas.

### Interfaz más fluida, adaptable y legible
- **Contraste:** los botones deshabilitados («Guardar perfil», «Guardar CV», «Siguiente») eran casi invisibles, con un contraste de 1,04 a 1,4. Ahora se leen, con 3:1 o más. El texto secundario de la vacante seleccionada sube a 4,5:1 o más. Una medición automática de contraste pasa en todas las pantallas, en claro y oscuro.
- **Tamaños de ventana:** cajón lateral desde 1100px y riel de navegación entre 720 y 1099px. La ventana mínima baja de 900 a 720px.
  - **Vacantes:** muestra lista o detalle, con «Volver a la lista».
  - **Tablero:** su ficha es una hoja lateral (se cierra con Esc).
  - **Perfiles, Postular y Optimizador:** se apilan cuando falta espacio.
  - **Pantallas muy anchas:** el contenido no pasa de 1440px.
- **Animaciones:**
  - Al cambiar de pantalla, el contenido se funde y la píldora de la navegación se desliza al nuevo destino (View Transitions).
  - El detalle de vacante, las tarjetas, la cola y los avisos entran con un movimiento breve, y los botones responden al presionarlos.
  - Al cargar vacantes aparecen siluetas en lugar de «Cargando…».
  - Todo respeta «reducir movimiento» del sistema.
- **Detalles:**
  - Barras de desplazamiento con el estilo de la app en Linux.
  - La etiqueta del campo con foco se pinta en azul.
  - Vacantes vacía muestra un estado que guía al siguiente paso.
  - Los pasos de la Bienvenida y los recordatorios ya no bajan de 15px.

### Interfaz con Material Design 3
- Rediseño de toda la interfaz con Material Design 3 y una identidad propia: principal **azul talavera** (#1F50A0) y acento **cempasúchil**, reservado para la compatibilidad alta y las vacantes nuevas. Los tonos para claro y oscuro salen de `@material/material-color-utilities`, y el tema sigue al del sistema.
- Tipografía Atkinson Hyperlegible Next, incluida en la app y pensada para leerse bien con baja visión: 16px de base y 15px para el texto secundario. Los controles miden 40–44px.
- Cajón de navegación con iconos Material Symbols, en el mismo lugar y orden de siempre. También hay botones en píldora, campos con contorno, botones segmentados, estados vacíos que indican el siguiente paso y foco visible con teclado.
- **Medidor de compatibilidad** compartido (`Meter.svelte`): una píldora tonal con barra lineal. Aparece en Vacantes (lista y desglose), Postular, Tablero y Optimizador (CV actual → CV adaptado).
- Vacantes: etiqueta «Nueva» para lo encontrado en las últimas 24 horas. «Calcular similitud semántica» pasa a «Comparar mejor con IA», junto al conteo.
- Tablero: resumen en fichas («3 postuladas · 1 entrevista…»), filtros plegables y columnas que caben enteras.
- Lenguaje más claro: «Nivel de experiencia» en lugar de «Seniority», y «filtros automáticos» en lugar de «ATS», con una explicación breve.
- `PRODUCT.md` y `DESIGN.md` documentan el producto y el sistema visual.

### Instaladores
- `tauri.conf.json`: instaladores `.deb`, `.rpm`, AppImage, NSIS (`.exe`, español/inglés, sin permisos de administrador) y `.msi`, con descripción y categoría.
- `.github/workflows/release.yml`: genera los de Windows y Linux en GitHub al subir una etiqueta `v*`, como borrador de versión.

### Copia de seguridad
- Ajustes → «Copia de seguridad»: «Hacer copia ahora» crea `Documentos/FinderRemoteJob-respaldo-AAAA-MM-DD.zip` con la base (copia consistente con `VACUUM INTO`, sin cerrar la app) y la carpeta `cvs/`. Las claves de API no se incluyen (siguen en el llavero).
- «Restaurar una copia» valida el archivo, lo deja pendiente y reinicia la app; al arrancar se cambia la base antes de abrirla. Lo anterior queda como `finder.db.antes-de-restaurar` y `cvs.antes-de-restaurar`. Si el respaldo viene de otra carpeta, se corrigen las rutas de los CV generados.

### Asistente de bienvenida
- Primer uso (sin perfiles): la app abre `/bienvenida` con cuatro pasos: crear el perfil, importar el CV, escribir los puestos buscados y buscar vacantes. También se abre desde Perfiles → «Crear un perfil paso a paso».
- Antes de importar el CV comprueba si Ollama está abierto y si el modelo local está descargado (comando `ollama_status`, vía `/api/tags`), y explica cómo instalarlo.

### Simplificación para usuarios sin conocimientos técnicos
- Se quitan funciones que pedían configuración técnica o aportaban poco al objetivo (encontrar vacantes, adaptar el CV y darles seguimiento):
  - Fuentes Adzuna y Jooble (pedían registrarse como desarrollador; Jooble solo da 500 consultas por clave).
  - Lector de alertas por correo (pedía servidor IMAP y contraseña de aplicación). Se quitan sus dependencias (`async-imap`, `mail-parser`, `tokio-rustls`…).
  - Costo estimado (había que escribir precios por millón de tokens).
  - Carta de presentación (Optimizador, Tablero, Postular; comandos, `letter`, `guard::validate_paragraphs`, `Db::set_cover_letter` y el campo `Application::cover_letter`). La columna sigue en la base, sin uso.
  - Pantalla Estadísticas: su resumen (postuladas, entrevistas, ofertas y tasa de respuesta) pasa a una línea arriba del Tablero.
- Las migraciones de las fuentes quitadas se conservan (SQLite exige que sigan existiendo las ya aplicadas).
- Ajustes, nuevo diseño: «¿Qué IA adapta tus CV?» con tres opciones (En tu computadora, Claude, Gemini o ChatGPT). Gemini y ChatGPT traen la dirección y el modelo ya puestos (`gemini-3.8-flash`, `gpt-6.1-terra`, según su documentación de noviembre de 2026); solo se pega la clave, con el enlace para crearla. Si falta la clave, se avisa antes de guardar.
- Ollama, modelos, esfuerzo, proveedor compatible propio y «Empresas que sigues» pasan a «Opciones avanzadas» (cerrado).
- Textos en lenguaje simple: «En uso: Claude · …»; la barra lateral solo dice qué IA se usa, o «La IA no está lista: revisa Ajustes».
