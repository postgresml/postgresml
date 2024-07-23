use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "code_editor/editor/template.html")]
pub struct Editor {
    show_model: bool,
    show_task: bool,
    show_question_input: bool,
    task: String,
    model: String,
    btn_location: String,
    btn_style: String,
    is_editable: bool,
    run_on_visible: bool,
    content: Option<String>,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            show_model: false,
            show_task: false,
            show_question_input: false,
            task: "text-generation".to_string(),
            model: "meta-llama/Meta-LLama-3.1-8B-Instruct".to_string(),
            btn_location: "text-area".to_string(),
            btn_style: "party".to_string(),
            is_editable: true,
            run_on_visible: false,
            content: None,
        }
    }

    pub fn new_embedded_query() -> Editor {
        Editor {
            show_model: false,
            show_task: false,
            show_question_input: true,
            task: "embedded-query".to_string(),
            model: "many".to_string(),
            btn_location: "question-header".to_string(),
            btn_style: "secondary".to_string(),
            is_editable: false,
            run_on_visible: false,
            content: None,
        }
    }

    pub fn new_custom(content: &str) -> Editor {
        Editor {
            show_model: false,
            show_task: false,
            show_question_input: false,
            task: "custom".to_string(),
            model: "many".to_string(),
            btn_location: "text-area".to_string(),
            btn_style: "secondary".to_string(),
            is_editable: true,
            run_on_visible: false,
            content: Some(content.to_owned()),
        }
    }

    pub fn set_show_model(mut self, show_model: bool) -> Self {
        self.show_model = show_model;
        self
    }

    pub fn set_show_task(mut self, show_task: bool) -> Self {
        self.show_task = show_task;
        self
    }

    pub fn set_show_question_input(mut self, show_question_input: bool) -> Self {
        self.show_question_input = show_question_input;
        self
    }

    pub fn set_task(mut self, task: &str) -> Self {
        self.task = task.to_owned();
        self
    }

    pub fn set_model(mut self, model: &str) -> Self {
        self.model = model.to_owned();
        self
    }

    pub fn show_btn_in_text_area(mut self) -> Self {
        self.btn_location = "text-area".to_string();
        self
    }

    pub fn set_btn_style_secondary(mut self) -> Self {
        self.btn_style = "secondary".to_string();
        self
    }

    pub fn set_btn_style_party(mut self) -> Self {
        self.btn_style = "party".to_string();
        self
    }

    pub fn set_is_editable(mut self, is_editable: bool) -> Self {
        self.is_editable = is_editable;
        self
    }

    pub fn set_run_on_visible(mut self, run_on_visible: bool) -> Self {
        self.run_on_visible = run_on_visible;
        self
    }

    pub fn set_content(mut self, content: &str) -> Self {
        self.content = Some(content.to_owned());
        self
    }
}

component!(Editor);
