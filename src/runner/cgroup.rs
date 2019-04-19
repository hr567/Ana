use std::fs;
use std::path;
use std::str;
use std::sync;
use std::time;

use log::*;
use nix;

static INIT_ANA_CGROUP: sync::Once = sync::Once::new();

const CPU_CGROUP_PATH: &str = "/sys/fs/cgroup/cpu/ana";
const MEMORY_CGROUP_PATH: &str = "/sys/fs/cgroup/memory/ana";

pub struct Cgroup {
    pub name: String,
    pub cpu_time_limit: time::Duration,
    pub memory_usage_limit: usize,
}

impl Cgroup {
    pub fn cpu_cgroup_path(&self) -> Box<path::Path> {
        path::Path::new(CPU_CGROUP_PATH)
            .join(&self.name)
            .into_boxed_path()
    }

    pub fn memory_cgroup_path(&self) -> Box<path::Path> {
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
            format!("{}", time::Duration::from_millis(500).as_micros()),
        )
        .unwrap();
        fs::write(
            &self.cpu_cgroup_path().join("cpu.cfs_quota_us"),
            format!("{}", time::Duration::from_millis(500).as_micros()),
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

impl Cgroup {
    pub fn new(cpu_time_limit: time::Duration, memory_usage_limit: usize) -> Cgroup {
        INIT_ANA_CGROUP.call_once(|| {
            fs::create_dir_all(CPU_CGROUP_PATH).expect("Failed to create cpu cgroup");
            fs::create_dir_all(MEMORY_CGROUP_PATH).expect("Failed to create memory cgroup");
            debug!(
                "Ana cgroup is created in {} and {}",
                CPU_CGROUP_PATH, MEMORY_CGROUP_PATH,
            );
        });
        let name = uuid::Uuid::new_v4();
        let ret = Cgroup {
            name: name.to_string(),
            cpu_time_limit,
            memory_usage_limit,
        };
        fs::create_dir(ret.cpu_cgroup_path()).unwrap();
        fs::create_dir(ret.memory_cgroup_path()).unwrap();
        debug!("Sub-cgroup {} is created", &name);
        ret.apply_cpu_time_limit();
        ret.apply_memory_usage_limit();
        ret
    }

    pub fn add_process(&self, pid: nix::unistd::Pid) {
        fs::write(
            &self.cpu_cgroup_path().join("cgroup.procs"),
            format!("{}", pid),
        )
        .expect("Failed to write to time cgroup processes");
        fs::write(
            &self.memory_cgroup_path().join("cgroup.procs"),
            format!("{}", pid),
        )
        .expect("Failed to write to memory cgroup processes");
    }

    pub fn get_cpu_time_usage(&self) -> time::Duration {
        let buf = fs::read(&self.cpu_cgroup_path().join("cpuacct.usage")).unwrap();
        let time_ns = str::from_utf8(&buf).unwrap().trim().parse().unwrap();
        time::Duration::from_nanos(time_ns)
    }

    pub fn get_memory_usage(&self) -> usize {
        let buf = fs::read(&self.memory_cgroup_path().join("memory.max_usage_in_bytes")).unwrap();
        str::from_utf8(&buf).unwrap().trim().parse().unwrap()
    }

    pub fn is_time_limit_exceeded(&self) -> bool {
        self.get_cpu_time_usage() > self.cpu_time_limit
    }

    fn memory_fail_count(&self) -> usize {
        let buf = fs::read(&self.memory_cgroup_path().join("memory.failcnt")).unwrap();
        str::from_utf8(&buf).unwrap().trim().parse().unwrap()
    }

    pub fn is_memory_limit_exceeded(&self) -> bool {
        self.memory_fail_count() > 0
    }
}

impl Drop for Cgroup {
    fn drop(&mut self) {
        fs::remove_dir(self.cpu_cgroup_path()).unwrap();
        fs::remove_dir(self.memory_cgroup_path()).unwrap();
        debug!("Sub cgroup {} is removed", &self.name);
    }
}
