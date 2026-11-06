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
}
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
  manual: "Importada",
};

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
