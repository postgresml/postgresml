use pgx::*;
use serde::Deserialize;

#[derive(PostgresEnum, Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Task {
    regression,
    classification,
    text_classification,
    question_answering,
    translation,
    summarization,
}

impl std::str::FromStr for Task {
    type Err = ();

    fn from_str(input: &str) -> Result<Task, Self::Err> {
        match input {
            "regression" => Ok(Task::regression),
            "classification" => Ok(Task::classification),
            "text_classification" => Ok(Task::classification),
            "question_answering" => Ok(Task::classification),
            "translation" => Ok(Task::classification),
            "summarization" => Ok(Task::classification),
            _ => Err(()),
        }
    }
}

impl std::string::ToString for Task {
    fn to_string(&self) -> String {
        match *self {
            Task::regression => "regression".to_string(),
            Task::classification => "classification".to_string(),
            Task::text_classification => "text_classification".to_string(),
            Task::question_answering => "question_answering".to_string(),
            Task::translation => "translation".to_string(),
            Task::summarization => "summarization".to_string(),
        }
    }
}
