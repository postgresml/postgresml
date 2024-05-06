use pgrx::*;
use serde::Deserialize;

#[derive(PostgresEnum, Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Task {
    regression,
    classification,
    decomposition,
    clustering,
    question_answering,
    summarization,
    translation,
    text_classification,
    text_generation,
    text2text,
    embedding,
    text_pair_classification,
    conversation,
}

// unfortunately the pgrx macro expands the enum names to underscore, but hugging face uses dash
impl Task {
    pub fn to_pg_enum(&self) -> String {
        match *self {
            Task::regression => "regression".to_string(),
            Task::classification => "classification".to_string(),
            Task::decomposition => "decomposition".to_string(),
            Task::clustering => "clustering".to_string(),
            Task::question_answering => "question_answering".to_string(),
            Task::summarization => "summarization".to_string(),
            Task::translation => "translation".to_string(),
            Task::text_classification => "text_classification".to_string(),
            Task::text_generation => "text_generation".to_string(),
            Task::text2text => "text2text".to_string(),
            Task::embedding => "embedding".to_string(),
            Task::text_pair_classification => "text_pair_classification".to_string(),
            Task::conversation => "conversation".to_string(),
        }
    }

    pub fn is_supervised(&self) -> bool {
        matches!(self, Task::regression | Task::classification)
    }

    pub fn default_target_metric(&self) -> String {
        match self {
            Task::regression => "r2",
            Task::classification => "f1",
            Task::decomposition => "cumulative_explained_variance",
            Task::clustering => "silhouette",
            Task::question_answering => "f1",
            Task::translation => "blue",
            Task::summarization => "rouge_ngram_f1",
            Task::text_classification => "f1",
            Task::text_generation => "perplexity",
            Task::text2text => "perplexity",
            Task::embedding => error!("No default target metric for embedding task"),
            Task::text_pair_classification => "f1",
            Task::conversation => "bleu",
        }
        .to_string()
    }

    pub fn default_target_metric_positive(&self) -> bool {
        match self {
            Task::regression => true,
            Task::classification => true,
            Task::decomposition => true,
            Task::clustering => true,
            Task::question_answering => true,
            Task::translation => true,
            Task::summarization => true,
            Task::text_classification => true,
            Task::text_generation => false,
            Task::text2text => false,
            Task::embedding => error!("No default target metric positive for embedding task"),
            Task::text_pair_classification => true,
            Task::conversation => true,
        }
    }

    pub fn value_is_better(&self, value: f64, other: f64) -> bool {
        if self.default_target_metric_positive() {
            value > other
        } else {
            value < other
        }
    }

    pub fn default_target_metric_sql_order(&self) -> String {
        let direction = if self.default_target_metric_positive() {
            "DESC"
        } else {
            "ASC"
        };
        format!(
            "ORDER BY models.metrics->>'{}' {} NULLS LAST",
            self.default_target_metric(),
            direction
        )
    }
}

impl std::str::FromStr for Task {
    type Err = ();

    fn from_str(input: &str) -> Result<Task, Self::Err> {
        match input {
            "regression" => Ok(Task::regression),
            "classification" => Ok(Task::classification),
            "decomposition" => Ok(Task::decomposition),
            "clustering" => Ok(Task::clustering),
            "question-answering" | "question_answering" => Ok(Task::question_answering),
            "summarization" => Ok(Task::summarization),
            "translation" => Ok(Task::translation),
            "text-classification" | "text_classification" => Ok(Task::text_classification),
            "text-generation" | "text_generation" => Ok(Task::text_generation),
            "text2text" => Ok(Task::text2text),
            "text-pair-classification" | "text_pair_classification" => Ok(Task::text_pair_classification),
            "conversation" => Ok(Task::conversation),
            _ => Err(()),
        }
    }
}

impl std::string::ToString for Task {
    fn to_string(&self) -> String {
        match *self {
            Task::regression => "regression".to_string(),
            Task::classification => "classification".to_string(),
            Task::decomposition => "decomposition".to_string(),
            Task::clustering => "clustering".to_string(),
            Task::question_answering => "question-answering".to_string(),
            Task::summarization => "summarization".to_string(),
            Task::translation => "translation".to_string(),
            Task::text_classification => "text-classification".to_string(),
            Task::text_generation => "text-generation".to_string(),
            Task::text2text => "text2text".to_string(),
            Task::embedding => "embedding".to_string(),
            Task::text_pair_classification => "text-pair-classification".to_string(),
            Task::conversation => "conversation".to_string(),
        }
    }
}
