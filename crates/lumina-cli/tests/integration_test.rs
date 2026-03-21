use std::process::Command;

fn lumina_bin() -> std::path::PathBuf {
    let path = std::path::PathBuf::from(env!("CARGO_BIN_EXE_lumina-cli"));
    path
}

fn spec_path(name: &str) -> String {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest.parent().unwrap().parent().unwrap();
    workspace_root.join("tests").join("spec").join(name)
        .to_string_lossy().to_string()
}

#[test]
fn test_fleet_lum_runs_without_error() {
    let output = Command::new(lumina_bin())
        .args(["run", &spec_path("fleet.lum")])
        .output()
        .expect("failed to run lumina");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(),
        "lumina run failed:\n{stderr}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Moto"), "expected Moto in output:\n{stdout}");
    assert!(stdout.contains("battery"), "expected battery in output:\n{stdout}");
}

#[test]
fn test_errors_lum_fails_check_with_l003() {
    let output = Command::new(lumina_bin())
        .args(["check", &spec_path("errors.lum")])
        .output()
        .expect("failed to run lumina check");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("L003"),
        "Expected L003 error, got:\n{stderr}");
}

#[test]
fn test_derived_fields_recompute() {
    let dir = std::env::temp_dir();
    let path = dir.join("lumina_test_derived.lum");
    std::fs::write(&path, concat!(
        "entity Sensor {\n",
        "  temp: Number\n",
        "  isHot := temp > 30\n",
        "}\n",
        "let Sensor = Sensor { temp: 25 }\n",
        "update Sensor.temp to 35\n",
        "show Sensor.isHot\n",
    )).unwrap();

    let output = Command::new(lumina_bin())
        .args(["run", &path.to_string_lossy()])
        .output()
        .expect("failed to run lumina");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "failed:\n{stderr}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // After update temp=35, isHot should be true
    assert!(stdout.contains("true"), "expected isHot=true in:\n{stdout}");

    std::fs::remove_file(path).ok();
}

#[test]
fn test_on_clear_fires_when_condition_recovers() {
    let dir = std::env::temp_dir();
    let path = dir.join("lumina_test_on_clear.lum");
    std::fs::write(&path, concat!(
        "entity Sensor {\n",
        "  temp: Number\n",
        "}\n",
        "let s1 = Sensor { temp: 20 }\n",
        "rule \"overheat\" {\n",
        "  when Sensor.temp > 40\n",
        "  then show \"ALERT: overheating\"\n",
        "  on clear {\n",
        "    then show \"RESOLVED: temperature normal\"\n",
        "  }\n",
        "}\n",
        "update s1.temp to 50\n",
        "update s1.temp to 30\n",
    )).unwrap();

    let output = Command::new(lumina_bin())
        .args(["run", &path.to_string_lossy()])
        .output()
        .expect("failed to run lumina");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "failed:\n{stderr}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ALERT: overheating"),
        "expected alert message in:\n{stdout}");
    assert!(stdout.contains("RESOLVED: temperature normal"),
        "expected on_clear resolved message in:\n{stdout}");

    std::fs::remove_file(path).ok();
}
