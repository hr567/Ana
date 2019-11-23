mod builder;
mod comparer;
// mod judge;
mod process;
#[cfg(feature = "rpc")]
mod rpc;
mod runner;
mod workspace;

// trait ResourceController {
//     fn set_limit(&mut self, limit: Resource) -> io::Result<()>;
//     fn get_usage(&self) -> io::Result<Resource>;
// }

// impl ResourceController for executor::cgroup::Context {
//     fn set_limit(&mut self, limit: Resource) -> io::Result<()> {
//         let cpu_controller = self.cpu_controller().unwrap();
//         let memory_controller = self.memory_controller().unwrap();

//         let real_time = limit.real_time;
//         let cpu_time = limit.cpu_time;

//         let period = Duration::from_secs(1);
//         let quota = {
//             let real_time = real_time.as_micros() as u32;
//             let cpu_time = cpu_time.as_micros() as u32;
//             period * cpu_time / real_time
//         };

//         cpu_controller.period().write(&period)?;
//         cpu_controller.quota().write(&quota)?;
//         memory_controller.limit_in_bytes().write(&limit.memory)?;

//         Ok(())
//     }

//     fn get_usage(&self) -> io::Result<Resource> {
//         let res = Resource {
//             cpu_time: self
//                 .cpuacct_controller()
//                 .expect("Failed to get cpuacct controller")
//                 .usage()?,
//             real_time: Duration::from_secs(0),
//             memory: self
//                 .memory_controller()
//                 .expect("Failed to get memory controller")
//                 .max_usage_in_bytes()?,
//         };
//         Ok(res)
//     }
// }
