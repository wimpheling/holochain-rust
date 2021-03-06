use colored::*;
use crate::{cli::package, error::DefaultResult, util};
use std::{fs, path::PathBuf};

pub const TEST_DIR_NAME: &str = "test";
pub const DIST_DIR_NAME: &str = "dist";

pub fn test(
    path: &PathBuf,
    tests_folder: &str,
    testfile: &str,
    skip_build: bool,
) -> DefaultResult<()> {
    // create dist folder
    let dist_path = path.join(&DIST_DIR_NAME);

    if !dist_path.exists() {
        fs::create_dir(dist_path.as_path())?;
    }

    if !skip_build {
        // build the package file, within the dist folder
        let bundle_file_path = dist_path.join(package::DEFAULT_BUNDLE_FILE_NAME);
        println!(
            "{} files for testing to file: {:?}",
            "Packaging".green().bold(),
            bundle_file_path
        );
        package(true, Some(bundle_file_path.to_path_buf()))?;
    }

    // build tests
    let tests_path = path.join(&tests_folder);
    ensure!(
        tests_path.exists(),
        "Directory {} does not exist",
        tests_folder
    );

    // npm install, if no node_modules yet
    let node_modules_path = tests_path.join("node_modules");
    if !node_modules_path.exists() {
        // CLI feedback
        println!("{}", "Installing node_modules".green().bold());
        util::run_cmd(
            tests_path.clone(),
            "npm".to_string(),
            vec!["install".to_string(), "--silent".to_string()],
        )?;
    }

    // execute the built test file using node
    // CLI feedback
    println!("{} tests in {}", "Running".green().bold(), testfile,);
    util::run_cmd(
        path.to_path_buf(),
        "node".to_string(),
        vec![testfile.to_string()],
    )?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use assert_cmd::prelude::*;
    use crate::cli::package;
    use std::process::Command;
    use tempfile::{Builder, TempDir};

    const HOLOCHAIN_TEST_PREFIX: &str = "org.holochain.test";

    fn gen_dir() -> TempDir {
        Builder::new()
            .prefix(HOLOCHAIN_TEST_PREFIX)
            .tempdir()
            .unwrap()
    }

    #[test]
    fn test_command_basic_test() {
        let temp_space = gen_dir();
        let temp_dir_path = temp_space.path();
        let temp_dir_path_buf = temp_space.path().to_path_buf();

        // do init first, so theres a project
        Command::main_binary()
            .unwrap()
            .args(&["init", temp_dir_path.to_str().unwrap()])
            .assert()
            .success();

        test(&temp_dir_path_buf, &TEST_DIR_NAME, "test/index.js", false)
            .unwrap_or_else(|e| panic!("test call failed: {}", e));

        // check success of packaging step
        assert!(
            temp_dir_path_buf
                .join(&DIST_DIR_NAME)
                .join(package::DEFAULT_BUNDLE_FILE_NAME)
                .exists()
        );
        // check success of npm install step
        assert!(
            temp_dir_path_buf
                .join(&TEST_DIR_NAME)
                .join("node_modules")
                .exists()
        );
    }

    #[test]
    fn test_command_no_test_folder() {
        let temp_space = gen_dir();
        let temp_dir_path = temp_space.path();
        let temp_dir_path_buf = temp_space.path().to_path_buf();

        // do init first, so theres a project
        Command::main_binary()
            .unwrap()
            .args(&["init", temp_dir_path.to_str().unwrap()])
            .assert()
            .success();

        let result = test(&temp_dir_path_buf, "west", "test/index.js", false);

        // should err because "west" directory doesn't exist
        assert!(result.is_err());
    }
}
