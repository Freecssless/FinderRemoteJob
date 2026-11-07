//! Persistencia en SQLite con `sqlx`. Esquema en `migrations/` (plan §5.2).
//!
//! Las consultas usan `sqlx::query` en tiempo de ejecución (no las macros `query!`), para no
//! exigir una base de datos ni `DATABASE_URL` al compilar.

use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use finder_core::model::{
    Application, CvMaster, CvVersion, Id, Job, JobRequirements, Profile, RemotePref, Status,
    StatusChange,
};
use finder_core::pipeline::{self, ReminderDays, ReminderKind};
use finder_core::stats::StatInput;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteRow};
use sqlx::{Row, SqlitePool};

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("error de base de datos: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("error al migrar la base de datos: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("JSON inválido guardado en la base de datos: {0}")]
    Json(#[from] serde_json::Error),
    #[error("valor inválido en la columna {column}: «{value}»")]
    InvalidValue { column: &'static str, value: String },
    #[error("no existe: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, DbError>;

pub struct Db {
    pool: SqlitePool,
}

/// CV maestro guardado, con el texto del archivo original si se importó.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredCv {
    pub cv: CvMaster,
    pub original_file_text: Option<String>,
    pub updated_at: String,
}

/// Resultado de guardar una vacante: si ya existía (mismo `dedupe_hash`), se devuelve la existente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobInsert {
    pub id: Id,
    pub created: bool,
}

/// Consulta base de los recordatorios pendientes (macro para usarla en `concat!`: sqlx solo
/// acepta SQL fijo).
macro_rules! reminder_select {
    () => {
        "SELECT r.id, r.application_id, a.profile_id, r.kind, r.due_at,
                j.title AS job_title, j.company AS job_company
         FROM reminders r JOIN applications a ON a.id = r.application_id JOIN jobs j ON j.id = a.job_id
         WHERE r.done = 0"
    };
}

/// Recordatorio pendiente, con los datos de su vacante para mostrarlo o notificarlo.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Reminder {
    pub id: i64,
    pub application_id: Id,
    pub profile_id: Id,
    pub kind: ReminderKind,
    /// ISO-8601 (fecha y hora, o solo fecha para el próximo paso).
    pub due_at: String,
    pub job_title: String,
    pub company: Option<String>,
}

/// Filtros del tablero. `None` = sin filtrar.
#[derive(Debug, Clone, Default)]
pub struct BoardFilter {
    pub profile_id: Option<Id>,
    pub status: Option<Status>,
    pub source_id: Option<String>,
    /// Fecha mínima (ISO) de postulación, o de hallazgo si no te has postulado.
    pub from: Option<String>,
    pub to: Option<String>,
}

/// Fila del tablero: la postulación con los datos de su vacante.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct BoardRow {
    pub application: Application,
    pub job_title: String,
    pub company: Option<String>,
    pub source_id: String,
    pub url: String,
    pub found_at: String,
}

