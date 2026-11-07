// Envoltorios tipados de invoke(). Los tipos reflejan crates/core/src/model.rs.
import { invoke } from "@tauri-apps/api/core";

export type RemotePref = "SoloRemoto" | "RemotoOHibrido" | "Cualquiera";

export const REMOTE_LABELS: Record<RemotePref, string> = {
  SoloRemoto: "Solo remoto",
  RemotoOHibrido: "Remoto o híbrido",
  Cualquiera: "Cualquiera",
};

export interface Profile {
  id: string;
  name: string;
  target_titles: string[];
  remote_pref: RemotePref;
  locations: string[];
  created_at: string;
}

export interface Basics {
  name: string;
  email: string;
  phone: string;
  headline: string;
  location: string;
  links: string[];
}
export interface Bullet { id: string; text: string; metrics: string[] }
export interface Experience {
  id: string;
  company: string;
  title: string;
  start: string;
  end: string | null;
  bullets: Bullet[];
}
export interface Skill { id: string; name: string; level: string | null; years: number | null }
export interface Education {
  id: string;
  institution: string;
  degree: string;
  start: string | null;
  end: string | null;
}
export interface Certification { id: string; name: string; issuer: string | null; date: string | null }
export interface Language { name: string; level: string }

export interface CvMaster {
  basics: Basics;
  experience: Experience[];
  skills: Skill[];
  education: Education[];
  certifications: Certification[];
  languages: Language[];
}

export interface ImportPreview { cv: CvMaster; warnings: string[]; source_text: string }

export interface AppInfo {
  db_path: string;
  env_file: string | null;
  local_model: string | null;
  writer_model: string | null;
  llm_error: string | null;
}

export const emptyCv = (): CvMaster => ({
  basics: { name: "", email: "", phone: "", headline: "", location: "", links: [] },
  experience: [],
  skills: [],
  education: [],
  certifications: [],
  languages: [],
});

export const api = {
  appInfo: () => invoke<AppInfo>("app_info"),
  navCounts: (profileId: string | null) =>
    invoke<{ new_jobs: number; queue: number; due: number }>("nav_counts", { profileId }),
  ollamaStatus: () => invoke<{ running: boolean; model: string; installed: boolean }>("ollama_status"),
  listProfiles: () => invoke<Profile[]>("list_profiles"),
  createProfile: (p: Omit<Profile, "id" | "created_at">) =>
    invoke<Profile>("create_profile", {
      name: p.name,
      targetTitles: p.target_titles,
      remotePref: p.remote_pref,
      locations: p.locations,
    }),
  updateProfile: (profile: Profile) => invoke<void>("update_profile", { profile }),
  deleteProfile: (id: string) => invoke<void>("delete_profile", { id }),
  getCv: (profileId: string) => invoke<CvMaster | null>("get_cv", { profileId }),
  saveCv: (profileId: string, cv: CvMaster, sourceText: string | null) =>
    invoke<void>("save_cv", { profileId, cv, sourceText }),
  importCv: async (file: File) =>
    invoke<ImportPreview>("import_cv", {
      fileName: file.name,
      bytes: Array.from(new Uint8Array(await file.arrayBuffer())),
    }),
};

// ---------------------------------------------------------------------------
// Vacantes (crates/core/src/matching.rs y src-tauri/src/jobs.rs)
// ---------------------------------------------------------------------------

export interface Term {
  surface: string;
  canonical: string;
  kind: "Hard" | "Soft" | "Tool" | "Language";
  required: boolean;
  weight: number;
}
export interface JobRequirements {
  terms: Term[];
  seniority: string | null;
  years_min: number | null;
  languages: string[];
  blockers: string[];
}
export interface Job {
  id: string;
  source_id: string;
  url: string;
  company: string | null;
  title: string;
  location: string | null;
  remote_type: string | null;
  salary_min: number | null;
  salary_max: number | null;
  currency: string | null;
  description: string;
  requirements: JobRequirements | null;
  posted_at: string | null;
  found_at: string;
}
export interface MatchReport {
  total: number;
  breakdown: {
    skills: number;
    semantic: number | null;
    title: number;
    seniority: number;
    years: number;
    logistics: number;
  };
  years_experience: number;
  missing_required: string[];
  blockers: string[];
  low_relevance: boolean;
  /** Máximo de cada parte (los pesos en uso). */
  max: Weights;
}
export interface Weights { skills: number; semantic: number; title: number; seniority: number; years: number; logistics: number }
export interface WeightsStatus {
  samples: number; advanced: number; not_advanced: number; needed: number; min_each: number;
  current: Weights; default: Weights; suggested: Weights | null;
}
export const weightsApi = {
  status: () => invoke<WeightsStatus>("weights_status"),
  apply: (weights: Weights) => invoke<void>("apply_weights", { weights }),
  reset: () => invoke<void>("reset_weights"),
};
export interface JobRow {
  id: string;
  title: string;
  company: string | null;
  location: string | null;
  source_id: string;
  url: string;
  posted_at: string | null;
  found_at: string;
  analyzed: boolean;
  score: MatchReport | null;
}
export interface JobDetail { job: Job; score: MatchReport | null }
export interface SyncSummary {
  source: string;
  skipped_too_soon: boolean;
  fetched: number;
  kept: number;
  new: number;
  error: string | null;
}
export interface ImportedJob { id: string; created: boolean }

