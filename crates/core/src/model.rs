//! Modelos del dominio. Reflejan el modelo de datos de `docs/ARQUITECTURA.md` §5.2.
//!
//! IDs y fechas se guardan como `String` (UUID v7 y fechas ISO-8601), igual que en SQLite.
//! shortcut: fechas como texto ISO, migrar a un tipo de fecha cuando el módulo de recordatorios necesite aritmética.

use serde::{Deserialize, Serialize};

pub type Id = String;

// ---------------------------------------------------------------------------
// Perfil y CV maestro
// ---------------------------------------------------------------------------

/// Un perfil profesional ("Editor de video", "Programador Jr."), con su propio CV maestro.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub id: Id,
    pub name: String,
    #[serde(default)]
    pub target_titles: Vec<String>,
    #[serde(default)]
    pub remote_pref: RemotePref,
    #[serde(default)]
    pub locations: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemotePref {
    #[default]
    SoloRemoto,
    RemotoOHibrido,
    Cualquiera,
}

/// CV maestro: la única fuente de la verdad sobre la experiencia del usuario.
///
/// Cada experiencia, logro y habilidad lleva un `id` estable. El optimizador solo puede
/// reformular contenido que cite estos ids (ver `guard`, módulo 4); así se evita inventar.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CvMaster {
    pub basics: Basics,
    #[serde(default)]
    pub experience: Vec<Experience>,
    #[serde(default)]
    pub skills: Vec<Skill>,
    #[serde(default)]
    pub education: Vec<Education>,
    #[serde(default)]
    pub certifications: Vec<Certification>,
    #[serde(default)]
    pub languages: Vec<Language>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Basics {
    pub name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub headline: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub links: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Experience {
    pub id: Id,
    pub company: String,
    pub title: String,
    /// "AAAA-MM"
    pub start: String,
    /// `None` = trabajo actual.
    pub end: Option<String>,
    #[serde(default)]
    pub bullets: Vec<Bullet>,
}

/// Un logro concreto. `metrics` guarda las cifras tal como aparecen en el texto,
/// para que el guardián compruebe que la IA no inventa números.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bullet {
    pub id: Id,
    pub text: String,
    #[serde(default)]
    pub metrics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skill {
    pub id: Id,
    pub name: String,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub years: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Education {
    pub id: Id,
    pub institution: String,
    pub degree: String,
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub end: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Certification {
    pub id: Id,
    pub name: String,
    #[serde(default)]
    pub issuer: Option<String>,
    #[serde(default)]
    pub date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Language {
    pub name: String,
    pub level: String,
}

impl CvMaster {
    /// Busca un logro por id en cualquier experiencia.
    pub fn bullet(&self, id: &str) -> Option<&Bullet> {
        self.experience
            .iter()
            .flat_map(|e| &e.bullets)
            .find(|b| b.id == id)
    }

    pub fn skill(&self, id: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.id == id)
    }
}

// ---------------------------------------------------------------------------
// Vacantes y requisitos
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Job {
    pub id: Id,
    pub source_id: String,
    #[serde(default)]
    pub external_id: Option<String>,
    pub url: String,
    #[serde(default)]
    pub company: Option<String>,
    pub title: String,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub remote_type: Option<String>,
    #[serde(default)]
    pub salary_min: Option<i64>,
    #[serde(default)]
    pub salary_max: Option<i64>,
    #[serde(default)]
    pub currency: Option<String>,
    pub description: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub requirements: Option<JobRequirements>,
    #[serde(default)]
    pub posted_at: Option<String>,
    pub found_at: String,
    pub dedupe_hash: String,
}

/// Requisitos extraídos de una vacante (diccionario + LLM local).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct JobRequirements {
    #[serde(default)]
    pub terms: Vec<Term>,
    #[serde(default)]
    pub seniority: Option<String>,
    #[serde(default)]
    pub years_min: Option<f32>,
    #[serde(default)]
    pub languages: Vec<String>,
    /// Restricciones que bloquean la postulación ("US only", ciudadanía...).
    #[serde(default)]
    pub blockers: Vec<String>,
}

/// Palabra clave de una vacante.
///
/// `surface` es la forma exacta del anuncio (se usa al escribir el CV);
/// `canonical` es la forma normalizada (se usa para comparar con el CV maestro).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Term {
    pub surface: String,
    pub canonical: String,
    pub kind: TermKind,
    pub required: bool,
    pub weight: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TermKind {
    Hard,
    Soft,
    Tool,
    Language,
}

// ---------------------------------------------------------------------------
// Seguimiento
// ---------------------------------------------------------------------------

/// Estados del tablero. El valor serializado es el que se guarda en SQLite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Status {
    Encontrada,
    Evaluada,
    CvAdaptado,
    Postulada,
    EnProceso,
    Entrevista,
    Oferta,
    Rechazada,
    SinRespuesta,
    Descartada,
}

impl Status {
    /// Orden de las columnas del Kanban.
    pub const ALL: [Status; 10] = [
        Status::Encontrada,
        Status::Evaluada,
        Status::CvAdaptado,
        Status::Postulada,
        Status::EnProceso,
        Status::Entrevista,
        Status::Oferta,
        Status::Rechazada,
        Status::SinRespuesta,
        Status::Descartada,
    ];

    /// Texto para mostrar en la interfaz.
    pub fn label(self) -> &'static str {
        match self {
            Status::Encontrada => "Encontrada",
            Status::Evaluada => "Evaluada",
            Status::CvAdaptado => "CV adaptado",
            Status::Postulada => "Postulada",
            Status::EnProceso => "En proceso",
            Status::Entrevista => "Entrevista",
            Status::Oferta => "Oferta",
            Status::Rechazada => "Rechazada",
            Status::SinRespuesta => "Sin respuesta",
            Status::Descartada => "Descartada",
        }
    }

    /// ¿Pasar de Postulada a este estado cuenta como "la empresa respondió"?
    /// Un rechazo también es respuesta (ver estadísticas, §5.5).
    pub fn is_response(self) -> bool {
        matches!(
            self,
            Status::EnProceso | Status::Entrevista | Status::Oferta | Status::Rechazada
        )
    }
}

