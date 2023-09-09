use crate::frontend::components::*;
use std::path::Path;

#[test]
fn test_component() {
    let component = Component::new("test_component", Path::new("frontend/pages/test_component"));
    assert_eq!(
        component.rust_path(),
        "crate::components::frontend::pages::TestComponent"
    );

    assert_eq!(
        component.full_path(),
        Path::new("src/components/frontend/pages/test_component")
    );

    assert_eq!(component.controller_name(), "frontend-pages-test-component",);
    assert_eq!(
        component.frame_url(),
        "/frames/frontend/pages/test-component"
    );
}