export const SOURCE_LABELS: Record<string, string> = {
  remoteok: "Remote OK",
  remotive: "Remotive",
  wwr: "We Work Remotely",
  getonboard: "Get on Board",
  jobicy: "Jobicy",
  himalayas: "Himalayas",
  greenhouse: "Greenhouse",
  lever: "Lever",
  ashby: "Ashby",
  manual: "Importada",
};

/** Clase CSS de la insignia `.score` según el puntaje (0–100). */
export const scoreClass = (n: number) => (n >= 70 ? "high" : n >= 45 ? "mid" : "low");

export const jobsApi = {
  list: (profileId: string | null, query: string) =>
    invoke<JobRow[]>("list_jobs", { profileId, query }),
  get: (jobId: string, profileId: string | null) => invoke<JobDetail>("get_job", { jobId, profileId }),
  analyze: (jobId: string, profileId: string | null) =>
    invoke<JobDetail>("analyze_job", { jobId, profileId }),
  sync: (profileId: string) => invoke<SyncSummary[]>("sync_jobs", { profileId }),
  importUrl: (url: string) => invoke<ImportedJob>("import_job_url", { url }),
  importText: (title: string, company: string, url: string, text: string) =>
    invoke<ImportedJob>("import_job_text", { title, company, url, text }),
  portals: (keywords: string) => invoke<{ name: string; url: string }[]>("assisted_portals", { keywords }),
  /** Empresas seguidas en Greenhouse, Lever o Ashby: una por línea (`greenhouse:gitlab`). */
  /** Similitud semántica (IA local) de las vacantes recientes que no la tienen; devuelve cuántas calculó. */
  embed: (profileId: string) => invoke<number>("embed_jobs", { profileId }),
  boards: () => invoke<string>("get_company_boards"),
  saveBoards: (text: string) => invoke<string>("save_company_boards", { text }),
};

// Perfil activo, recordado entre pantallas y sesiones (solo comodidad: si falla, se elige el primero).
const PROFILE_KEY = "frj.profile";
export function rememberedProfile(): string | null {
  try {
    return localStorage.getItem(PROFILE_KEY);
  } catch {
    return null;
  }
}
export function rememberProfile(id: string) {
  try {
    localStorage.setItem(PROFILE_KEY, id);
  } catch {
    /* sin almacenamiento: no pasa nada */
  }
}

// ---------------------------------------------------------------------------
// Optimizador (crates/infra/src/optimizer.rs y src-tauri/src/optimizer.rs)
// ---------------------------------------------------------------------------

export interface AtsReport {
  total: number;
  breakdown: { keywords: number; title: number; sections: number; legibility: number };
  matched: string[];
  synonym_only: { job_surface: string; cv_surface: string }[];
  missing: Term[];
  suggestions: string[];
}
export interface Checked {
  exp_id: string | null;
  source_ids: string[];
  proposed: string;
  originals: string[];
}
export interface Choices { summary: boolean; bullets: boolean[] }
export interface Optimization {
  prepared: {
    requirements: JobRequirements;
    notes: string[];
    lang: "es" | "en";
    ats_before: AtsReport;
    gaps: { with_evidence: Term[]; without_evidence: Term[] };
  };
  review: { summary: Checked | null; bullets: Checked[]; invalid_skill_ids: string[] };
  choices: Choices;
  ats_after: AtsReport;
  roundtrip: { missing_words: string[]; missing_ratio: number; sections_in_order: boolean };
}
export interface OptimizerView {
  profile_id: string;
  opt: Optimization;
  summary_issues: string[];
  bullet_issues: string[][];
  experiences: [string, string][];
}
export interface SavedCv {
  ats_before: number | null;
  ats_after: number | null;
  pdf_path: string | null;
  docx_path: string | null;
  missing: string[];
}

