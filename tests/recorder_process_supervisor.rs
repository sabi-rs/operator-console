use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::thread;
use std::time::Duration;

use operator_console::recorder::{ProcessRecorderSupervisor, RecorderConfig, RecorderSupervisor};

#[test]
fn process_supervisor_writes_watcher_output_to_run_dir_log() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let run_dir = temp_dir.path().join("run");
    let script_path = temp_dir.path().join("fake-recorder.sh");

    write_executable_script(
        &script_path,
        "#!/bin/sh\necho watcher stdout diagnostics\necho watcher stderr diagnostics >&2\nsleep 1\n",
    )
    .expect("write script");

    let mut supervisor = ProcessRecorderSupervisor::default();
    supervisor
        .start(&RecorderConfig {
            command: script_path,
            run_dir: run_dir.clone(),
            session: String::from("helium-copy"),
            companion_legs_path: None,
            profile_path: Some(std::path::PathBuf::from("/tmp/owned-profile")),
            autostart: false,
            interval_seconds: 5,
            commission_rate: String::from("0"),
            target_profit: String::from("1"),
            stop_loss: String::from("1"),
            hard_margin_call_profit_floor: String::new(),
            warn_only_default: true,
        })
        .expect("start supervisor");

    thread::sleep(Duration::from_millis(100));

    let log_path = run_dir.join("watcher.log");
    let log_contents = fs::read_to_string(log_path).expect("read watcher log");
    assert!(log_contents.contains("watcher stdout diagnostics"));
    assert!(log_contents.contains("watcher stderr diagnostics"));

    supervisor.stop().expect("stop supervisor");
}

#[test]
fn process_supervisor_clears_stale_run_dir_state_before_start() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let run_dir = temp_dir.path().join("run");
    let screenshots_dir = run_dir.join("screenshots");
    let script_path = temp_dir.path().join("fake-recorder.sh");

    fs::create_dir_all(&screenshots_dir).expect("create screenshots dir");
    fs::write(run_dir.join("watcher-state.json"), "stale").expect("write stale state");
    fs::write(run_dir.join("events.jsonl"), "stale-event\n").expect("write stale events");
    fs::write(screenshots_dir.join("stale.png"), "old").expect("write stale screenshot");

    write_executable_script(&script_path, "#!/bin/sh\nsleep 1\n").expect("write script");

    let mut supervisor = ProcessRecorderSupervisor::default();
    supervisor
        .start(&RecorderConfig {
            command: script_path,
            run_dir: run_dir.clone(),
            session: String::from("helium-copy"),
            companion_legs_path: None,
            profile_path: Some(std::path::PathBuf::from("/tmp/owned-profile")),
            autostart: false,
            interval_seconds: 5,
            commission_rate: String::from("0"),
            target_profit: String::from("1"),
            stop_loss: String::from("1"),
            hard_margin_call_profit_floor: String::new(),
            warn_only_default: true,
        })
        .expect("start supervisor");

    assert!(!run_dir.join("watcher-state.json").exists());
    assert!(!run_dir.join("events.jsonl").exists());
    assert!(!screenshots_dir.join("stale.png").exists());

    supervisor.stop().expect("stop supervisor");
}

#[test]
fn process_supervisor_passes_owned_profile_path_to_watcher_process() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let run_dir = temp_dir.path().join("run");
    let script_path = temp_dir.path().join("fake-recorder.sh");

    write_executable_script(&script_path, "#!/bin/sh\nprintf '%s\n' \"$@\"\nsleep 1\n")
        .expect("write script");

    let mut supervisor = ProcessRecorderSupervisor::default();
    supervisor
        .start(&RecorderConfig {
            command: script_path,
            run_dir: run_dir.clone(),
            session: String::from("helium-copy"),
            companion_legs_path: None,
            profile_path: Some(std::path::PathBuf::from("/tmp/owned-profile")),
            autostart: false,
            interval_seconds: 5,
            commission_rate: String::from("0"),
            target_profit: String::from("1"),
            stop_loss: String::from("1"),
            hard_margin_call_profit_floor: String::new(),
            warn_only_default: true,
        })
        .expect("start supervisor");

    thread::sleep(Duration::from_millis(100));

    let log_path = run_dir.join("watcher.log");
    let log_contents = fs::read_to_string(log_path).expect("read watcher log");
    assert!(log_contents.contains("--profile-path"));
    assert!(log_contents.contains("/tmp/owned-profile"));

    supervisor.stop().expect("stop supervisor");
}

