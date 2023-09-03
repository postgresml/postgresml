use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

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
