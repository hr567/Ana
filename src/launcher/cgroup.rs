use std::fs;
use std::io;
use std::path;

pub trait Cgroup {
    fn add_task(&self, task_id: u32) -> io::Result<()>;
    fn set_time_limit(&self, cpu_limit: u64) -> io::Result<()>;
    fn set_memory_limit(&self, memory_limit: u64) -> io::Result<()>;

    fn get_time_usage(&self) -> io::Result<u64>;
    fn get_memory_usage(&self) -> io::Result<u64>;
}

pub struct AnaCgroup {
    judge_id: String,
}

impl AnaCgroup {
    fn inited() -> bool {
        AnaCgroup::cgroup_cpu_path().exists() && AnaCgroup::cgroup_memory_path().exists()
    }

    fn init() -> io::Result<()> {
        fs::create_dir_all(AnaCgroup::cgroup_cpu_path())?;
        fs::create_dir_all(AnaCgroup::cgroup_memory_path())?;
        Ok(())
    }

    fn deinit() -> io::Result<()> {
        fs::remove_dir(AnaCgroup::cgroup_cpu_path())?;
        fs::remove_dir(AnaCgroup::cgroup_memory_path())?;
        Ok(())
    }

    pub fn new(judge_id: &str) -> AnaCgroup {
        if !AnaCgroup::inited() {
            AnaCgroup::init().expect("Failed to init cgroup");
        }

        let new_cgroup = AnaCgroup {
            judge_id: judge_id.to_string(),
        };
        fs::create_dir(&new_cgroup.cpu_path()).unwrap();
        fs::create_dir(&new_cgroup.memory_path()).unwrap();
        new_cgroup
    }

    fn cgroup_cpu_path() -> &'static path::Path {
        &path::Path::new("/sys/fs/cgroup/cpu/ana")
    }

    fn cgroup_memory_path() -> &'static path::Path {
        &path::Path::new("/sys/fs/cgroup/memory/ana")
    }

    fn cpu_path(&self) -> Box<path::Path> {
        AnaCgroup::cgroup_cpu_path()
            .join(&self.judge_id)
            .into_boxed_path()
    }

    fn memory_path(&self) -> Box<path::Path> {
        AnaCgroup::cgroup_memory_path()
            .join(&self.judge_id)
            .into_boxed_path()
    }
}

impl Drop for AnaCgroup {
    fn drop(&mut self) {
        fs::remove_dir(&self.cpu_path()).unwrap();
        fs::remove_dir(&self.memory_path()).unwrap();
        AnaCgroup::deinit().unwrap_or(());
    }
}

impl Cgroup for AnaCgroup {
    fn add_task(&self, task_id: u32) -> io::Result<()> {
        fs::write(&self.cpu_path().join("tasks"), format!("{}", task_id))?;
        fs::write(&self.memory_path().join("tasks"), format!("{}", task_id))?;
        Ok(())
    }

    fn set_time_limit(&self, cpu_limit: u64) -> io::Result<()> {
        fs::write(
            &self.cpu_path().join("cpu.cfs_period_us"),
            format!("{}", cpu_limit / 1000),
        )?;
        fs::write(
            &self.cpu_path().join("cpu.cfs_quota_us"),
            format!("{}", cpu_limit / 1000),
        )?;
        Ok(())
    }

    fn set_memory_limit(&self, memory_limit: u64) -> io::Result<()> {
        fs::write(
            &self.memory_path().join("memory.limit_in_bytes"),
            format!("{}", memory_limit),
        )?;
        fs::write(
            &self.memory_path().join("memory.swappiness"),
            format!("{}", 0),
        )?;
        Ok(())
    }

    fn get_time_usage(&self) -> io::Result<u64> {
        Ok({
            String::from_utf8({ fs::read(&self.cpu_path().join("cpuacct.usage"))? })
                .unwrap()
                .trim()
                .parse()
                .unwrap()
        })
    }

    fn get_memory_usage(&self) -> io::Result<u64> {
        Ok({
            String::from_utf8({ fs::read(&self.memory_path().join("memory.max_usage_in_bytes"))? })
                .unwrap()
                .trim()
                .parse()
                .unwrap()
        })
    }
}
