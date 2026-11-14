//! RustLab — biblioteca principal.
//!
//! A aplicação vive aqui (e não em `main.rs`) para que testes de
//! integração (ex.: snapshots de UI em `tests/`) usem o mesmo código
//! que o binário. `src/main.rs` é uma casca mínima.

pub mod animation;
pub mod app;
pub mod audio;
pub mod completion;
pub mod config;
pub mod create;
pub mod curriculum;
pub mod doc_hint;
pub mod editor;
pub mod event;
pub mod execution;
pub mod exercises;
pub mod highlight;
pub mod i18n;
pub mod json;
pub mod lab_tools;
pub mod learn;
pub mod lsp;
pub mod mentor;
pub mod projects;
pub mod storage;
pub mod theme;
pub mod ui;
