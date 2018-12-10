use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path;

const CGROUP_ROOT: &'static str = "/sys/fs/cgroup/";

pub struct Cgroup {
    cgroup_cpu_fs: Box<path::Path>,
    cgroup_memory_fs: Box<path::Path>,
}

// Use the default cgroup path created by systemd
impl Cgroup {
    pub fn new(time: u64 /*us*/, memory: u64 /*bytes*/) -> io::Result<Cgroup> {
        let cgroup_cpu_fs = path::Path::new(CGROUP_ROOT)
            .join("cpu")
            .join(env::var("ANA_JUDGE_ID").unwrap());
        let cgroup_memory_fs = path::Path::new(CGROUP_ROOT)
            .join("memory")
            .join(env::var("ANA_JUDGE_ID").unwrap());

        fs::create_dir(&cgroup_cpu_fs)?;
        let cpu_period = cgroup_cpu_fs.join("cpu.cfs_period_us");
        fs::write(&cpu_period, &format!("{}", time))?;
        let cpu_quota = cgroup_cpu_fs.join("cpu.cfs_quota_us");
        fs::write(&cpu_quota, &format!("{}", time))?;

        fs::create_dir(&cgroup_memory_fs)?;
        let memory_limit = cgroup_memory_fs.join("memory.limit_in_bytes");
        fs::write(&memory_limit, &format!("{}", memory))?;

        Ok(Cgroup {
            cgroup_cpu_fs: cgroup_cpu_fs.into_boxed_path(),
            cgroup_memory_fs: cgroup_memory_fs.into_boxed_path(),
        })
    }

    pub fn set_task(&mut self, task: u32) -> io::Result<()> {
        fs::write(&self.cgroup_cpu_fs.join("tasks"), &format!("{}", task))?;
        fs::write(&self.cgroup_memory_fs.join("tasks"), &format!("{}", task))?;
        Ok(())
    }

    pub fn report(&self) -> io::Result<(u64, u64)> {
        let mut cpu_usage = String::new();
        fs::File::open(&self.cgroup_cpu_fs.join("cpuacct.usage"))?
            .read_to_string(&mut cpu_usage)?;
        let cpu_usage: u64 = cpu_usage.trim().parse().unwrap();

        let mut memory_usage = String::new();
        fs::File::open(&self.cgroup_memory_fs.join("memory.usage_in_bytes"))?
            .read_to_string(&mut memory_usage)?;
        let memory_usage: u64 = memory_usage.trim().parse().unwrap();

        Ok((cpu_usage, memory_usage))
    }
}

impl Drop for Cgroup {
    fn drop(&mut self) {
        fs::remove_dir(&self.cgroup_cpu_fs).unwrap();
        fs::remove_dir(&self.cgroup_memory_fs).unwrap();
    }
}
