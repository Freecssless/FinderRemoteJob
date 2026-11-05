-- Esquema inicial (docs/ARQUITECTURA.md §5.2).
-- IDs: UUID v7 en texto. Fechas: ISO-8601 en texto (UTC).

CREATE TABLE profiles (
  id            TEXT PRIMARY KEY,
  name          TEXT NOT NULL,
  target_titles TEXT NOT NULL DEFAULT '[]',          -- JSON: lista de puestos objetivo
  remote_pref   TEXT NOT NULL DEFAULT 'SoloRemoto',
  locations     TEXT NOT NULL DEFAULT '[]',          -- JSON
  created_at    TEXT NOT NULL
);

CREATE TABLE cv_master (
  profile_id         TEXT PRIMARY KEY REFERENCES profiles(id) ON DELETE CASCADE,
  data_json          TEXT NOT NULL,                  -- CvMaster
  original_file_text TEXT,                           -- texto del CV importado (para el ATS "antes")
  updated_at         TEXT NOT NULL
);

CREATE TABLE sources (
  id           TEXT PRIMARY KEY,
  name         TEXT NOT NULL,
  access       TEXT NOT NULL,                        -- api | rss | email | manual
  enabled      INTEGER NOT NULL DEFAULT 1,
  last_sync_at TEXT
);

-- Fuentes del MVP (módulo 6). Portales sin API solo entran por importación manual.
INSERT INTO sources (id, name, access) VALUES
  ('remoteok',   'Remote OK',        'api'),
  ('remotive',   'Remotive',         'api'),
  ('wwr',        'We Work Remotely', 'rss'),
  ('getonboard', 'Get on Board',     'api'),
  ('jobicy',     'Jobicy',           'api'),
  ('manual',     'Importación manual', 'manual');

CREATE TABLE jobs (
  id                TEXT PRIMARY KEY,
  source_id         TEXT NOT NULL REFERENCES sources(id),
  external_id       TEXT,
  url               TEXT NOT NULL,
  company           TEXT,
  title             TEXT NOT NULL,
  location          TEXT,
  remote_type       TEXT,
  salary_min        INTEGER,
  salary_max        INTEGER,
  currency          TEXT,
  description       TEXT NOT NULL,
  language          TEXT,
  requirements_json TEXT,                            -- JobRequirements
  posted_at         TEXT,
  found_at          TEXT NOT NULL,
  dedupe_hash       TEXT NOT NULL UNIQUE             -- core::model::dedupe_key
);
CREATE INDEX jobs_found_at ON jobs(found_at);

-- Búsqueda de texto completo. Tabla FTS propia (no "external content"): el rowid de `jobs`
-- puede cambiar con VACUUM porque su clave es TEXT, y eso desincronizaría el índice.
CREATE VIRTUAL TABLE jobs_fts USING fts5(job_id UNINDEXED, title, company, description);
CREATE TRIGGER jobs_fts_ai AFTER INSERT ON jobs BEGIN
  INSERT INTO jobs_fts (job_id, title, company, description)
  VALUES (new.id, new.title, coalesce(new.company, ''), new.description);
END;
CREATE TRIGGER jobs_fts_ad AFTER DELETE ON jobs BEGIN
  DELETE FROM jobs_fts WHERE job_id = old.id;
END;
CREATE TRIGGER jobs_fts_au AFTER UPDATE OF title, company, description ON jobs BEGIN
  UPDATE jobs_fts SET title = new.title, company = coalesce(new.company, ''), description = new.description
  WHERE job_id = new.id;
END;

CREATE TABLE cv_versions (
  id           TEXT PRIMARY KEY,
  profile_id   TEXT NOT NULL REFERENCES profiles(id),
  job_id       TEXT REFERENCES jobs(id),
  variant      TEXT NOT NULL,
  content_json TEXT NOT NULL,
  master_hash  TEXT NOT NULL,
  ats_before   INTEGER,
  ats_after    INTEGER,
  missing_json TEXT,
  pdf_path     TEXT,
  docx_path    TEXT,
  created_at   TEXT NOT NULL
);

CREATE TABLE applications (
  id                TEXT PRIMARY KEY,
  job_id            TEXT NOT NULL REFERENCES jobs(id),
  profile_id        TEXT NOT NULL REFERENCES profiles(id),
  status            TEXT NOT NULL,
  match_score       INTEGER,
  cv_version_id     TEXT REFERENCES cv_versions(id),
  cover_letter      TEXT,
  applied_at        TEXT,
  first_response_at TEXT,
  next_step         TEXT,
  next_step_at      TEXT,
  notes             TEXT,
  updated_at        TEXT NOT NULL,
  UNIQUE (job_id, profile_id)
);
CREATE INDEX applications_profile_status ON applications(profile_id, status);

CREATE TABLE status_history (
  id             INTEGER PRIMARY KEY,
  application_id TEXT NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
  from_status    TEXT,
  to_status      TEXT NOT NULL,
  at             TEXT NOT NULL
);

CREATE TABLE reminders (
  id             INTEGER PRIMARY KEY,
  application_id TEXT NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
  due_at         TEXT NOT NULL,
  kind           TEXT NOT NULL,
  done           INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
