use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs::read_to_string;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("cargo-pgml-components").unwrap();

    cmd.arg("pgml-components")
        .arg("add")
        .arg("component")
        .arg("--help");

    cmd.assert().success();
}

#[test]
fn test_add_component() {
    let mut cmd = Command::cargo_bin("cargo-pgml-components").unwrap();
    let temp = assert_fs::TempDir::new().unwrap();

    cmd.arg("pgml-components")
        .arg("--project-path")
        .arg(temp.path().display().to_string())
        .arg("add")
        .arg("component")
        .arg("test_component");

    cmd.assert().success();

    for path in [
        "mod.rs",
        "template.html",
        "test_component.scss",
        "test_component_controller.js",
    ] {
        temp.child(&format!("src/components/test_component/{}", path))
            .assert(predicate::path::exists());
    }

    let rust = read_to_string(temp.child("src/components/test_component/mod.rs").path()).unwrap();
    assert!(rust.contains("pub struct TestComponent {"));

    let js = read_to_string(
        temp.child("src/components/test_component/test_component_controller.js")
            .path(),
    )
    .unwrap();
    assert!(js.contains("export default class extends Controller"));
    assert!(js.contains("console.log('Initialized test-component')"));

    let html = read_to_string(
        temp.child("src/components/test_component/template.html")
            .path(),
    )
    .unwrap();
    assert!(html.contains("<div data-controller=\"test-component\">"));
}

#[test]
fn test_add_upper_camel() {
    let temp = assert_fs::TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("cargo-pgml-components").unwrap();
    cmd.arg("pgml-components")
        .arg("--project-path")
        .arg(temp.path().display().to_string())
        .arg("add")
        .arg("component")
        .arg("TestComponent");

    cmd.assert().success();

    for path in [
        "mod.rs",
        "template.html",
        "test_component.scss",
        "test_component_controller.js",
    ] {
        temp.child(&format!("src/components/test_component/{}", path))
            .assert(predicate::path::exists());
    }

    let rust = read_to_string(temp.child("src/components/test_component/mod.rs").path()).unwrap();
    assert!(rust.contains("pub struct TestComponent {"));

    let js = read_to_string(
        temp.child("src/components/test_component/test_component_controller.js")
            .path(),
    )
    .unwrap();
    assert!(js.contains("export default class extends Controller"));
    assert!(js.contains("console.log('Initialized test-component')"));

    let html = read_to_string(
        temp.child("src/components/test_component/template.html")
            .path(),
    )
    .unwrap();
    assert!(html.contains("<div data-controller=\"test-component\">"));

    let mut cmd = Command::cargo_bin("cargo-pgml-components").unwrap();
    cmd.arg("pgml-components")
        .arg("--project-path")
        .arg(temp.path().display().to_string())
        .arg("add")
        .arg("component")
        .arg("RandomTest/Hello/snake_path/CamelComponent");
    cmd.assert().success();

    for path in [
        "mod.rs",
        "template.html",
        "camel_component.scss",
        "camel_component_controller.js",
    ] {
        temp.child(&format!(
            "src/components/random_test/hello/snake_path/camel_component/{}",
            path
        ))
        .assert(predicate::path::exists());
    }

    let js = temp.child(
        "src/components/random_test/hello/snake_path/camel_component/camel_component_controller.js",
    );

    let js = read_to_string(js.path()).unwrap();
    assert!(js.contains("export default class extends Controller"));
    assert!(js.contains("console.log('Initialized random-test-hello-snake-path-camel-component')"));

    let html = read_to_string(
        temp.child("src/components/random_test/hello/snake_path/camel_component/template.html")
            .path(),
    )
    .unwrap();
    assert!(html.contains("<div data-controller=\"random-test-hello-snake-path-camel-component\">"));

    let rust = read_to_string(
        temp.child("src/components/random_test/hello/snake_path/camel_component/mod.rs")
            .path(),
    )
    .unwrap();
    assert!(rust.contains("pub struct CamelComponent {"));
    assert!(rust.contains("impl CamelComponent {"));
}

