use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
#[ignore]
fn print_number() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("nea")?;

    cmd.arg("file/name");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("it prints!"));

    Ok(())
}
