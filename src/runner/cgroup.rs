use std::fs;
use std::path;
use std::str;
use std::sync;

use log::*;

static INIT_ANA_CGROUP: sync::Once = sync::Once::new();

const CPU_CGROUP_PATH: &str = "/sys/fs/cgroup/cpu/ana";
const MEMORY_CGROUP_PATH: &str = "/sys/fs/cgroup/memory/ana";

pub struct Cgroup {
    pub name: String,
    pub cpu_time_limit: u64,
    pub memory_usage_limit: u64,
}

impl Cgroup {
    fn cpu_cgroup_path(&self) -> Box<path::Path> {
        path::Path::new(CPU_CGROUP_PATH)
            .join(&self.name)
            .into_boxed_path()
    }

    fn memory_cgroup_path(&self) -> Box<path::Path> {
        path::Path::new(MEMORY_CGROUP_PATH)
            .join(&self.name)
            .into_boxed_path()
    }

    // This function do not limit total cpu time usage
    // but only limit the process to use at most
    // 100% cpu time as the time in real world.
    fn apply_cpu_time_limit(&self) {
        fs::write(
            &self.cpu_cgroup_path().join("cpu.cfs_period_us"),
            format!("{}", self.cpu_time_limit / 1000),
        )
        .unwrap();
        fs::write(
            &self.cpu_cgroup_path().join("cpu.cfs_quota_us"),
            format!("{}", self.cpu_time_limit / 1000),
        )
        .unwrap();
    }

    // Limit the memory usage and stop the process swapping to disk
    fn apply_memory_usage_limit(&self) {
        fs::write(
            &self.memory_cgroup_path().join("memory.limit_in_bytes"),
            format!("{}", self.memory_usage_limit),
        )
        .unwrap();
        fs::write(
            &self.memory_cgroup_path().join("memory.swappiness"),
            format!("{}", 0),
        )
        .unwrap();
    }
}

impl Drop for Cgroup {
    fn drop(&mut self) {
        fs::remove_dir(self.cpu_cgroup_path()).expect("Failed to remove cpu cgroup");
        fs::remove_dir(self.memory_cgroup_path()).expect("Failed to remove memory cgroup");
        debug!("Sub cgroup {} is removed", &self.name);
    }
}

impl Cgroup {
    pub fn new(name: &str, cpu_time_limit: u64, memory_usage_limit: u64) -> Cgroup {
        INIT_ANA_CGROUP.call_once(|| {
            fs::create_dir_all(CPU_CGROUP_PATH).expect("Failed to create cgroup");
            fs::create_dir_all(MEMORY_CGROUP_PATH).expect("Failed to create cgroup");
            debug!("Ana cgroup is created");
        });
        let res = Cgroup {
            name: name.to_string(),
            cpu_time_limit,
            memory_usage_limit,
        };
        debug!("Creating sub-cgroup {}", &name);
        fs::create_dir(res.cpu_cgroup_path()).expect("Failed to create cpu cgroup");
        fs::create_dir(res.memory_cgroup_path()).expect("Failed to create memory cgroup");
        debug!("Sub-cgroup {} is created", &name);
        res.apply_cpu_time_limit();
        res.apply_memory_usage_limit();
        res
    }

    pub fn add_process_method(&self) -> impl Fn(u32) {
        let cpu_procs = self
            .cpu_cgroup_path()
            .join("cgroup.procs")
            .into_boxed_path();
        let memory_proc = self
            .memory_cgroup_path()
            .join("cgroup.procs")
            .into_boxed_path();
        move |pid: u32| {
            fs::write(&cpu_procs, format!("{}", pid)).unwrap();
            fs::write(&memory_proc, format!("{}", pid)).unwrap();
        }
    }

    pub fn get_cpu_time_usage(&self) -> u64 {
        let buf = fs::read(&self.cpu_cgroup_path().join("cpuacct.usage"))
            .expect("Failed to read cpu usage");
        str::from_utf8(&buf).unwrap().trim().parse().unwrap()
    }

    pub fn get_memory_usage(&self) -> u64 {
        let buf = fs::read(&self.memory_cgroup_path().join("memory.max_usage_in_bytes"))
            .expect("Failed to read memory usage");
        str::from_utf8(&buf).unwrap().trim().parse().unwrap()
    }

    pub fn is_time_limit_exceeded(&self) -> bool {
        self.get_cpu_time_usage() >= self.cpu_time_limit
    }

    fn memory_fail_count(&self) -> usize {
        let buf = fs::read(&self.memory_cgroup_path().join("memory.failcnt"))
            .expect("Failed to read memory fail count");
        str::from_utf8(&buf).unwrap().trim().parse().unwrap()
    }

    pub fn is_memory_limit_exceeded(&self) -> bool {
        self.memory_fail_count() != 0
    }
}