impl Db {
    /// Abre (o crea) la base en `path` y aplica las migraciones pendientes.
    pub async fn open(path: &Path) -> Result<Self> {
        let opts = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal);
        Self::connect(opts, 4).await
    }

    /// Base temporal en memoria, para pruebas.
    pub async fn open_in_memory() -> Result<Self> {
        let opts = SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);
        // Una sola conexión que nunca se cierra: cada conexión en memoria sería otra base vacía.
        Self::connect(opts, 1).await
    }

    async fn connect(opts: SqliteConnectOptions, max: u32) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(max)
            .min_connections(1)
            .idle_timeout(None)
            .max_lifetime(None)
            .connect_with(opts)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    /// Fecha y hora actual en UTC (ISO-8601), tomada de SQLite para no depender de otro crate.
    pub async fn now(&self) -> Result<String> {
        Ok(
            sqlx::query_scalar("SELECT strftime('%Y-%m-%dT%H:%M:%SZ', 'now')")
                .fetch_one(&self.pool)
                .await?,
        )
    }

    /// Nuevo id UUID v7 (ordenable por fecha de creación).
    pub fn new_id() -> Id {
        uuid::Uuid::now_v7().to_string()
    }

    // -----------------------------------------------------------------------
    // Perfiles
    // -----------------------------------------------------------------------

    pub async fn create_profile(
        &self,
        name: &str,
        target_titles: &[String],
        remote_pref: RemotePref,
        locations: &[String],
    ) -> Result<Profile> {
        let p = Profile {
            id: Self::new_id(),
            name: name.to_string(),
            target_titles: target_titles.to_vec(),
            remote_pref,
            locations: locations.to_vec(),
            created_at: self.now().await?,
        };
        sqlx::query(
            "INSERT INTO profiles (id, name, target_titles, remote_pref, locations, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&p.id)
        .bind(&p.name)
        .bind(serde_json::to_string(&p.target_titles)?)
        .bind(p.remote_pref.as_str())
        .bind(serde_json::to_string(&p.locations)?)
        .bind(&p.created_at)
        .execute(&self.pool)
        .await?;
        Ok(p)
    }

    pub async fn update_profile(&self, p: &Profile) -> Result<()> {
        let r = sqlx::query(
            "UPDATE profiles SET name = ?, target_titles = ?, remote_pref = ?, locations = ? WHERE id = ?",
        )
        .bind(&p.name)
        .bind(serde_json::to_string(&p.target_titles)?)
        .bind(p.remote_pref.as_str())
        .bind(serde_json::to_string(&p.locations)?)
        .bind(&p.id)
        .execute(&self.pool)
        .await?;
        if r.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("perfil {}", p.id)));
        }
        Ok(())
    }

    pub async fn get_profile(&self, id: &str) -> Result<Option<Profile>> {
        sqlx::query("SELECT * FROM profiles WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .map(|r| profile_from_row(&r))
            .transpose()
    }

    pub async fn list_profiles(&self) -> Result<Vec<Profile>> {
        sqlx::query("SELECT * FROM profiles ORDER BY created_at")
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(profile_from_row)
            .collect()
    }

    /// Borra el perfil **y todo lo suyo**: CV maestro, versiones de CV y postulaciones
    /// (con su historial y recordatorios). Las vacantes se conservan: pueden servir a otros perfiles.
    /// La interfaz debe pedir confirmación antes de llamar a esto.
    pub async fn delete_profile(&self, id: &str) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        // status_history y reminders se borran en cascada con applications.
        sqlx::query("DELETE FROM applications WHERE profile_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM cv_versions WHERE profile_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        let r = sqlx::query("DELETE FROM profiles WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        if r.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("perfil {id}")));
        }
        tx.commit().await?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // CV maestro
    // -----------------------------------------------------------------------

    pub async fn save_cv_master(
        &self,
        profile_id: &str,
        cv: &CvMaster,
        original_file_text: Option<&str>,
    ) -> Result<()> {
        // COALESCE: guardar cambios del formulario no borra el texto del archivo importado.
        sqlx::query(
            "INSERT INTO cv_master (profile_id, data_json, original_file_text, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(profile_id) DO UPDATE SET
               data_json = ?2,
               original_file_text = COALESCE(?3, original_file_text),
               updated_at = ?4",
        )
        .bind(profile_id)
        .bind(serde_json::to_string(cv)?)
        .bind(original_file_text)
        .bind(self.now().await?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_cv_master(&self, profile_id: &str) -> Result<Option<StoredCv>> {
        let Some(r) = sqlx::query("SELECT * FROM cv_master WHERE profile_id = ?")
            .bind(profile_id)
            .fetch_optional(&self.pool)
            .await?
        else {
            return Ok(None);
        };
        Ok(Some(StoredCv {
            cv: serde_json::from_str(&r.try_get::<String, _>("data_json")?)?,
            original_file_text: r.try_get("original_file_text")?,
            updated_at: r.try_get("updated_at")?,
        }))
    }

    // -----------------------------------------------------------------------
    // Fuentes
    // -----------------------------------------------------------------------

    /// ¿Ya pasaron `min_minutes` desde la última consulta a la fuente? (o nunca se consultó)
    pub async fn source_due(&self, source_id: &str, min_minutes: u32) -> Result<bool> {
        let due: Option<bool> = sqlx::query_scalar(
            "SELECT last_sync_at IS NULL
                    OR last_sync_at <= strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-' || ?2 || ' minutes')
             FROM sources WHERE id = ?1",
        )
        .bind(source_id)
        .bind(min_minutes)
        .fetch_optional(&self.pool)
        .await?;
        due.ok_or_else(|| DbError::NotFound(format!("fuente {source_id}")))
    }

    pub async fn mark_source_synced(&self, source_id: &str) -> Result<()> {
        sqlx::query("UPDATE sources SET last_sync_at = ? WHERE id = ?")
            .bind(self.now().await?)
            .bind(source_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Vacantes
    // -----------------------------------------------------------------------

    /// Guarda la vacante tal como viene (el llamador fija `id`, `found_at` y `dedupe_hash`).
    /// Si ya existe una con el mismo `dedupe_hash`, no la duplica y devuelve la existente.
    pub async fn insert_job(&self, job: &Job) -> Result<JobInsert> {
        let requirements = job
            .requirements
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let r = sqlx::query(
            "INSERT INTO jobs (id, source_id, external_id, url, company, title, location, remote_type,
                               salary_min, salary_max, currency, description, language,
                               requirements_json, posted_at, found_at, dedupe_hash)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(dedupe_hash) DO NOTHING",
        )
        .bind(&job.id)
        .bind(&job.source_id)
        .bind(&job.external_id)
        .bind(&job.url)
        .bind(&job.company)
        .bind(&job.title)
        .bind(&job.location)
        .bind(&job.remote_type)
        .bind(job.salary_min)
        .bind(job.salary_max)
        .bind(&job.currency)
        .bind(&job.description)
        .bind(&job.language)
        .bind(requirements)
        .bind(&job.posted_at)
        .bind(&job.found_at)
        .bind(&job.dedupe_hash)
        .execute(&self.pool)
        .await?;
        if r.rows_affected() == 1 {
            return Ok(JobInsert {
                id: job.id.clone(),
                created: true,
            });
        }
        let id = sqlx::query_scalar("SELECT id FROM jobs WHERE dedupe_hash = ?")
            .bind(&job.dedupe_hash)
            .fetch_one(&self.pool)
            .await?;
        Ok(JobInsert { id, created: false })
    }

    /// Guarda los requisitos extraídos (diccionario o LLM) de una vacante.
    pub async fn set_job_requirements(&self, job_id: &str, req: &JobRequirements) -> Result<()> {
        sqlx::query("UPDATE jobs SET requirements_json = ? WHERE id = ?")
            .bind(serde_json::to_string(req)?)
            .bind(job_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_job(&self, id: &str) -> Result<Option<Job>> {
        sqlx::query("SELECT * FROM jobs WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .map(|r| job_from_row(&r))
            .transpose()
    }

    /// Vacantes más recientes primero.
    /// Fecha y hora ISO de hace `days` días, del reloj de SQLite (como el resto de fechas).
    pub async fn days_ago(&self, days: u32) -> Result<String> {
        Ok(
            sqlx::query_scalar("SELECT strftime('%Y-%m-%dT%H:%M:%SZ', 'now', ?)")
                .bind(format!("-{days} days"))
                .fetch_one(&self.pool)
                .await?,
        )
    }

    /// Vacantes encontradas en las últimas `hours` horas (aviso «nuevas» del menú).
    pub async fn count_jobs_found_within(&self, hours: u32) -> Result<i64> {
        Ok(sqlx::query_scalar(
            "SELECT COUNT(*) FROM jobs WHERE found_at >= strftime('%Y-%m-%dT%H:%M:%SZ', 'now', ?)",
        )
        .bind(format!("-{hours} hours"))
        .fetch_one(&self.pool)
        .await?)
    }

    pub async fn list_jobs(&self, limit: u32) -> Result<Vec<Job>> {
        sqlx::query("SELECT * FROM jobs ORDER BY found_at DESC, id DESC LIMIT ?")
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(job_from_row)
            .collect()
    }

    /// Búsqueda de texto completo (título, empresa y descripción), por relevancia.
    /// Lo que escribe el usuario se trata como palabras sueltas, nunca como sintaxis FTS.
    pub async fn search_jobs(&self, query: &str, limit: u32) -> Result<Vec<Job>> {
        let Some(fts) = fts_query(query) else {
            return self.list_jobs(limit).await;
        };
        sqlx::query(
            "SELECT jobs.* FROM jobs_fts JOIN jobs ON jobs.id = jobs_fts.job_id
             WHERE jobs_fts MATCH ? ORDER BY rank LIMIT ?",
        )
        .bind(fts)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(job_from_row)
        .collect()
    }

    // -----------------------------------------------------------------------
    // Postulaciones y tablero
    // -----------------------------------------------------------------------

    /// Registra la vacante en el tablero del perfil como "Encontrada" con su compatibilidad.
    /// Si ya estaba, solo actualiza el puntaje (no toca el estado).
    pub async fn upsert_found(
        &self,
        job_id: &str,
        profile_id: &str,
        match_score: u8,
    ) -> Result<Id> {
        let now = self.now().await?;
        let id = sqlx::query_scalar(
            "INSERT INTO applications (id, job_id, profile_id, status, match_score, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(job_id, profile_id) DO UPDATE SET match_score = ?5
             RETURNING id",
        )
        .bind(Self::new_id())
        .bind(job_id)
        .bind(profile_id)
        .bind(Status::Encontrada.as_str())
        .bind(match_score)
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn get_application(&self, id: &str) -> Result<Option<Application>> {
        sqlx::query("SELECT * FROM applications WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .map(|r| application_from_row(&r))
            .transpose()
    }

    /// Cambia el estado (reglas en `finder_core::pipeline`) y lo anota en el historial.
    /// Al cambiar, se cierran los recordatorios pendientes del estado anterior (salvo el del próximo
    /// paso) y se crean los del nuevo (`pipeline::reminders_on`).
    pub async fn set_status(&self, application_id: &str, to: Status) -> Result<Application> {
        let now = self.now().await?;
        let days = self.reminder_days().await?;
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query("SELECT * FROM applications WHERE id = ?")
            .bind(application_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| DbError::NotFound(format!("postulación {application_id}")))?;
        let mut app = application_from_row(&row)?;
        let from = app.status;
        if !pipeline::transition(&mut app, to, &now) {
            return Ok(app);
        }
        sqlx::query(
            "UPDATE applications SET status = ?, applied_at = ?, first_response_at = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(app.status.as_str())
        .bind(&app.applied_at)
        .bind(&app.first_response_at)
        .bind(&app.updated_at)
        .bind(&app.id)
        .execute(&mut *tx)
        .await?;
        sqlx::query("INSERT INTO status_history (application_id, from_status, to_status, at) VALUES (?, ?, ?, ?)")
            .bind(&app.id)
            .bind(from.as_str())
            .bind(to.as_str())
            .bind(&now)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "UPDATE reminders SET done = 1 WHERE application_id = ? AND done = 0 AND kind != ?",
        )
        .bind(&app.id)
        .bind(ReminderKind::ProximoPaso.as_str())
        .execute(&mut *tx)
        .await?;
        for (kind, hours) in pipeline::reminders_on(to, days) {
            sqlx::query(
                "INSERT INTO reminders (application_id, due_at, kind)
                 VALUES (?, strftime('%Y-%m-%dT%H:%M:%SZ', ?, printf('+%d hours', ?)), ?)",
            )
            .bind(&app.id)
            .bind(&now)
            .bind(hours)
            .bind(kind.as_str())
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(app)
    }

    /// Notas y próximo paso, tal como los escribe el usuario (`None` = vaciar el campo).
    /// La fecha del próximo paso queda como recordatorio (reemplaza al pendiente anterior).
    pub async fn update_application_notes(
        &self,
        application_id: &str,
        notes: Option<&str>,
        next_step: Option<&str>,
        next_step_at: Option<&str>,
    ) -> Result<()> {
        let now = self.now().await?;
        let mut tx = self.pool.begin().await?;
        let r = sqlx::query(
            "UPDATE applications SET notes = ?, next_step = ?, next_step_at = ?, updated_at = ? WHERE id = ?",
        )
        .bind(notes)
        .bind(next_step)
        .bind(next_step_at)
        .bind(now)
        .bind(application_id)
        .execute(&mut *tx)
        .await?;
        if r.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("postulación {application_id}")));
        }
        let kind = ReminderKind::ProximoPaso.as_str();
        sqlx::query("DELETE FROM reminders WHERE application_id = ? AND kind = ? AND done = 0")
            .bind(application_id)
            .bind(kind)
            .execute(&mut *tx)
            .await?;
        if let Some(at) = next_step_at {
            sqlx::query("INSERT INTO reminders (application_id, due_at, kind) VALUES (?, ?, ?)")
                .bind(application_id)
                .bind(at)
                .bind(kind)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Postulaciones con fecha de postulación, para `finder_core::stats::compute` (todos los perfiles
    /// si `profile_id` es `None`). "Llegó a entrevista/oferta" sale del historial, no del estado actual.
    pub async fn stat_inputs(&self, profile_id: Option<&str>) -> Result<Vec<StatInput>> {
        let rows = sqlx::query(
            "SELECT a.applied_at, a.first_response_at, a.match_score, a.profile_id, a.status,
                    j.source_id, v.variant, v.ats_after,
                    EXISTS (SELECT 1 FROM status_history h WHERE h.application_id = a.id
                            AND h.to_status IN ('Entrevista', 'Oferta')) AS reached_interview,
                    EXISTS (SELECT 1 FROM status_history h WHERE h.application_id = a.id
                            AND h.to_status = 'Oferta') AS reached_offer
             FROM applications a
             JOIN jobs j ON j.id = a.job_id
             LEFT JOIN cv_versions v ON v.id = a.cv_version_id
             WHERE a.applied_at IS NOT NULL AND (?1 IS NULL OR a.profile_id = ?1)",
        )
        .bind(profile_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter()
            .map(|r| {
                Ok(StatInput {
                    applied_at: r.try_get("applied_at")?,
                    first_response_at: r.try_get("first_response_at")?,
                    match_score: r.try_get("match_score")?,
                    source_id: r.try_get("source_id")?,
                    profile_id: r.try_get("profile_id")?,
                    variant: r.try_get("variant")?,
                    ats_after: r.try_get("ats_after")?,
                    reached_interview: r.try_get("reached_interview")?,
                    reached_offer: r.try_get("reached_offer")?,
                })
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Recordatorios (plan §5.3)
    // -----------------------------------------------------------------------

    /// Pendientes que ya vencieron (todos los perfiles si `profile_id` es `None`), más antiguos primero.
    pub async fn list_due_reminders(&self, profile_id: Option<&str>) -> Result<Vec<Reminder>> {
        let rows = sqlx::query(concat!(
            reminder_select!(),
            " AND r.due_at <= ?1 AND (?2 IS NULL OR a.profile_id = ?2) ORDER BY r.due_at, r.id"
        ))
        .bind(self.now().await?)
        .bind(profile_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(reminder_from_row).collect()
    }

    /// Pendientes de una postulación (vencidos o no), en orden de fecha.
    pub async fn pending_reminders(&self, application_id: &str) -> Result<Vec<Reminder>> {
        let rows = sqlx::query(concat!(
            reminder_select!(),
            " AND r.application_id = ? ORDER BY r.due_at, r.id"
        ))
        .bind(application_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(reminder_from_row).collect()
    }

    pub async fn complete_reminder(&self, id: i64) -> Result<()> {
        sqlx::query("UPDATE reminders SET done = 1 WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Ajustes cuyo nombre empieza con `prefix` (sin el prefijo).
    pub async fn settings_with_prefix(&self, prefix: &str) -> Result<Vec<(String, String)>> {
        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT key, value FROM settings WHERE substr(key, 1, length(?1)) = ?1")
                .bind(prefix)
                .fetch_all(&self.pool)
                .await?;
        Ok(rows
            .into_iter()
            .map(|(k, v)| (k[prefix.len()..].to_string(), v))
            .collect())
    }

    /// Embeddings guardados de un modelo cuyos dueños empiezan con `prefix` → (hash del texto, vector).
    pub async fn embeddings(
        &self,
        model: &str,
        prefix: &str,
    ) -> Result<HashMap<String, (String, Vec<f32>)>> {
        let rows: Vec<(String, String, Vec<u8>)> = sqlx::query_as(
            "SELECT owner, text_hash, vector FROM embeddings
             WHERE model = ?1 AND substr(owner, 1, length(?2)) = ?2",
        )
        .bind(model)
        .bind(prefix)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(owner, hash, bytes)| {
                let v = bytes
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|c| f32::from_le_bytes(*c))
                    .collect();
                (owner, (hash, v))
            })
            .collect())
    }

    pub async fn put_embedding(
        &self,
        owner: &str,
        model: &str,
        text_hash: &str,
        vector: &[f32],
    ) -> Result<()> {
        let bytes: Vec<u8> = vector.iter().flat_map(|x| x.to_le_bytes()).collect();
        sqlx::query(
            "INSERT INTO embeddings (owner, model, text_hash, vector) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(owner, model) DO UPDATE SET text_hash = ?3, vector = ?4",
        )
        .bind(owner)
        .bind(model)
        .bind(text_hash)
        .bind(bytes)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn setting(&self, key: &str) -> Result<Option<String>> {
        Ok(
            sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
                .bind(key)
                .fetch_optional(&self.pool)
                .await?,
        )
    }

    /// Copia consistente de toda la base en `dest` (que no debe existir), sin cerrar la app.
    pub async fn backup_to(&self, dest: &Path) -> Result<()> {
        sqlx::query("VACUUM INTO ?")
            .bind(dest.to_string_lossy().as_ref())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Tras restaurar un respaldo de otra carpeta: las rutas de los CV generados apuntan a la nueva.
    pub async fn rebase_cv_paths(&self, old_dir: &str, new_dir: &str) -> Result<()> {
        sqlx::query(
            "UPDATE cv_versions SET
               pdf_path = CASE WHEN substr(pdf_path, 1, length(?1)) = ?1
                          THEN ?2 || substr(pdf_path, length(?1) + 1) ELSE pdf_path END,
               docx_path = CASE WHEN substr(docx_path, 1, length(?1)) = ?1
                           THEN ?2 || substr(docx_path, length(?1) + 1) ELSE docx_path END",
        )
        .bind(old_dir)
        .bind(new_dir)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Guarda un ajuste; `None` lo borra.
    pub async fn set_setting(&self, key: &str, value: Option<&str>) -> Result<()> {
        match value {
            Some(v) => sqlx::query(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = ?2",
            )
            .bind(key)
            .bind(v),
            None => sqlx::query("DELETE FROM settings WHERE key = ?").bind(key),
        }
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Plazos de los recordatorios (tabla `settings`; por defecto 7 y 21 días).
    pub async fn reminder_days(&self) -> Result<ReminderDays> {
        let get = |key: &'static str| {
            sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = ?")
                .bind(key)
                .fetch_optional(&self.pool)
        };
        let d = ReminderDays::default();
        let parse = |v: Option<String>, def: u32| v.and_then(|v| v.parse().ok()).unwrap_or(def);
        Ok(ReminderDays {
            follow_up: parse(get("reminder_follow_up_days").await?, d.follow_up),
            no_response: parse(get("reminder_no_response_days").await?, d.no_response),
        })
    }

    pub async fn set_reminder_days(&self, days: ReminderDays) -> Result<()> {
        for (key, value) in [
            ("reminder_follow_up_days", days.follow_up),
            ("reminder_no_response_days", days.no_response),
        ] {
            sqlx::query(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = ?2",
            )
            .bind(key)
            .bind(value.to_string())
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    /// Vincula la versión de CV enviada a esta postulación.
    pub async fn attach_cv_version(&self, application_id: &str, cv_version_id: &str) -> Result<()> {
        let r =
            sqlx::query("UPDATE applications SET cv_version_id = ?, updated_at = ? WHERE id = ?")
                .bind(cv_version_id)
                .bind(self.now().await?)
                .bind(application_id)
                .execute(&self.pool)
                .await?;
        if r.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("postulación {application_id}")));
        }
        Ok(())
    }

    /// Postulaciones para el tablero o la lista, con los filtros indicados.
    /// Orden: mayor compatibilidad primero, y después las más recientes.
    pub async fn list_board(&self, f: &BoardFilter) -> Result<Vec<BoardRow>> {
        // Cada filtro es "(? IS NULL OR condición)": un solo SQL fijo, sin concatenar texto del usuario.
        let rows = sqlx::query(
            "SELECT a.*, j.title AS job_title, j.company AS job_company, j.source_id AS job_source,
                    j.url AS job_url, j.found_at AS job_found_at
             FROM applications a JOIN jobs j ON j.id = a.job_id
             WHERE (?1 IS NULL OR a.profile_id = ?1)
               AND (?2 IS NULL OR a.status = ?2)
               AND (?3 IS NULL OR j.source_id = ?3)
               AND (?4 IS NULL OR COALESCE(a.applied_at, j.found_at) >= ?4)
               AND (?5 IS NULL OR COALESCE(a.applied_at, j.found_at) <= ?5)
             ORDER BY a.match_score DESC NULLS LAST, a.updated_at DESC",
        )
        .bind(&f.profile_id)
        .bind(f.status.map(Status::as_str))
        .bind(&f.source_id)
        .bind(&f.from)
        .bind(&f.to)
        .fetch_all(&self.pool)
        .await?;
        rows.iter()
            .map(|r| {
                Ok(BoardRow {
                    application: application_from_row(r)?,
                    job_title: r.try_get("job_title")?,
                    company: r.try_get("job_company")?,
                    source_id: r.try_get("job_source")?,
                    url: r.try_get("job_url")?,
                    found_at: r.try_get("job_found_at")?,
                })
            })
            .collect()
    }

    pub async fn status_history(&self, application_id: &str) -> Result<Vec<StatusChange>> {
        sqlx::query("SELECT * FROM status_history WHERE application_id = ? ORDER BY id")
            .bind(application_id)
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(|r| {
                Ok(StatusChange {
                    from: r
                        .try_get::<Option<String>, _>("from_status")?
                        .map(|s| parse_status("from_status", &s))
                        .transpose()?,
                    to: parse_status("to_status", &r.try_get::<String, _>("to_status")?)?,
                    at: r.try_get("at")?,
                })
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Versiones de CV
    // -----------------------------------------------------------------------

    pub async fn insert_cv_version(&self, v: &CvVersion) -> Result<()> {
        sqlx::query(
            "INSERT INTO cv_versions (id, profile_id, job_id, variant, content_json, master_hash,
                                      ats_before, ats_after, missing_json, pdf_path, docx_path, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&v.id)
        .bind(&v.profile_id)
        .bind(&v.job_id)
        .bind(&v.variant)
        .bind(&v.content_json)
        .bind(&v.master_hash)
        .bind(v.ats_before)
        .bind(v.ats_after)
        .bind(serde_json::to_string(&v.missing)?)
        .bind(&v.pdf_path)
        .bind(&v.docx_path)
        .bind(&v.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_cv_version(&self, id: &str) -> Result<Option<CvVersion>> {
        sqlx::query("SELECT * FROM cv_versions WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .map(|r| cv_version_from_row(&r))
            .transpose()
    }

    /// Versiones de un perfil (opcionalmente solo las de una vacante), más recientes primero.
    pub async fn list_cv_versions(
        &self,
        profile_id: &str,
        job_id: Option<&str>,
    ) -> Result<Vec<CvVersion>> {
        sqlx::query(
            "SELECT * FROM cv_versions WHERE profile_id = ?1 AND (?2 IS NULL OR job_id = ?2)
             ORDER BY created_at DESC, id DESC",
        )
        .bind(profile_id)
        .bind(job_id)
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(cv_version_from_row)
        .collect()
    }
}

// ---------------------------------------------------------------------------
// Conversión de filas
// ---------------------------------------------------------------------------

fn reminder_from_row(r: &SqliteRow) -> Result<Reminder> {
    let kind: String = r.try_get("kind")?;
    Ok(Reminder {
        id: r.try_get("id")?,
        application_id: r.try_get("application_id")?,
        profile_id: r.try_get("profile_id")?,
        kind: ReminderKind::parse(&kind).ok_or_else(|| DbError::InvalidValue {
            column: "kind",
            value: kind.clone(),
        })?,
        due_at: r.try_get("due_at")?,
        job_title: r.try_get("job_title")?,
        company: r.try_get("job_company")?,
    })
}

fn parse_status(column: &'static str, value: &str) -> Result<Status> {
    Status::parse(value).ok_or_else(|| DbError::InvalidValue {
        column,
        value: value.to_string(),
    })
}

fn profile_from_row(r: &SqliteRow) -> Result<Profile> {
    let pref: String = r.try_get("remote_pref")?;
    Ok(Profile {
        id: r.try_get("id")?,
        name: r.try_get("name")?,
        target_titles: serde_json::from_str(&r.try_get::<String, _>("target_titles")?)?,
        remote_pref: RemotePref::parse(&pref).ok_or(DbError::InvalidValue {
            column: "remote_pref",
            value: pref,
        })?,
        locations: serde_json::from_str(&r.try_get::<String, _>("locations")?)?,
        created_at: r.try_get("created_at")?,
    })
}

fn job_from_row(r: &SqliteRow) -> Result<Job> {
    Ok(Job {
        id: r.try_get("id")?,
        source_id: r.try_get("source_id")?,
        external_id: r.try_get("external_id")?,
        url: r.try_get("url")?,
        company: r.try_get("company")?,
        title: r.try_get("title")?,
        location: r.try_get("location")?,
        remote_type: r.try_get("remote_type")?,
        salary_min: r.try_get("salary_min")?,
        salary_max: r.try_get("salary_max")?,
        currency: r.try_get("currency")?,
        description: r.try_get("description")?,
        language: r.try_get("language")?,
        requirements: r
            .try_get::<Option<String>, _>("requirements_json")?
            .map(|s| serde_json::from_str(&s))
            .transpose()?,
        posted_at: r.try_get("posted_at")?,
        found_at: r.try_get("found_at")?,
        dedupe_hash: r.try_get("dedupe_hash")?,
    })
}

fn application_from_row(r: &SqliteRow) -> Result<Application> {
    Ok(Application {
        id: r.try_get("id")?,
        job_id: r.try_get("job_id")?,
        profile_id: r.try_get("profile_id")?,
        status: parse_status("status", &r.try_get::<String, _>("status")?)?,
        match_score: r.try_get("match_score")?,
        cv_version_id: r.try_get("cv_version_id")?,
        applied_at: r.try_get("applied_at")?,
        first_response_at: r.try_get("first_response_at")?,
        next_step: r.try_get("next_step")?,
        next_step_at: r.try_get("next_step_at")?,
        notes: r.try_get("notes")?,
        updated_at: r.try_get("updated_at")?,
    })
}

fn cv_version_from_row(r: &SqliteRow) -> Result<CvVersion> {
    Ok(CvVersion {
        id: r.try_get("id")?,
        profile_id: r.try_get("profile_id")?,
        job_id: r.try_get("job_id")?,
        variant: r.try_get("variant")?,
        content_json: r.try_get("content_json")?,
        master_hash: r.try_get("master_hash")?,
        ats_before: r.try_get("ats_before")?,
        ats_after: r.try_get("ats_after")?,
        missing: r
            .try_get::<Option<String>, _>("missing_json")?
            .map(|s| serde_json::from_str(&s))
            .transpose()?
            .unwrap_or_default(),
        pdf_path: r.try_get("pdf_path")?,
        docx_path: r.try_get("docx_path")?,
        created_at: r.try_get("created_at")?,
    })
}

/// Convierte lo que escribe el usuario en una consulta FTS5 segura: cada palabra entre comillas
/// y con prefijo (`"edit"*` encuentra "editor"). `None` si no hay palabras.
fn fts_query(input: &str) -> Option<String> {
    let terms: Vec<String> = input
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
        .map(|w| format!("\"{}\"*", w.replace('"', "\"\"")))
        .collect();
    (!terms.is_empty()).then(|| terms.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use finder_core::model::dedupe_key;

    type R = std::result::Result<(), Box<dyn std::error::Error>>;

    fn job(id: &str, title: &str, company: &str, description: &str) -> Job {
        Job {
            id: id.into(),
            source_id: "manual".into(),
            external_id: None,
            url: format!("https://example.com/{id}"),
            company: Some(company.into()),
            title: title.into(),
            location: None,
            remote_type: Some("remote".into()),
            salary_min: None,
            salary_max: None,
            currency: None,
            description: description.into(),
            language: Some("es".into()),
            requirements: None,
            posted_at: None,
            found_at: "2026-11-05T10:00:00Z".into(),
            dedupe_hash: dedupe_key(Some(company), title, None),
        }
    }

    async fn setup() -> Result<(Db, Profile, Id)> {
        let db = Db::open_in_memory().await?;
        let p = db
            .create_profile(
                "Editor de video",
                &["Editor de video".into()],
                RemotePref::SoloRemoto,
                &[],
            )
            .await?;
        let j = db
            .insert_job(&job(
                "j1",
                "Editor de Video",
                "Agencia X",
                "Edición en Premiere y After Effects",
            ))
            .await?;
        Ok((db, p, j.id))
    }

    #[tokio::test]
    async fn cuenta_solo_las_vacantes_recientes() -> R {
        let (db, _, _) = setup().await?; // su vacante es de 2026-11-05
        assert_eq!(db.count_jobs_found_within(24).await?, 0);
        let mut nueva = job("j2", "Motion designer", "Estudio Y", "After Effects");
        nueva.found_at = db.now().await?;
        db.insert_job(&nueva).await?;
        assert_eq!(db.count_jobs_found_within(24).await?, 1);
        Ok(())
    }

    #[tokio::test]
    async fn perfiles_crear_editar_listar_y_borrar() -> R {
        let db = Db::open_in_memory().await?;
        let mut a = db
            .create_profile("Editor de video", &[], RemotePref::SoloRemoto, &[])
            .await?;
        let b = db
            .create_profile(
                "Programador Jr.",
                &["Frontend".into()],
                RemotePref::Cualquiera,
                &[],
            )
            .await?;

        a.name = "Editor y motion".into();
        a.locations = vec!["CDMX".into()];
        db.update_profile(&a).await?;
        assert_eq!(db.get_profile(&a.id).await?, Some(a.clone()));
        assert_eq!(db.list_profiles().await?, [a.clone(), b.clone()]);

        db.delete_profile(&a.id).await?;
        assert_eq!(db.list_profiles().await?, [b]);
        assert!(matches!(
            db.delete_profile(&a.id).await,
            Err(DbError::NotFound(_))
        ));
        Ok(())
    }

    #[tokio::test]
    async fn cv_maestro_se_guarda_y_conserva_el_texto_original() -> R {
        let (db, p, _) = setup().await?;
        let cv: CvMaster = serde_json::from_str(include_str!(
            "../../core/tests/fixtures/cv_editor_video.json"
        ))?;
        db.save_cv_master(&p.id, &cv, Some("texto del PDF")).await?;

        let mut editado = cv.clone();
        editado.basics.headline = "Editor de video y motion".into();
        db.save_cv_master(&p.id, &editado, None).await?;

        let guardado = db.get_cv_master(&p.id).await?.ok_or("falta el CV")?;
        assert_eq!(guardado.cv, editado);
        assert_eq!(
            guardado.original_file_text.as_deref(),
            Some("texto del PDF")
        );
        Ok(())
    }

    #[tokio::test]
    async fn vacante_duplicada_no_se_inserta_dos_veces() -> R {
        let (db, _, first) = setup().await?;
        let again = db
            .insert_job(&job(
                "j2",
                "Editor de vídeo",
                "AGENCIA X",
                "Otro texto, misma vacante",
            ))
            .await?;
        assert_eq!(
            again,
            JobInsert {
                id: first.clone(),
                created: false
            }
        );
        assert_eq!(db.list_jobs(10).await?.len(), 1);

        let req = JobRequirements {
            years_min: Some(3.0),
            ..Default::default()
        };
        db.set_job_requirements(&first, &req).await?;
        assert_eq!(
            db.get_job(&first).await?.ok_or("falta")?.requirements,
            Some(req)
        );
        Ok(())
    }

    #[tokio::test]
    async fn busqueda_de_texto_completo_es_segura() -> R {
        let (db, _, _) = setup().await?;
        db.insert_job(&job(
            "j2",
            "Frontend Developer",
            "Startup",
            "React y TypeScript",
        ))
        .await?;

        let r = db.search_jobs("premiere", 10).await?;
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, "j1");
        assert_eq!(
            db.search_jobs("edit", 10).await?.len(),
            1,
            "busca por prefijo"
        );
        // Sintaxis FTS o comillas sueltas no rompen la consulta.
        // "AND" se busca como palabra (no como operador), por eso aquí no hay resultados.
        assert_eq!(db.search_jobs("\"react (", 10).await?.len(), 1);
        assert!(db.search_jobs("react AND (", 10).await?.is_empty());
        assert_eq!(
            db.search_jobs("   ", 10).await?.len(),
            2,
            "vacío = recientes"
        );
        Ok(())
    }

    #[tokio::test]
    async fn datos_para_estadisticas() -> R {
        let (db, p, job_id) = setup().await?;
        let app_id = db.upsert_found(&job_id, &p.id, 70).await?;
        assert!(
            db.stat_inputs(None).await?.is_empty(),
            "sin postular no cuenta"
        );
        db.set_status(&app_id, Status::Postulada).await?;
        db.set_status(&app_id, Status::Entrevista).await?;
        db.set_status(&app_id, Status::Rechazada).await?;
        let rows = db.stat_inputs(Some(&p.id)).await?;
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert!(r.reached_interview && !r.reached_offer, "{r:?}");
        assert!(r.first_response_at.is_some());
        assert_eq!(
            (r.match_score, r.source_id.as_str(), r.variant.as_deref()),
            (Some(70), "manual", None)
        );
        assert!(db.stat_inputs(Some("otro")).await?.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn ajustes_por_prefijo() -> R {
        let (db, _, _) = setup().await?;
        db.set_setting("llm.FRJ_WRITER", Some("anthropic")).await?;
        db.set_setting("llm.FRJ_LOCAL_MODEL", Some("qwen3:8b"))
            .await?;
        db.set_setting("llm.FRJ_LOCAL_MODEL", Some("qwen3:4b"))
            .await?;
        db.set_setting("otra_cosa", Some("x")).await?;
        let mut got = db.settings_with_prefix("llm.").await?;
        got.sort();
        assert_eq!(
            got,
            [
                ("FRJ_LOCAL_MODEL".to_string(), "qwen3:4b".to_string()),
                ("FRJ_WRITER".into(), "anthropic".into())
            ]
        );
        db.set_setting("llm.FRJ_WRITER", None).await?;
        assert_eq!(db.settings_with_prefix("llm.").await?.len(), 1);
        assert!(db.setting("llm.no_existe").await?.is_none());
        db.put_embedding("job:1", "bge-m3", "h1", &[0.5, -1.0])
            .await?;
        db.put_embedding("job:1", "bge-m3", "h2", &[0.25, 2.0])
            .await?;
        db.put_embedding("cv:p", "bge-m3", "h3", &[1.0]).await?;
        let jobs = db.embeddings("bge-m3", "job:").await?;
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs["job:1"], ("h2".to_string(), vec![0.25, 2.0]));
        assert!(db.embeddings("otro-modelo", "job:").await?.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn recordatorios_por_estado_y_proximo_paso() -> R {
        let (db, p, job_id) = setup().await?;
        let app_id = db.upsert_found(&job_id, &p.id, 70).await?;
        db.set_reminder_days(ReminderDays {
            follow_up: 5,
            no_response: 30,
        })
        .await?;
        assert_eq!(db.reminder_days().await?.follow_up, 5);

        let app = db.set_status(&app_id, Status::Postulada).await?;
        let applied = app.applied_at.clone().unwrap_or_default();
        let r = db.pending_reminders(&app_id).await?;
        let kinds: Vec<_> = r.iter().map(|r| r.kind).collect();
        assert_eq!(
            kinds,
            [ReminderKind::Seguimiento, ReminderKind::SinRespuesta]
        );
        assert!(r[0].due_at > applied && r[0].due_at < r[1].due_at);
        assert!(
            db.list_due_reminders(None).await?.is_empty(),
            "aún no vencen"
        );

        // Al entrar a Entrevista, los de la postulación ya no aplican.
        db.set_status(&app_id, Status::Entrevista).await?;
        let kinds: Vec<_> = db
            .pending_reminders(&app_id)
            .await?
            .iter()
            .map(|r| r.kind)
            .collect();
        assert_eq!(kinds, [ReminderKind::Agradecimiento]);

        // El próximo paso con fecha es un recordatorio; cambiar la fecha lo reemplaza.
        db.update_application_notes(&app_id, None, Some("Llamar"), Some("2026-12-01"))
            .await?;
        db.update_application_notes(&app_id, None, Some("Llamar"), Some("2000-01-01"))
            .await?;
        let due = db.list_due_reminders(Some(&p.id)).await?;
        assert_eq!(due.len(), 1);
        assert_eq!(
            (due[0].kind, due[0].job_title.as_str()),
            (ReminderKind::ProximoPaso, "Editor de Video")
        );
        assert_eq!(db.list_due_reminders(Some("otro")).await?.len(), 0);

        db.complete_reminder(due[0].id).await?;
        assert!(db.list_due_reminders(None).await?.is_empty());
        // Sin fecha: no queda recordatorio de próximo paso.
        db.update_application_notes(&app_id, None, Some("Llamar"), Some("2027-01-01"))
            .await?;
        db.update_application_notes(&app_id, None, None, None)
            .await?;
        assert!(
            db.pending_reminders(&app_id)
                .await?
                .iter()
                .all(|r| r.kind != ReminderKind::ProximoPaso)
        );
        Ok(())
    }

    #[tokio::test]
    async fn tablero_estados_historial_y_filtros() -> R {
        let (db, p, job_id) = setup().await?;
        let app_id = db.upsert_found(&job_id, &p.id, 71).await?;
        // Repetir solo actualiza el puntaje.
        assert_eq!(db.upsert_found(&job_id, &p.id, 75).await?, app_id);

        db.set_status(&app_id, Status::Postulada).await?;
        let app = db.set_status(&app_id, Status::Entrevista).await?;
        assert_eq!(app.match_score, Some(75));
        assert!(app.applied_at.is_some() && app.first_response_at.is_some());
        // Mismo estado: no se duplica el historial.
        db.set_status(&app_id, Status::Entrevista).await?;

        let h = db.status_history(&app_id).await?;
        let pasos: Vec<_> = h.iter().map(|c| (c.from, c.to)).collect();
        assert_eq!(
            pasos,
            [
                (Some(Status::Encontrada), Status::Postulada),
                (Some(Status::Postulada), Status::Entrevista)
            ]
        );

        db.update_application_notes(
            &app_id,
            Some("Entrevista el lunes"),
            Some("Preparar portafolio"),
            None,
        )
        .await?;
        let filtro = BoardFilter {
            profile_id: Some(p.id.clone()),
            status: Some(Status::Entrevista),
            ..Default::default()
        };
        let tablero = db.list_board(&filtro).await?;
        assert_eq!(tablero.len(), 1);
        assert_eq!(tablero[0].job_title, "Editor de Video");
        assert_eq!(
            tablero[0].application.notes.as_deref(),
            Some("Entrevista el lunes")
        );

        let otro = BoardFilter {
            status: Some(Status::Oferta),
            ..Default::default()
        };
        assert!(db.list_board(&otro).await?.is_empty());
        let fuente = BoardFilter {
            source_id: Some("remoteok".into()),
            ..Default::default()
        };
        assert!(db.list_board(&fuente).await?.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn version_de_cv_vinculada_a_la_postulacion() -> R {
        let (db, p, job_id) = setup().await?;
        let app_id = db.upsert_found(&job_id, &p.id, 70).await?;
        let v = CvVersion {
            id: Db::new_id(),
            profile_id: p.id.clone(),
            job_id: Some(job_id.clone()),
            variant: "clasica".into(),
            content_json: "{}".into(),
            master_hash: "abc".into(),
            ats_before: Some(67),
            ats_after: Some(84),
            missing: vec!["TikTok".into()],
            pdf_path: Some("/tmp/cv.pdf".into()),
            docx_path: None,
            created_at: db.now().await?,
        };
        db.insert_cv_version(&v).await?;
        db.attach_cv_version(&app_id, &v.id).await?;

        assert_eq!(db.get_cv_version(&v.id).await?, Some(v.clone()));
        assert_eq!(
            db.list_cv_versions(&p.id, Some(&job_id)).await?,
            std::slice::from_ref(&v)
        );
        let app = db.get_application(&app_id).await?.ok_or("falta")?;
        assert_eq!(app.cv_version_id, Some(v.id.clone()));

        // Restaurar en otra carpeta: solo cambian las rutas que empiezan con la vieja.
        db.rebase_cv_paths("/tmp", "/home/ana/datos").await?;
        db.rebase_cv_paths("/otra", "/nada").await?;
        let moved = db.get_cv_version(&v.id).await?.ok_or("falta")?;
        assert_eq!(moved.pdf_path.as_deref(), Some("/home/ana/datos/cv.pdf"));
        assert_eq!(moved.docx_path, None);

        // Borrar el perfil borra sus versiones y postulaciones, pero no la vacante.
        db.delete_profile(&p.id).await?;
        assert!(db.get_application(&app_id).await?.is_none());
        assert!(db.get_job(&job_id).await?.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn base_en_archivo_persiste_entre_aperturas() -> R {
        let dir = std::env::temp_dir().join(format!("finder-test-{}", Db::new_id()));
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("datos.db");
        {
            let db = Db::open(&path).await?;
            db.create_profile("Perfil", &[], RemotePref::SoloRemoto, &[])
                .await?;
        }
        let db = Db::open(&path).await?;
        assert_eq!(db.list_profiles().await?.len(), 1);
        let copy = dir.join("copia.db");
        db.backup_to(&copy).await?;
        drop(db);
        assert_eq!(Db::open(&copy).await?.list_profiles().await?.len(), 1);
        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[tokio::test]
    async fn limite_de_frecuencia_por_fuente() -> R {
        let db = Db::open_in_memory().await?;
        assert!(db.source_due("remotive", 360).await?, "nunca consultada");
        db.mark_source_synced("remotive").await?;
        assert!(!db.source_due("remotive", 360).await?, "recién consultada");
        assert!(db.source_due("remotive", 0).await?, "sin intervalo mínimo");
        assert!(matches!(
            db.source_due("inventada", 60).await,
            Err(DbError::NotFound(_))
        ));
        Ok(())
    }

    #[test]
    fn consulta_fts_escapa_la_entrada() {
        assert_eq!(
            fts_query("editor de video").as_deref(),
            Some("\"editor\"* \"de\"* \"video\"*")
        );
        assert_eq!(fts_query("\"x\" AND (").as_deref(), Some("\"x\"* \"AND\"*"));
        assert_eq!(fts_query("  ¿? "), None);
    }
}
