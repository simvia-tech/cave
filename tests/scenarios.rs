use assert_cmd::Command;
use tempfile::tempdir;
use std::fs;
use predicates::prelude::*;

#[test]
fn test_available_versions() {
    let mut cmd = Command::cargo_bin("cave").expect("cave binary should be built");

    let output = cmd.arg("available")
        .assert()
        .success()
        .get_output()
        .stdout.clone();

    let stdout_str = String::from_utf8(output).expect("output not utf8");

    let count_versions = stdout_str
        .lines()
        .filter(|line| line.contains('.'))
        .count();

    assert!(count_versions >= 10, "Expected at least 10 versions, got {}", count_versions);
}

#[test]
fn test_use_sets_global_version() {
    let temp_home = tempdir().expect("create temp dir");
    let cave_file_path = temp_home.path().join(".cave");

    let mut cmd = Command::cargo_bin("cave").expect("binary built");
    cmd.env("HOME", temp_home.path())
        .arg("use")
        .arg("17.3.1")
        .assert()
        .success();

    assert!(cave_file_path.exists(), ".cave file should exist");
    let content = fs::read_to_string(&cave_file_path).expect("read .cave");
    assert!(content.contains("17.3.1"), "Global version file should contain 17.3.1");
}

#[test]
fn test_config_enable_auto_update() {
    use serde_json::Value;

    let temp_home = tempdir().expect("create temp dir");
    let config_path = temp_home.path().join(".caveconfig.json");

    let mut cmd = Command::cargo_bin("cave").expect("binary built");
    cmd.env("HOME", temp_home.path())
        .arg("config")
        .arg("enable-auto-update")
        .assert()
        .success();

    assert!(config_path.exists(), ".caveconfig.json file should exist");

    let content = fs::read_to_string(&config_path).expect("read config file");

    let json: Value = serde_json::from_str(&content).expect("valid JSON config");
    assert_eq!(
        json["auto_update"],
        Value::Bool(true),
        "Config file should enable auto-update"
    );
}


#[test]
fn test_error_on_unknown_version_use_and_pin() {
    let mut cmd_use = Command::cargo_bin("cave").expect("binary built");
    cmd_use.arg("use")
        .arg("99.99.99")
        .assert()
        .failure()  // on attend un échec
        .stderr(predicate::str::contains("Version '99.99.99' is not available"));

    let temp_dir = tempdir().expect("create temp dir");
    let mut cmd_pin = Command::cargo_bin("cave").expect("binary built");
    cmd_pin.current_dir(temp_dir.path())
        .arg("pin")
        .arg("99.99.99")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Version '99.99.99' is not available"));
}



