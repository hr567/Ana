use std::env;
use std::fs;
use std::path;
use std::process;

mod limit;
mod report;

pub use self::{
    limit::Limit,
    report::{LaunchReport, LrunExceed, LrunReport},
};

fn create_lrun_log_file() -> Box<path::Path> {
    let mut lrun_log = path::PathBuf::from(env::var("ANA_WORK_DIR").unwrap());
    lrun_log.push(env::var("ANA_JUDGE_ID").unwrap());
    lrun_log.set_extension("log");
    lrun_log.into_boxed_path()
}

fn create_output_file() -> Box<path::Path> {
    let mut output_file = path::PathBuf::from(env::var("ANA_WORK_DIR").unwrap());
    output_file.push(env::var("ANA_JUDGE_ID").unwrap());
    output_file.set_extension("out");
    output_file.into_boxed_path()
}

pub fn launch(
    executable_file: &path::Path,
    input_file: &path::Path,
    limit: &Limit,
) -> (LaunchReport, LrunReport) {
    let lrun_log = create_lrun_log_file();
    let output_file = create_output_file();

    let status = process::Command::new("sudo") // lrun need root user to be executed
        .arg("bash")
        .arg("-c")
        .arg(format!(
            "{} --max-cpu-time {} --max-real-time {} --max-memory {} --network false {} 3> {}",
            "lrun --uid 65534 --gid 65534", // 65534 is the id of nobody on my computer
            limit.time,
            limit.time + 0.1,
            limit.memory * 1024.0 * 1024.0,
            executable_file
                .to_str()
                .expect("Failed to format executable file"),
            lrun_log.to_str().expect("Failed to format lrun log file"),
        ))
        .stdin(fs::File::open(&input_file).expect("Failed to open input file"))
        .stdout(fs::File::create(&output_file).expect("Failed to create output file"))
        .status()
        .expect("Failed to spawn chile process");

    assert!(status.success(), "Failed to run lrun");

    let report = LrunReport::from_log_file(&lrun_log);

    (
        match report.exceed {
            LrunExceed::Pass => {
                if report.exit_code == 0 {
                    LaunchReport::Pass(output_file)
                } else {
                    LaunchReport::RE
                }
            }
            LrunExceed::CpuTime | LrunExceed::RealTime => LaunchReport::TLE,
            LrunExceed::Memory => LaunchReport::MLE,
            LrunExceed::Output => LaunchReport::OLE,
        },
        report,
    )
}

#[cfg(test)]
mod tests {
    // FIXME: These tests fail sometime. But work well most of time.

    use super::*;
    use std::io::prelude::*;

    fn set_test_environments() {
        env::set_var("ANA_WORK_DIR", env::temp_dir());
        env::set_var("ANA_JUDGE_ID", "test_judge_id");
    }

    #[test]
    fn test_launcher() {
        set_test_environments();

        let mut input_file = env::temp_dir();
        input_file.push("test_launcher");
        input_file.set_extension("in");
        fs::File::create(&input_file)
            .expect("Failed to create test_launcher.in file")
            .write_all("echo hello world".as_bytes())
            .expect("Failed to write to test_launcher.in file");
        match launch(
            &path::Path::new("bash"),
            input_file.as_path(),
            &Limit::new(1.0, 64.0),
        )
        .0
        {
            LaunchReport::Pass(output_file) => {
                let mut output = String::new();
                fs::File::open(output_file)
                    .expect("Failed to open output file")
                    .read_to_string(&mut output)
                    .expect("Failed to read output from file");
                assert_eq!(output, "hello world\n")
            }
            _ => panic!("Failed when execute program"),
        }
        fs::remove_file(&input_file).expect("Failed to delete input file after testing");
    }

    #[test]
    fn test_memory_limit() {
        set_test_environments();

        let mut input_file = env::temp_dir();
        input_file.push("test_memory_limit");
        input_file.set_extension("in");
        fs::File::create(&input_file)
            .expect("Failed to create test_memory_limit.in file")
            .write_all("x='a'; while true; do x=$x$x; done".as_bytes())
            .expect("Failed to write to test_memory_limit.in file");
        match launch(
            &path::Path::new("bash"),
            input_file.as_path(),
            &Limit::new(1.0, 64.0),
        )
        .0
        {
            LaunchReport::MLE => {}
            _ => panic!("Failed when test memory limit"),
        }
        fs::remove_file(&input_file).expect("Failed to delete input file after testing");
    }

    #[test]
    fn test_time_limit() {
        set_test_environments();

        let mut input_file = env::temp_dir();
        input_file.push("test_time_limit");
        input_file.set_extension("in");
        fs::File::create(&input_file)
            .expect("Failed to create test_time_limit.in file")
            .write_all("while true; do echo -n; done".as_bytes())
            .expect("Failed to write to test_time_limit.in file");
        match launch(
            &path::Path::new("bash"),
            input_file.as_path(),
            &Limit::new(1.0, 64.0),
        )
        .0
        {
            LaunchReport::TLE => {}
            _ => panic!("Failed when test time limit"),
        }
        fs::remove_file(&input_file).expect("Failed to delete input file after testing");
    }

    #[test]
    fn test_runtime_error() {
        set_test_environments();

        let mut input_file = env::temp_dir();
        input_file.push("test_runtime_error");
        input_file.set_extension("in");
        fs::File::create(&input_file)
            .expect("Failed to create test_runtime_error.in file")
            .write_all("exit 1".as_bytes())
            .expect("Failed to write to test_runtime_error.in file");
        match launch(
            &path::Path::new("bash"),
            input_file.as_path(),
            &Limit::new(1.0, 64.0),
        )
        .0
        {
            LaunchReport::RE => {}
            _ => panic!("Failed when test runtime error"),
        }
        fs::remove_file(&input_file).expect("Failed to delete input file after testing");
    }
}
