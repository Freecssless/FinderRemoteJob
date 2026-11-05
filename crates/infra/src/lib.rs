//! Infraestructura: todo lo que hace IO (SQLite, red, LLM, archivos).

pub use finder_core as core;

pub mod cvfile;
pub mod db;
pub mod docgen;

pub mod llm;
pub mod optimizer;
pub mod sources;
