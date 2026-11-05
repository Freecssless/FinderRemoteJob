//! Transiciones de estado de una postulación (plan §5.1).
//!
//! Cualquier transición está permitida (la gente se equivoca), pero se registran las fechas
//! clave para las estadísticas: cuándo te postulaste y cuándo respondió la empresa por primera vez.

use crate::model::{Application, Status};

/// Aplica el cambio de estado en `app`. Devuelve `false` si ya estaba en ese estado.
///
/// - Pasar a Postulada o a un estado posterior fija `applied_at` si faltaba
///   (si saltas directo a Entrevista, es que te postulaste).
/// - El primer paso a un estado de respuesta (En proceso, Entrevista, Oferta, Rechazada)
///   fija `first_response_at`. Regresar a un estado anterior no lo borra.
pub fn transition(app: &mut Application, to: Status, at: &str) -> bool {
    if app.status == to {
        return false;
    }
    if to.implies_applied() && app.applied_at.is_none() {
        app.applied_at = Some(at.to_string());
    }
    if to.is_response() && app.first_response_at.is_none() {
        app.first_response_at = Some(at.to_string());
    }
    app.status = to;
    app.updated_at = at.to_string();
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> Application {
        Application {
            id: "a".into(),
            job_id: "j".into(),
            profile_id: "p".into(),
            status: Status::Encontrada,
            match_score: None,
            cv_version_id: None,
            cover_letter: None,
            applied_at: None,
            first_response_at: None,
            next_step: None,
            next_step_at: None,
            notes: None,
            updated_at: "t0".into(),
        }
    }

    #[test]
    fn flujo_normal_registra_postulacion_y_primera_respuesta() {
        let mut a = app();
        assert!(transition(&mut a, Status::CvAdaptado, "t1"));
        assert_eq!(a.applied_at, None);
        assert!(transition(&mut a, Status::Postulada, "t2"));
        assert_eq!(a.applied_at.as_deref(), Some("t2"));
        assert!(transition(&mut a, Status::Entrevista, "t3"));
        assert!(transition(&mut a, Status::Oferta, "t4"));
        assert_eq!(
            a.first_response_at.as_deref(),
            Some("t3"),
            "solo cuenta la primera respuesta"
        );
        assert_eq!(a.updated_at, "t4");
    }

    #[test]
    fn mismo_estado_no_cambia_nada() {
        let mut a = app();
        assert!(!transition(&mut a, Status::Encontrada, "t1"));
        assert_eq!(a.updated_at, "t0");
    }

    #[test]
    fn saltar_directo_a_entrevista_implica_postulacion() {
        let mut a = app();
        transition(&mut a, Status::Entrevista, "t1");
        assert_eq!(a.applied_at.as_deref(), Some("t1"));
        assert_eq!(a.first_response_at.as_deref(), Some("t1"));
    }

    #[test]
    fn rechazo_cuenta_como_respuesta_y_sin_respuesta_no() {
        let mut a = app();
        transition(&mut a, Status::Postulada, "t1");
        transition(&mut a, Status::SinRespuesta, "t2");
        assert_eq!(a.first_response_at, None);
        transition(&mut a, Status::Rechazada, "t3");
        assert_eq!(a.first_response_at.as_deref(), Some("t3"));
    }
}