/// Postulación: une una vacante con un perfil y la versión de CV enviada.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Application {
    pub id: Id,
    pub job_id: Id,
    pub profile_id: Id,
    pub status: Status,
    #[serde(default)]
    pub match_score: Option<u8>,
    #[serde(default)]
    pub cv_version_id: Option<Id>,
    #[serde(default)]
    pub cover_letter: Option<String>,
    #[serde(default)]
    pub applied_at: Option<String>,
    #[serde(default)]
    pub first_response_at: Option<String>,
    #[serde(default)]
    pub next_step: Option<String>,
    #[serde(default)]
    pub next_step_at: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const CV_JSON: &str = include_str!("../tests/fixtures/cv_editor_video.json");

    #[test]
    fn cv_maestro_de_ejemplo_se_lee_y_resuelve_ids() -> Result<(), serde_json::Error> {
        let cv: CvMaster = serde_json::from_str(CV_JSON)?;
        assert_eq!(cv.basics.headline, "Editor de video");
        let b = cv.bullet("b_01").expect("b_01 existe en el fixture");
        assert_eq!(b.metrics, vec!["120"]);
        assert!(cv.skill("sk_01").is_some());
        assert!(cv.bullet("no_existe").is_none());

        // Ida y vuelta sin pérdida.
        let again: CvMaster = serde_json::from_str(&serde_json::to_string(&cv)?)?;
        assert_eq!(cv, again);
        Ok(())
    }

    #[test]
    fn estados_de_respuesta() {
        let respuestas: Vec<_> = Status::ALL.into_iter().filter(|s| s.is_response()).collect();
        assert_eq!(
            respuestas,
            [Status::EnProceso, Status::Entrevista, Status::Oferta, Status::Rechazada]
        );
    }

    #[test]
    fn estado_se_serializa_como_en_sqlite() -> Result<(), serde_json::Error> {
        assert_eq!(serde_json::to_string(&Status::CvAdaptado)?, "\"CvAdaptado\"");
        assert_eq!(Status::CvAdaptado.label(), "CV adaptado");
        Ok(())
    }
}
