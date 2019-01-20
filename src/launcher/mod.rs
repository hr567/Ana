use std::fs;
use std::io;
use std::path;
use std::thread;
use std::time;

use libc;
use unshare;

mod cgroup;
use cgroup::Cgroup;

pub enum LaunchResult {
    Pass,
    TLE,
    MLE,
    #[allow(dead_code)]
    OLE,
    RE,
}

pub struct Report {
    pub status: LaunchResult,
    pub time: u64,   // ns
    pub memory: u64, // bytes
}

pub fn launch(
    judge_id: &str,
    executable_file: &path::Path,
    input_file: &path::Path,
    output_file: &path::Path,
    time_limit: u64,   // ns
    memory_limit: u64, // bytes
) -> io::Result<Report> {
    let limit = cgroup::AnaCgroup::new(&judge_id);
    limit.set_time_limit(time_limit)?;
    limit.set_memory_limit(memory_limit)?;

    let status = {
        let limit: *const cgroup::AnaCgroup = &limit;

        let mut child = unshare::Command::new(&executable_file);
        child.before_unfreeze(move |child_pid| {
            if let Err(_) = unsafe { (*limit).add_task(child_pid) } {
                panic!("Failed to add task {} to cgroup", child_pid);
            }
            thread::spawn(move || {
                let timeout = time::Duration::from_nanos(time_limit + time_limit / 10);
                thread::sleep(timeout);
                unsafe {
                    libc::kill(child_pid as i32, libc::SIGKILL);
                }
            });
            Ok(())
        });
        child
            .env_clear()
            .unshare(&[
                unshare::Namespace::Cgroup,
                unshare::Namespace::Net,
                unshare::Namespace::Pid,
                unshare::Namespace::User,
                unshare::Namespace::Ipc,
            ])
            // .uid(65534)
            // .gid(65534)
            .arg0("main")
            .stdin(unshare::Stdio::from_file(fs::File::open(&input_file)?))
            .stdout(unshare::Stdio::from_file(fs::File::create(&output_file)?))
            .spawn()
            .unwrap()
            .wait()?
    };
    let time_usage = limit.get_time_usage()?;
    let memory_usage = limit.get_memory_usage()?;

    let status = if memory_usage >= memory_limit {
        LaunchResult::MLE
    } else if time_usage >= time_limit {
        LaunchResult::TLE
    } else if status.success() {
        LaunchResult::Pass
    } else {
        LaunchResult::RE
    };

    Ok(Report {
        status,
        time: time_usage,
        memory: memory_usage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io;

    use tempfile;

    struct JudgeDir {
        inner: tempfile::TempDir,
    }

    impl JudgeDir {
        fn new(name: &str, input_content: &str) -> io::Result<JudgeDir> {
            let res = JudgeDir {
                inner: tempfile::tempdir()?,
            };
            fs::write(&res.readme_file(), &name)?;
            fs::write(&res.input_file(), &input_content)?;
            Ok(res)
        }

        fn readme_file(&self) -> Box<path::Path> {
            self.inner.path().join("README").into_boxed_path()
        }

        fn input_file(&self) -> Box<path::Path> {
            self.inner.path().join("input").into_boxed_path()
        }

        fn output_file(&self) -> Box<path::Path> {
            self.inner.path().join("output").into_boxed_path()
        }
    }

    #[test]
    fn test_launcher() -> io::Result<()> {
        let judge_dir = JudgeDir::new("test_launcher", "echo hello world")?;
        match launch(
            "test_launcher",
            &path::Path::new("/bin/bash"),
            &judge_dir.input_file(),
            &judge_dir.output_file(),
            1_000_000_000,
            64 * 1024 * 1024,
        )?
        .status
        {
            LaunchResult::Pass => {
                assert_eq!(
                    fs::read(&judge_dir.output_file())?,
                    "hello world\n".as_bytes()
                );
            }
            _ => panic!("Failed to execute program"),
        }
        Ok(())
    }

    #[test]
    fn test_memory_limit() -> io::Result<()> {
        let judge_dir = JudgeDir::new("test_memory_limit", "x='a'; while true; do x=$x$x; done")?;
        match launch(
            "test_memory_limit",
            &path::Path::new("/bin/bash"),
            &judge_dir.input_file(),
            &judge_dir.output_file(),
            1_000_000_000,
            64 * 1024 * 1024,
        )?
        .status
        {
            LaunchResult::MLE => {}
            _ => panic!("Failed when test memory limit"),
        }
        Ok(())
    }

    #[test]
    fn test_time_limit() -> io::Result<()> {
        let judge_dir = JudgeDir::new("test_time_limit", "while true; do echo -n; done")?;
        match launch(
            "test_time_limit",
            &path::Path::new("/bin/bash"),
            &judge_dir.input_file(),
            &judge_dir.output_file(),
            1_000_000_000,
            64 * 1024 * 1024,
        )?
        .status
        {
            LaunchResult::TLE => {}
            _ => panic!("Failed when test time limit"),
        }
        Ok(())
    }

    #[test]
    fn test_runtime_error() -> io::Result<()> {
        let judge_dir = JudgeDir::new("test_runtime_error", "exit 1")?;
        match launch(
            "test_runtime_error",
            &path::Path::new("/bin/bash"),
            &judge_dir.input_file(),
            &judge_dir.output_file(),
            1_000_000_000,
            64 * 1024 * 1024,
        )?
        .status
        {
            LaunchResult::RE => {}
            _ => panic!("Failed when test runtime error"),
        }
        Ok(())
    }
}
