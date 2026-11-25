use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use rustlab::curriculum::v3_blueprint::{
    V3Entry, V3EntryKind, V3Stage, build_v3_lesson, build_v3_practice_spec, find_v3_entry,
    is_v3_entry_completed, is_v3_entry_unlocked, v3_curriculum_blueprint,
};
use rustlab::curriculum::{Block, CalloutKind, Curriculum};
use rustlab::learn::{FerrisMood, PracticeSpec, Progress, calculate_v3_practice_xp};
use rustlab::storage::{self, ProgressData};
use serde::Serialize;

pub const MAX_SOURCE: usize = 256 * 1024;
const MAX_IMPORT: usize = 64 * 1024 * 1024;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseMap {
    pub stages: Vec<Stage>,
    pub xp: u64,
    pub next: Option<&'static str>,
    pub reviews: Vec<String>,
    pub ferris: &'static str,
    pub warning: Option<String>,
}

#[derive(Serialize)]
pub struct Stage {
    pub title: &'static str,
    pub entries: Vec<Entry>,
}

#[derive(Serialize)]
pub struct Entry {
    pub id: &'static str,
    pub title: &'static str,
    pub prerequisites: &'static [&'static str],
    pub kind: &'static str,
    pub completed: bool,
    pub unlocked: bool,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Content {
    Text {
        text: String,
    },
    Code {
        code: String,
        file: Option<String>,
        explain: Option<String>,
    },
    Callout {
        text: String,
        label: &'static str,
    },
    Demo {
        name: String,
        steps: Vec<String>,
        caption: Option<String>,
    },
}

#[derive(Serialize)]
pub struct Question {
    pub question: String,
    pub options: Vec<String>,
}

#[derive(Serialize)]
pub struct Practice {
    pub prompt: &'static str,
    pub source: String,
    pub hints: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct Lesson {
    pub entry: Entry,
    pub stage: &'static str,
    pub minutes: u32,
    pub content: Vec<Content>,
    pub questions: Vec<Question>,
    pub practice: Option<Practice>,
}

#[derive(Serialize)]
pub struct QuizResult {
    pub correct: Vec<bool>,
    pub explanations: Vec<Option<String>>,
    pub completed: bool,
    pub awarded: u32,
    pub ferris: &'static str,
}

pub struct Course {
    curriculum: Curriculum,
    path: PathBuf,
    progress: Progress,
    warning: Option<String>,
}

impl Course {
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let curriculum = Curriculum::embedded().map_err(|error| error.to_string())?;
        let (data, warning) = storage::load_from(&path);
        Ok(Self {
            curriculum,
            path,
            progress: Progress::from_data(data),
            warning,
        })
    }

    pub fn map(&self) -> CourseMap {
        let mut next = None;
        let stages = V3Stage::ALL
            .into_iter()
            .map(|stage| Stage {
                title: stage.label(),
                entries: v3_curriculum_blueprint()
                    .iter()
                    .filter(|entry| entry.stage == stage)
                    .map(|entry| {
                        let item = self.entry(entry);
                        if next.is_none() && item.unlocked && !item.completed {
                            next = Some(item.id);
                        }
                        item
                    })
                    .collect(),
            })
            .collect();
        CourseMap {
            stages,
            xp: self
                .progress
                .xp_events
                .values()
                .map(|&xp| u64::from(xp))
                .sum(),
            next,
            reviews: self.progress.due_srs_items(),
            ferris: FerrisMood::Neutral.ascii(),
            warning: self.warning.clone(),
        }
    }

    fn entry(&self, entry: &V3Entry) -> Entry {
        Entry {
            id: entry.id,
            title: entry.title,
            prerequisites: entry.prerequisites,
            kind: match entry.kind {
                V3EntryKind::Theory => "Teoria",
                V3EntryKind::Practice => "Prática",
                V3EntryKind::CapstoneMilestone => "Marco de projeto",
            },
            completed: is_v3_entry_completed(&self.progress, entry),
            unlocked: is_v3_entry_unlocked(&self.progress, entry, false),
        }
    }