#[test]
fn test_add_subcomponent() {
    let mut cmd = Command::cargo_bin("cargo-pgml-components").unwrap();
    let temp = assert_fs::TempDir::new().unwrap();

    cmd.arg("pgml-components")
        .arg("--project-path")
        .arg(temp.path().display().to_string())
        .arg("add")
        .arg("component")
        .arg("test_component/subcomponent/alpha");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("written src/components/mod.rs"))
        .stdout(predicate::str::contains(
            "written src/components/test_component/mod.rs",
        ));

    for path in [
        "mod.rs",
        "template.html",
        "alpha.scss",
        "alpha_controller.js",
    ] {
        temp.child(&format!(
            "src/components/test_component/subcomponent/alpha/{}",
            path
        ))
        .assert(predicate::path::exists());
    }

    // Try to add a component in a folder that already has one.
    let mut cmd = Command::cargo_bin("cargo-pgml-components").unwrap();
    cmd.arg("pgml-components")
        .arg("--project-path")
        .arg(temp.path().display().to_string())
        .arg("add")
        .arg("component")
        .arg("test_component/subcomponent/alpha/beta");

    cmd.assert().failure().stdout(predicate::str::contains(
        "component cannot be placed into a directory that has a component already",
    ));

    // Try one deeper
    let mut cmd = Command::cargo_bin("cargo-pgml-components").unwrap();
    cmd.arg("pgml-components")
        .arg("--project-path")
        .arg(temp.path().display().to_string())
        .arg("add")
        .arg("component")
        .arg("test_component/subcomponent/alpha/beta/theta");

    cmd.assert().failure().stdout(predicate::str::contains(
        "component cannot be placed into a directory that has a component already",
    ));
}

#[test]
fn test_component_with_dashes() {
    let mut cmd = Command::cargo_bin("cargo-pgml-components").unwrap();
    let temp = assert_fs::TempDir::new().unwrap();

    cmd.arg("pgml-components")
        .arg("--project-path")
        .arg(temp.path().display().to_string())
        .arg("add")
        .arg("component")
        .arg("test-component/subcomponent/alpha-beta-gamma");

    cmd.assert().success();

    for path in [
        "mod.rs",
        "template.html",
        "alpha_beta_gamma.scss",
        "alpha_beta_gamma_controller.js",
    ] {
        temp.child(&format!(
            "src/components/test_component/subcomponent/alpha_beta_gamma/{}",
            path
        ))
        .assert(predicate::path::exists());
    }

    let rust = read_to_string(
        temp.child("src/components/test_component/subcomponent/alpha_beta_gamma/mod.rs")
            .path(),
    )
    .unwrap();

    assert!(rust.contains("pub struct AlphaBetaGamma {"));

    let js = read_to_string(
        temp.child(
            "src/components/test_component/subcomponent/alpha_beta_gamma/alpha_beta_gamma_controller.js",
        )
        .path(),
    ).unwrap();

    assert!(js.contains("export default class extends Controller"));
    assert!(js.contains("console.log('Initialized test-component-subcomponent-alpha-beta-gamma')"));

    let html = read_to_string(
        temp.child("src/components/test_component/subcomponent/alpha_beta_gamma/template.html")
            .path(),
    )
    .unwrap();

    assert!(html.contains("<div data-controller=\"test-component-subcomponent-alpha-beta-gamma\">"));

    for path in [
        "test_component/subcomponent/mod.rs",
        "test_component/mod.rs",
    ] {
        temp.child(&format!("src/components/{}", path))
            .assert(predicate::path::exists());

        let file = read_to_string(temp.child(&format!("src/components/{}", path)).path()).unwrap();
        assert!(file.contains("pub mod"));
    }
}

#[test]
fn test_invalid_component_names() {
    let temp = assert_fs::TempDir::new().unwrap();
    for name in ["5_starts_with_a_number", "has%_special_characters"] {
        let mut cmd = Command::cargo_bin("cargo-pgml-components").unwrap();

        cmd.arg("pgml-components")
            .arg("--project-path")
            .arg(temp.path().display().to_string())
            .arg("add")
            .arg("component")
            .arg(name);

        cmd.assert()
            .failure()
            .stdout(predicate::str::contains("component name is not valid"));
    }
}