/** Plantillas del CV (plan §4.6); se guardan como `variant`. */
export type Template = "clasica" | "compacta" | "tecnica";
export const TEMPLATES: { id: Template; label: string; hint: string }[] = [
  { id: "clasica", label: "Clásica", hint: "Una columna, espaciado cómodo." },
  { id: "compacta", label: "Compacta", hint: "Letra y márgenes más chicos para caber en una página." },
  { id: "tecnica", label: "Técnica", hint: "Habilidades arriba, antes de la experiencia." },
];
export const TEMPLATE_LABELS: Record<string, string> = Object.fromEntries(TEMPLATES.map((t) => [t.id, t.label]));

/** El CV listo para imprimir (crates/core/src/document.rs); también es el `content_json` guardado. */
export interface CvDocument {
  template: Template;
  headings: Record<"summary" | "experience" | "education" | "skills" | "certifications" | "languages", string>;
  name: string;
  headline: string;
  contact: string[];
  summary: string | null;
  experience: { title: string; company: string; dates: string; bullets: string[] }[];
  skills: string[];
  education: { degree: string; institution: string; dates: string }[];
  certifications: string[];
  languages: string[];
}
export interface SavedEntry { version: CvVersion & { content_json: string; job_id: string | null }; job_title: string | null; company: string | null }

export const optimizerApi = {
  get: (jobId: string, profileId: string) =>
    invoke<OptimizerView | null>("get_optimization", { jobId, profileId }),
  run: (jobId: string, profileId: string) => invoke<OptimizerView>("run_optimizer", { jobId, profileId }),
  save: (jobId: string, choices: Choices, template: Template) =>
    invoke<SavedCv>("save_optimized", { jobId, choices, template }),
  /** `bullet` = null corrige el resumen. */
  edit: (jobId: string, bullet: number | null, text: string) =>
    invoke<OptimizerView>("edit_proposal", { jobId, bullet, text }),
  preview: (jobId: string, choices: Choices, template: Template) =>
    invoke<CvDocument>("preview_cv", { jobId, choices, template }),
  listSaved: (profileId: string) => invoke<SavedEntry[]>("list_saved_cvs", { profileId }),
};

// ---------------------------------------------------------------------------
// Tablero
// ---------------------------------------------------------------------------

export type Status =
  | "Encontrada" | "Evaluada" | "CvAdaptado" | "Postulada" | "EnProceso"
  | "Entrevista" | "Oferta" | "Rechazada" | "SinRespuesta" | "Descartada";

/** Columnas del Kanban, en el orden de `Status::ALL`. */
export const STATUSES: { id: Status; label: string }[] = [
  { id: "Encontrada", label: "Encontrada" },
  { id: "Evaluada", label: "Evaluada" },
  { id: "CvAdaptado", label: "CV adaptado" },
  { id: "Postulada", label: "Postulada" },
  { id: "EnProceso", label: "En proceso" },
  { id: "Entrevista", label: "Entrevista" },
  { id: "Oferta", label: "Oferta" },
  { id: "Rechazada", label: "Rechazada" },
  { id: "SinRespuesta", label: "Sin respuesta" },
  { id: "Descartada", label: "Descartada" },
];
export const STATUS_LABELS = Object.fromEntries(STATUSES.map((s) => [s.id, s.label])) as Record<Status, string>;

export interface Application {
  id: string;
  job_id: string;
  profile_id: string;
  status: Status;
  match_score: number | null;
  cv_version_id: string | null;
  applied_at: string | null;
  first_response_at: string | null;
  next_step: string | null;
  next_step_at: string | null;
  notes: string | null;
  updated_at: string;
}
export interface BoardRow {
  application: Application;
  job_title: string;
  company: string | null;
  source_id: string;
  url: string;
  found_at: string;
}
export interface CvVersion {
  id: string;
  variant: string;
  ats_before: number | null;
  ats_after: number | null;
  missing: string[];
  pdf_path: string | null;
  docx_path: string | null;
  created_at: string;
}
export interface StatusChange { from: Status | null; to: Status; at: string }
export type ReminderKind = "Seguimiento" | "SinRespuesta" | "Agradecimiento" | "ProximoPaso";
export const REMINDER_LABELS: Record<ReminderKind, string> = {
  Seguimiento: "Dar seguimiento",
  SinRespuesta: "Sin noticias: ¿marcar «Sin respuesta»?",
  Agradecimiento: "Enviar agradecimiento por la entrevista",
  ProximoPaso: "Próximo paso",
};
export interface Reminder {
  id: number;
  application_id: string;
  profile_id: string;
  kind: ReminderKind;
  due_at: string;
  job_title: string;
  company: string | null;
}
export interface ReminderDays { follow_up: number; no_response: number }
export interface ApplicationDetail { history: StatusChange[]; cv: CvVersion | null; reminders: Reminder[] }

