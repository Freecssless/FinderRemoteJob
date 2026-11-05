//! Dominio puro de FinderRemoteJob.
//!
//! Regla de oro: este crate no hace IO (ni red, ni disco, ni base de datos),
//! para poder reutilizarlo tal cual en escritorio, Android e iOS.

pub mod ats;
pub mod guard;
pub mod matching;
pub mod model;
