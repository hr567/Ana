use std::fs;
use std::io;
use std::path;

pub struct AnaCgroup {
    judge_id: String,
}

pub trait Cgroup {
    fn cgroup_root_path() -> &'static path::Path;
    fn cgroup_cpu_path() -> &'static path::Path;
    fn cgroup_memory_path() -> &'static path::Path;

    fn cpu_path(&self) -> Box<path::Path>;
    fn memory_path(&self) -> Box<path::Path>;

    unsafe fn add_task(&self, task_id: u32) -> io::Result<()>;
    unsafe fn set_time_limit(&self, cpu_limit: u64) -> io::Result<()>;
    unsafe fn set_memory_limit(&self, memory_limit: u64) -> io::Result<()>;

    fn get_time_usage(&self) -> io::Result<u64>;
    fn get_memory_usage(&self) -> io::Result<u64>;
}

impl Cgroup for AnaCgroup {
    fn cgroup_root_path() -> &'static path::Path {
        &path::Path::new("/sys/fs/cgroup/")
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

    unsafe fn add_task(&self, task_id: u32) -> io::Result<()> {
        fs::write(&self.cpu_path().join("tasks"), format!("{}", task_id))?;
        fs::write(&self.memory_path().join("tasks"), format!("{}", task_id))?;
        Ok(())
    }
    unsafe fn set_time_limit(&self, cpu_limit: u64) -> io::Result<()> {
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
    unsafe fn set_memory_limit(&self, memory_limit: u64) -> io::Result<()> {
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

impl AnaCgroup {
    pub fn init() -> io::Result<()> {
        fs::create_dir(AnaCgroup::cgroup_cpu_path())?;
        fs::create_dir(AnaCgroup::cgroup_memory_path())?;
        Ok(())
    }

    pub fn inited() -> bool {
        AnaCgroup::cgroup_cpu_path().exists() && AnaCgroup::cgroup_memory_path().exists()
    }

    pub fn new(judge_id: String) -> io::Result<AnaCgroup> {
        let new_cgroup = AnaCgroup { judge_id };
        fs::create_dir(&new_cgroup.cpu_path())?;
        fs::create_dir(&new_cgroup.memory_path())?;
        Ok(new_cgroup)
    }
}

impl Drop for AnaCgroup {
    fn drop(&mut self) {
        fs::remove_dir(&self.cpu_path()).unwrap();
        fs::remove_dir(&self.memory_path()).unwrap();
    }
}
