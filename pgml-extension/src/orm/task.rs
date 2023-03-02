use pgx::*;
use serde::Deserialize;

#[derive(PostgresEnum, Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Task {
    regression,
    classification,
    question_answering,
    summarization,
    translation,
    text_classification,
    text_generation,
    text2text,
}

impl std::str::FromStr for Task {
    type Err = ();

    fn from_str(input: &str) -> Result<Task, Self::Err> {
        match input {
            "regression" => Ok(Task::regression),
            "classification" => Ok(Task::classification),
            "question_answering" => Ok(Task::question_answering),
            "summarization" => Ok(Task::summarization),
            "translation" => Ok(Task::translation),
            "text_classification" => Ok(Task::text_classification),
            "text_generation" => Ok(Task::text_generation),
            "text2text" => Ok(Task::text2text),
            _ => Err(()),
        }
    }
}

impl std::string::ToString for Task {
    fn to_string(&self) -> String {
        match *self {
            Task::regression => "regression".to_string(),
            Task::classification => "classification".to_string(),
            Task::question_answering => "question_answering".to_string(),
            Task::summarization => "summarization".to_string(),
            Task::translation => "translation".to_string(),
            Task::text_classification => "text_classification".to_string(),
            Task::text_generation => "text_generation".to_string(),
            Task::text2text => "text2text".to_string(),
        }
    }
}
