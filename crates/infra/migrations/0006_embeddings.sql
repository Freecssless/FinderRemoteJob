-- Embeddings guardados (similitud semántica): `owner` es "job:<id>" o "cv:<perfil>".
-- `text_hash` dice con qué texto se calculó: si el CV cambia, se recalcula.
CREATE TABLE embeddings (
  owner      TEXT NOT NULL,
  model      TEXT NOT NULL,
  text_hash  TEXT NOT NULL,
  vector     BLOB NOT NULL,             -- f32 little-endian
  PRIMARY KEY (owner, model)
);