export const boardApi = {
  list: (profileId: string) => invoke<BoardRow[]>("list_board", { profileId }),
  setStatus: (applicationId: string, status: Status) =>
    invoke<Application>("set_application_status", { applicationId, status }),
  saveNotes: (applicationId: string, notes: string, nextStep: string, nextStepAt: string) =>
    invoke<void>("save_application_notes", { applicationId, notes, nextStep, nextStepAt }),
  detail: (applicationId: string) => invoke<ApplicationDetail>("application_detail", { applicationId }),
  add: (jobId: string, profileId: string) => invoke<string>("add_to_board", { jobId, profileId }),
  dueReminders: (profileId: string) => invoke<Reminder[]>("list_due_reminders", { profileId }),
  completeReminder: (id: number) => invoke<void>("complete_reminder", { id }),
  reminderDays: () => invoke<ReminderDays>("get_reminder_days"),
  setReminderDays: (days: ReminderDays) => invoke<void>("set_reminder_days", { days }),
};

// ---------------------------------------------------------------------------
// Estadísticas
// ---------------------------------------------------------------------------

export interface GroupStat {
  group: string;
  applied: number;
  responded: number;
  interviews: number;
  response_rate: number;
  few_data: boolean;
}
export interface Stats {
  eligible: number;
  too_recent: number;
  response_rate: number | null;
  median_days_to_response: number | null;
  funnel: { applied: number; interview: number; offer: number };
  avg_match_responded: number | null;
  avg_match_not_responded: number | null;
  by_source: GroupStat[];
  by_profile: GroupStat[];
  by_variant: GroupStat[];
  by_ats: GroupStat[];
}

export const statsApi = {
  get: (profileId: string | null) => invoke<Stats>("get_stats", { profileId }),
};

// ---------------------------------------------------------------------------
// Ajustes de IA
// ---------------------------------------------------------------------------

export interface LlmSettings {
  values: Record<string, string>;
  saved_keys: string[];
  overridden: string[];
  env_file: string | null;
  writer: string | null;
  writer_kind: "local" | "anthropic" | "openai" | null;
  local: string | null;
  error: string | null;
  keyring_error: string | null;
}

export const settingsApi = {
  get: () => invoke<LlmSettings>("get_llm_settings"),
  save: (values: Record<string, string>, keys: Record<string, string>) =>
    invoke<LlmSettings>("save_llm_settings", { input: { values, keys } }),
  deleteKey: (key: string) => invoke<LlmSettings>("delete_api_key", { key }),
};

// ---------------------------------------------------------------------------
// Cola de postulación asistida y respuestas frecuentes (plan §3)
// ---------------------------------------------------------------------------

export interface QueueItem {
  job_id: string;
  title: string;
  company: string | null;
  url: string;
  score: number;
  application_id: string | null;
  status: Status | null;
  pdf_path: string | null;
}

export interface Faq {
  question: string;
  answer: string;
}

export interface FaqView {
  from_cv: Faq[];
  custom: Faq[];
}

export const queueApi = {
  list: (profileId: string) => invoke<QueueItem[]>("application_queue", { profileId }),
  faq: (profileId: string) => invoke<FaqView>("get_faq", { profileId }),
  saveFaq: (profileId: string, items: Faq[]) => invoke<void>("save_faq", { profileId, items }),
};

// Respaldo (src-tauri/src/backup.rs)
export const backupApi = {
  /** Devuelve la ruta del .zip creado. */
  create: () => invoke<string>("create_backup"),
  /** Reinicia la app si todo va bien. */
  restore: async (file: File) =>
    invoke<void>("restore_backup", { bytes: Array.from(new Uint8Array(await file.arrayBuffer())) }),
};
