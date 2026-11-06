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