#[test]
fn process_supervisor_restores_stale_run_dir_if_watcher_fails_to_spawn() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let run_dir = temp_dir.path().join("run");
    let screenshots_dir = run_dir.join("screenshots");
    let missing_command = temp_dir.path().join("missing-recorder.sh");

    fs::create_dir_all(&screenshots_dir).expect("create screenshots dir");
    fs::write(run_dir.join("watcher-state.json"), "stale").expect("write stale state");
    fs::write(run_dir.join("events.jsonl"), "stale-event\n").expect("write stale events");
    fs::write(run_dir.join("watcher.log"), "previous log\n").expect("write stale log");
    fs::write(screenshots_dir.join("stale.png"), "old").expect("write stale screenshot");

    let mut supervisor = ProcessRecorderSupervisor::default();
    let error = supervisor
        .start(&RecorderConfig {
            command: missing_command,
            run_dir: run_dir.clone(),
            session: String::from("helium-copy"),
            companion_legs_path: None,
            profile_path: Some(std::path::PathBuf::from("/tmp/owned-profile")),
            autostart: false,
            interval_seconds: 5,
            commission_rate: String::from("0"),
            target_profit: String::from("1"),
            stop_loss: String::from("1"),
            hard_margin_call_profit_floor: String::new(),
            warn_only_default: true,
        })
        .expect_err("start should fail");

    assert!(error
        .to_string()
        .contains("failed to start recorder watcher"));
    assert_eq!(
        fs::read_to_string(run_dir.join("watcher-state.json")).expect("restored state"),
        "stale"
    );
    assert_eq!(
        fs::read_to_string(run_dir.join("events.jsonl")).expect("restored events"),
        "stale-event\n"
    );
    assert_eq!(
        fs::read_to_string(run_dir.join("watcher.log")).expect("restored log"),
        "previous log\n"
    );
    assert!(screenshots_dir.join("stale.png").exists());
}

#[test]
fn process_supervisor_restores_stale_run_dir_if_watcher_exits_immediately() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let run_dir = temp_dir.path().join("run");
    let screenshots_dir = run_dir.join("screenshots");
    let script_path = temp_dir.path().join("failing-recorder.sh");

    fs::create_dir_all(&screenshots_dir).expect("create screenshots dir");
    fs::write(run_dir.join("watcher-state.json"), "stale").expect("write stale state");
    fs::write(run_dir.join("events.jsonl"), "stale-event\n").expect("write stale events");
    fs::write(run_dir.join("watcher.log"), "previous log\n").expect("write stale log");
    fs::write(screenshots_dir.join("stale.png"), "old").expect("write stale screenshot");
    write_executable_script(&script_path, "#!/bin/sh\necho startup failed >&2\nexit 1\n")
        .expect("write script");

    let mut supervisor = ProcessRecorderSupervisor::default();
    let error = supervisor
        .start(&RecorderConfig {
            command: script_path,
            run_dir: run_dir.clone(),
            session: String::from("helium-copy"),
            companion_legs_path: None,
            profile_path: Some(std::path::PathBuf::from("/tmp/owned-profile")),
            autostart: false,
            interval_seconds: 5,
            commission_rate: String::from("0"),
            target_profit: String::from("1"),
            stop_loss: String::from("1"),
            hard_margin_call_profit_floor: String::new(),
            warn_only_default: true,
        })
        .expect_err("start should fail");

    assert!(error.to_string().contains("exited immediately"));
    assert_eq!(
        fs::read_to_string(run_dir.join("watcher-state.json")).expect("restored state"),
        "stale"
    );
    assert_eq!(
        fs::read_to_string(run_dir.join("events.jsonl")).expect("restored events"),
        "stale-event\n"
    );
    assert_eq!(
        fs::read_to_string(run_dir.join("watcher.log")).expect("restored log"),
        "previous log\n"
    );
    assert!(screenshots_dir.join("stale.png").exists());
}

fn write_executable_script(path: &std::path::Path, body: &str) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(body.as_bytes())?;
    file.sync_all()?;
    drop(file);

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}