    fn accessible_entry(&self, id: &str) -> Result<&'static V3Entry, String> {
        let entry = find_v3_entry(id).ok_or("Lição desconhecida.")?;
        if !self.entry(entry).unlocked {
            return Err("Conclua os pré-requisitos para abrir esta lição.".into());
        }
        Ok(entry)
    }

    pub fn lesson(&self, id: &str) -> Result<Lesson, String> {
        let entry = self.accessible_entry(id)?;
        let lesson = build_v3_lesson(entry, &self.curriculum);
        let mut questions = Vec::new();
        let mut content = Vec::new();
        for block in lesson.blocks {
            match block {
                Block::Text { text } => content.push(Content::Text { text }),
                Block::Code {
                    code,
                    file,
                    explain,
                } => {
                    content.push(Content::Code {
                        code,
                        file,
                        explain,
                    });
                }
                Block::Callout { text, kind } => content.push(Content::Callout {
                    text,
                    label: match kind {
                        CalloutKind::Note => "Nota",
                        CalloutKind::Tip => "Sugestão",
                        CalloutKind::Warning => "Atenção",
                    },
                }),
                Block::Demo {
                    name,
                    steps,
                    caption,
                    ..
                } => {
                    content.push(Content::Demo {
                        name,
                        steps,
                        caption,
                    });
                }
                Block::Quiz {
                    question, options, ..
                } => questions.push(Question { question, options }),
            }
        }
        let practice = build_v3_practice_spec(entry).map(|spec| Practice {
            prompt: spec.prompt,
            source: self
                .progress
                .exercise_answers
                .get(&format!("practice:{}", entry.id))
                .cloned()
                .unwrap_or_else(|| spec.starter.into()),
            hints: spec.hints[..self.shown_hints(&spec)].to_vec(),
        });
        Ok(Lesson {
            entry: self.entry(entry),
            stage: entry.stage.label(),
            minutes: lesson.estimated_minutes,
            content,
            questions,
            practice,
        })
    }

    pub fn answer(&mut self, id: &str, picks: &[usize]) -> Result<QuizResult, String> {
        let entry = self.accessible_entry(id)?;
        let lesson = build_v3_lesson(entry, &self.curriculum);
        let questions: Vec<_> = lesson
            .blocks
            .into_iter()
            .filter_map(|block| match block {
                Block::Quiz {
                    options,
                    correct,
                    why,
                    ..
                } => Some((options.len(), correct, why)),
                _ => None,
            })
            .collect();
        if questions.len() != picks.len()
            || picks.iter().zip(&questions).any(|(pick, q)| *pick >= q.0)
        {
            return Err("Selecione uma opção em cada pergunta.".into());
        }
        let correct: Vec<_> = picks
            .iter()
            .zip(&questions)
            .map(|(pick, q)| *pick == q.1)
            .collect();
        let explanations = questions
            .into_iter()
            .zip(&correct)
            .map(|((_, _, why), correct)| if *correct { why } else { None })
            .collect();
        let all_correct = correct.iter().all(|correct| *correct);
        let mut awarded = 0;
        if all_correct {
            let mut next = self.progress.clone();
            next.read.insert(id.into());
            if entry.kind == V3EntryKind::Theory && !is_v3_entry_completed(&next, entry) {
                next.completed.insert(id.into());
                if let std::collections::hash_map::Entry::Vacant(event) =
                    next.xp_events.entry(format!("lesson:{id}"))
                {
                    event.insert(20);
                    awarded = 20;
                }
                next.record_srs_mastery(id, true);
            }
            self.commit(next)?;
        }
        Ok(QuizResult {
            correct,
            explanations,
            awarded,
            completed: is_v3_entry_completed(&self.progress, entry),
            ferris: if all_correct {
                FerrisMood::Happy.ascii()
            } else {
                FerrisMood::Encouraging.ascii()
            },
        })
    }

    pub fn practice_spec(&self, id: &str) -> Result<PracticeSpec, String> {
        build_v3_practice_spec(self.accessible_entry(id)?)
            .ok_or("Esta lição não tem prática obrigatória.".into())
    }

    pub fn save_draft(&mut self, id: &str, source: String) -> Result<(), String> {
        self.practice_spec(id)?;
        if source.len() > MAX_SOURCE {
            return Err("O rascunho excede 256 KiB.".into());
        }
        let mut next = self.progress.clone();
        next.exercise_answers
            .insert(format!("practice:{id}"), source);
        self.commit(next)
    }

    fn shown_hints(&self, spec: &PracticeSpec) -> usize {
        usize::from(
            *self
                .progress
                .hints_used
                .get(&format!("practice:{}", spec.id))
                .unwrap_or(&0),
        )
        .min(spec.hints.len())
    }

    pub fn reveal_hint(&mut self, id: &str) -> Result<Vec<&'static str>, String> {
        let spec = self.practice_spec(id)?;
        let shown = (self.shown_hints(&spec) + 1).min(spec.hints.len());
        let mut next = self.progress.clone();
        next.hints_used
            .insert(format!("practice:{}", spec.id), shown as u8);
        self.commit(next)?;
        Ok(spec.hints[..shown].to_vec())
    }

    pub fn pass_practice(&mut self, spec: &PracticeSpec) -> Result<u32, String> {
        let mut next = self.progress.clone();
        let (base, bonus) = calculate_v3_practice_xp(self.shown_hints(spec), false);
        next.read.insert(spec.lesson_id.into());
        next.exercises_completed.insert(spec.id.into());
        let mut awarded = 0;
        for (key, amount) in [
            (format!("practice:{}", spec.id), base),
            (format!("practice:{}:independent", spec.id), bonus),
        ] {
            if amount > 0
                && let std::collections::hash_map::Entry::Vacant(event) = next.xp_events.entry(key)
            {
                event.insert(amount);
                awarded += amount;
            }
        }
        next.record_srs_mastery(spec.id, bonus > 0);
        self.commit(next)?;
        Ok(awarded)
    }

    fn commit(&mut self, progress: Progress) -> Result<(), String> {
        storage::save_to(&progress.to_data(), &self.path).map_err(|error| error.to_string())?;
        self.progress = progress;
        self.warning = None;
        Ok(())
    }

    pub fn import(&mut self, source: &Path) -> Result<Option<String>, String> {
        let mut bytes = Vec::new();
        File::open(source)
            .and_then(|file| file.take((MAX_IMPORT + 1) as u64).read_to_end(&mut bytes))
            .map_err(|error| error.to_string())?;
        if bytes.len() > MAX_IMPORT {
            return Err("O arquivo de progresso excede 64 MiB.".into());
        }
        let text =
            std::str::from_utf8(&bytes).map_err(|_| "O progresso precisa estar em UTF-8.")?;
        let root: toml::Table =
            toml::from_str(text).map_err(|_| "Arquivo de progresso inválido.")?;
        if !root.keys().any(|key| {
            matches!(
                key.as_str(),
                "version"
                    | "read_lessons"
                    | "completed_lessons"
                    | "completed_exercises"
                    | "xp_events"
                    | "exercise_answers"
                    | "hints_used"
                    | "srs_cards"
                    | "project_milestones"
                    | "error_counts"
                    | "onboarding_completed"
            )
        }) {
            return Err("O arquivo não contém dados de progresso RustLab.".into());
        }
        let data: ProgressData =
            toml::from_str(text).map_err(|_| "Arquivo de progresso inválido.")?;
        if data.version > 1
            || data
                .srs_cards
                .values()
                .map(|card| u64::from(card.review_count))
                .sum::<u64>()
                >= u64::from(u32::MAX)
        {
            return Err("Versão ou contadores de progresso incompatíveis.".into());
        }
        let backup = match fs::read(&self.path) {
            Ok(old) => {
                let parent = self.path.parent().ok_or("Pasta de dados indisponível.")?;
                let path = tempfile::Builder::new()
                    .prefix("progresso-anterior-")
                    .suffix(".toml")
                    .tempfile_in(parent)
                    .and_then(|mut file| {
                        file.write_all(&old)?;
                        file.as_file().sync_all()?;
                        file.into_temp_path().keep().map_err(|error| error.error)
                    })
                    .map_err(|error| error.to_string())?;
                Some(path.display().to_string())
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.to_string()),
        };
        self.commit(Progress::from_data(data))?;
        Ok(backup)
    }

    pub fn export(&self, destination: &Path) -> Result<(), String> {
        if self.warning.is_some() {
            return Err("Recupere o progresso local antes de exportar.".into());
        }
        let parent = destination
            .parent()
            .ok_or("Pasta de destino indisponível.")?;
        let text =
            toml::to_string_pretty(&self.progress.to_data()).map_err(|error| error.to_string())?;
        let mut file =
            tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
        file.write_all(text.as_bytes())
            .and_then(|()| file.as_file().sync_all())
            .map_err(|error| error.to_string())?;
        file.persist_noclobber(destination)
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enforces_prerequisites_validates_quiz_and_persists_awards_once() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progresso.toml");
        let mut course = Course::load(path.clone()).unwrap();
        assert_eq!(
            course
                .map()
                .stages
                .iter()
                .map(|s| s.entries.len())
                .sum::<usize>(),
            213
        );
        assert!(course.lesson("B-T02").is_err());
        let picks: Vec<_> = build_v3_lesson(find_v3_entry("B-T01").unwrap(), &course.curriculum)
            .blocks
            .into_iter()
            .filter_map(|block| match block {
                Block::Quiz { correct, .. } => Some(correct),
                _ => None,
            })
            .collect();
        assert!(!picks.is_empty());
        assert_eq!(course.answer("B-T01", &picks).unwrap().awarded, 20);
        assert_eq!(course.answer("B-T01", &picks).unwrap().awarded, 0);
        let mut restored = Course::load(path).unwrap();
        assert!(restored.lesson("B-T02").is_ok());
        let export = dir.path().join("export.toml");
        restored.export(&export).unwrap();
        assert!(restored.export(&export).is_err());
        assert!(restored.import(&export).unwrap().is_some());
        assert_eq!(restored.map().xp, 20);
        fs::write(&export, "version = 64").unwrap();
        assert!(restored.import(&export).is_err());
        assert_eq!(restored.map().xp, 20);
    }
}
