use sailfish::TemplateOnce;
use crate::components::component;

#[derive(TemplateOnce, Default)]
#[template(path = "test_component/template.html")]
pub struct TestComponent {
    value: String,
}

impl TestComponent {
    pub fn new() -> TestComponent {
        TestComponent::default()
    }
}

component!(TestComponent);
