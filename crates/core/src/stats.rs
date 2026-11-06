//! Estadísticas de postulaciones (plan §5.5).
//!
//! - Solo cuentan las postulaciones con **al menos 14 días** desde que te postulaste, para no
//!   castigar las recientes.
//! - Tasa de respuesta por portal, perfil, plantilla de CV y rango de ATS después (<60, 60–79, ≥80).
//! - Días medianos hasta la primera respuesta, embudo Postulada → Entrevista → Oferta y
//!   compatibilidad promedio de las que respondieron frente a las que no.
//! - Cada grupo lleva su **n**; con menos de 10 se marca "pocos datos" y va al final.

use std::collections::BTreeMap;

use serde::Serialize;

/// Días mínimos desde la postulación para que cuente.
pub const MIN_AGE_DAYS: i64 = 14;
/// Por debajo de esta n, un grupo no se usa para ordenar.
pub const FEW_DATA: u32 = 10;

/// Una postulación tal como la necesitan las estadísticas (la arma la base de datos).
#[derive(Debug, Clone, PartialEq)]
pub struct StatInput {
    pub applied_at: String,
    pub first_response_at: Option<String>,
    pub match_score: Option<u8>,
    pub source_id: String,
    pub profile_id: String,
    /// Plantilla del CV enviado (`None` = sin CV adaptado).
    pub variant: Option<String>,
    pub ats_after: Option<u8>,
    /// Llegó alguna vez a Entrevista (u Oferta).
    pub reached_interview: bool,
    pub reached_offer: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GroupStat {
    pub group: String,
    pub applied: u32,
    pub responded: u32,
    pub interviews: u32,
    /// 0–100.
    pub response_rate: f32,
    pub few_data: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Funnel {
    pub applied: u32,
    pub interview: u32,
    pub offer: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Stats {
    /// Postulaciones que cuentan (≥ 14 días).
    pub eligible: u32,
    /// Postuladas hace menos de 14 días (todavía no cuentan).
    pub too_recent: u32,
    pub response_rate: Option<f32>,
    pub median_days_to_response: Option<f32>,
    pub funnel: Funnel,
    pub avg_match_responded: Option<f32>,
    pub avg_match_not_responded: Option<f32>,
    pub by_source: Vec<GroupStat>,
    pub by_profile: Vec<GroupStat>,
    pub by_variant: Vec<GroupStat>,
    pub by_ats: Vec<GroupStat>,
}

/// `today` = fecha actual "AAAA-MM-DD…" (ISO).
pub fn compute(rows: &[StatInput], today: &str) -> Stats {
    let today_n = day_number(today);
    let (eligible, recent): (Vec<&StatInput>, Vec<&StatInput>) =
        rows.iter()
            .partition(|r| match (day_number(&r.applied_at), today_n) {
                (Some(a), Some(t)) => t - a >= MIN_AGE_DAYS,
                _ => false,
            });

    let responded = |r: &StatInput| r.first_response_at.is_some();
    let mut days: Vec<f32> = eligible
        .iter()
        .filter_map(|r| {
            let d = day_number(r.first_response_at.as_deref()?)? - day_number(&r.applied_at)?;
            (d >= 0).then_some(d as f32)
        })
        .collect();
    days.sort_by(f32::total_cmp);

    let avg_match = |want: bool| {
        let v: Vec<f32> = eligible
            .iter()
            .filter(|r| responded(r) == want)
            .filter_map(|r| r.match_score.map(f32::from))
            .collect();
        (!v.is_empty()).then(|| v.iter().sum::<f32>() / v.len() as f32)
    };

    let n = eligible.len() as u32;
    Stats {
        eligible: n,
        too_recent: recent.len() as u32,
        response_rate: (n > 0)
            .then(|| pct(eligible.iter().filter(|r| responded(r)).count() as u32, n)),
        median_days_to_response: median(&days),
        funnel: Funnel {
            applied: n,
            interview: eligible.iter().filter(|r| r.reached_interview).count() as u32,
            offer: eligible.iter().filter(|r| r.reached_offer).count() as u32,
        },
        avg_match_responded: avg_match(true),
        avg_match_not_responded: avg_match(false),
        by_source: group(&eligible, |r| r.source_id.clone()),
        by_profile: group(&eligible, |r| r.profile_id.clone()),
        by_variant: group(&eligible, |r| {
            r.variant
                .clone()
                .unwrap_or_else(|| "sin CV adaptado".into())
        }),
        by_ats: group(&eligible, |r| ats_band(r.ats_after).into()),
    }
}

fn ats_band(ats: Option<u8>) -> &'static str {
    match ats {
        None => "sin CV adaptado",
        Some(a) if a < 60 => "ATS < 60",
        Some(a) if a < 80 => "ATS 60–79",
        Some(_) => "ATS ≥ 80",
    }
}

fn group(rows: &[&StatInput], key: impl Fn(&StatInput) -> String) -> Vec<GroupStat> {
    let mut acc: BTreeMap<String, (u32, u32, u32)> = BTreeMap::new();
    for r in rows {
        let e = acc.entry(key(r)).or_default();
        e.0 += 1;
        e.1 += u32::from(r.first_response_at.is_some());
        e.2 += u32::from(r.reached_interview);
    }
    let mut out: Vec<GroupStat> = acc
        .into_iter()
        .map(|(group, (applied, responded, interviews))| GroupStat {
            group,
            applied,
            responded,
            interviews,
            response_rate: pct(responded, applied),
            few_data: applied < FEW_DATA,
        })
        .collect();
    // Con datos suficientes, por tasa; los de pocos datos al final, por n.
    out.sort_by(|a, b| {
        a.few_data
            .cmp(&b.few_data)
            .then_with(|| {
                if a.few_data {
                    b.applied.cmp(&a.applied)
                } else {
                    b.response_rate.total_cmp(&a.response_rate)
                }
            })
            .then_with(|| a.group.cmp(&b.group))
    });
    out
}

fn pct(part: u32, total: u32) -> f32 {
    if total == 0 {
        0.0
    } else {
        (1000.0 * part as f32 / total as f32).round() / 10.0
    }
}

fn median(sorted: &[f32]) -> Option<f32> {
    let n = sorted.len();
    match n {
        0 => None,
        _ if n % 2 == 1 => Some(sorted[n / 2]),
        _ => Some((sorted[n / 2 - 1] + sorted[n / 2]) / 2.0),
    }
}

/// Días desde 1970-01-01 de una fecha ISO ("AAAA-MM-DD", con o sin hora).
fn day_number(iso: &str) -> Option<i64> {
    let y: i64 = iso.get(0..4)?.parse().ok()?;
    let m: i64 = iso.get(5..7)?.parse().ok()?;
    let d: i64 = iso.get(8..10)?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    // Algoritmo de Howard Hinnant (days_from_civil).
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(applied: &str, response: Option<&str>, source: &str) -> StatInput {
        StatInput {
            applied_at: applied.into(),
            first_response_at: response.map(String::from),
            match_score: Some(if response.is_some() { 80 } else { 60 }),
            source_id: source.into(),
            profile_id: "p1".into(),
            variant: Some("clasica".into()),
            ats_after: Some(75),
            reached_interview: false,
            reached_offer: false,
        }
    }

    #[test]
    fn dias_entre_fechas() {
        assert_eq!(day_number("1970-01-01"), Some(0));
        assert_eq!(
            day_number("2026-03-01T10:00:00Z"),
            day_number("2026-02-28").map(|d| d + 1)
        );
        assert_eq!(
            day_number("2024-03-01")
                .zip(day_number("2024-02-28"))
                .map(|(a, b)| a - b),
            Some(2)
        );
        assert_eq!(day_number("no"), None);
    }

    #[test]
    fn solo_cuentan_las_de_14_dias_o_mas() {
        let rows = [
            row("2026-10-01", Some("2026-10-05"), "remotive"),
            row("2026-10-23", None, "remotive"), // justo 14 días: cuenta
            row("2026-10-30", Some("2026-11-01"), "remotive"), // 7 días: todavía no
        ];
        let s = compute(&rows, "2026-11-06T06:00:00Z");
        assert_eq!((s.eligible, s.too_recent), (2, 1));
        assert_eq!(s.response_rate, Some(50.0));
        assert_eq!(s.median_days_to_response, Some(4.0));
        assert_eq!(s.avg_match_responded, Some(80.0));
        assert_eq!(s.avg_match_not_responded, Some(60.0));
    }

    #[test]
    fn grupos_con_n_y_pocos_datos_al_final() {
        let mut rows: Vec<StatInput> = (0..10)
            .map(|i| row("2026-09-01", (i < 3).then_some("2026-09-10"), "remoteok"))
            .collect();
        rows.extend((0..2).map(|_| row("2026-09-01", Some("2026-09-03"), "jobicy")));
        rows[0].reached_interview = true;
        rows[0].reached_offer = true;
        rows[1].ats_after = None;
        rows[1].variant = None;
        let s = compute(&rows, "2026-11-06");

        // Jobicy tiene 100 % pero solo n = 2: no se ordena por encima de Remote OK.
        let src: Vec<(&str, u32, f32, bool)> = s
            .by_source
            .iter()
            .map(|g| (g.group.as_str(), g.applied, g.response_rate, g.few_data))
            .collect();
        assert_eq!(
            src,
            [("remoteok", 10, 30.0, false), ("jobicy", 2, 100.0, true)]
        );
        assert_eq!(
            s.funnel,
            Funnel {
                applied: 12,
                interview: 1,
                offer: 1
            }
        );
        let ats: Vec<&str> = s.by_ats.iter().map(|g| g.group.as_str()).collect();
        assert_eq!(ats, ["ATS 60–79", "sin CV adaptado"]);
        assert!(s.by_variant.iter().any(|g| g.group == "sin CV adaptado"));
    }

    #[test]
    fn sin_datos_no_inventa_cifras() {
        let s = compute(&[], "2026-11-06");
        assert_eq!(s.eligible, 0);
        assert_eq!((s.response_rate, s.median_days_to_response), (None, None));
        assert!(s.by_source.is_empty());
    }
}
