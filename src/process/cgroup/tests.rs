use super::*;

use std::iter::FromIterator;
use std::time::Duration;

#[test]
fn test_cgroup_path() -> io::Result<()> {
    let ctx = Builder::new().build()?;
    let cpu_controller = ctx.cpu_controller().unwrap();
    let cpu_path = cpu_controller.as_ref();
    assert!(cpu_path.exists());
    assert_eq!(
        cpu_path,
        PathBuf::from_iter(&["/sys/fs/cgroup/cpu/", ctx.name.as_str()])
    );

    let cpuacct_controller = ctx.cpuacct_controller().unwrap();
    let cpuact_path = cpuacct_controller.as_ref();
    assert!(cpuact_path.exists());
    assert_eq!(
        cpuact_path,
        PathBuf::from_iter(&["/sys/fs/cgroup/cpuacct/", ctx.name.as_str()])
    );

    let memory_controller = ctx.memory_controller().unwrap();
    let memory_path = memory_controller.as_ref();
    assert!(memory_path.exists());
    assert_eq!(
        memory_path,
        PathBuf::from_iter(&["/sys/fs/cgroup/memory/", ctx.name.as_str()])
    );

    Ok(())
}

#[test]
fn test_cpu_controller() -> io::Result<()> {
    let ctx = Builder::new().build()?;

    let cpu_controller = ctx.cpu_controller().unwrap();

    cpu_controller.period().write(&Duration::from_millis(200))?;
    cpu_controller.quota().write(&Duration::from_millis(80))?;
    assert_eq!(cpu_controller.period().read()?, Duration::from_millis(200));
    assert_eq!(cpu_controller.quota().read()?, Duration::from_millis(80));

    cpu_controller.period().write(&Duration::from_millis(150))?;
    cpu_controller.quota().write(&Duration::from_millis(50))?;
    assert_eq!(cpu_controller.period().read()?, Duration::from_millis(150));
    assert_eq!(cpu_controller.quota().read()?, Duration::from_millis(50));

    Ok(())
}

#[test]
fn test_cpuacct_controller() -> io::Result<()> {
    let ctx = Builder::new().build()?;

    let cpuacct_controller = ctx.cpuacct_controller().unwrap();
    let cpu_usage = cpuacct_controller.usage()?;
    assert_eq!(cpu_usage, Duration::from_secs(0));

    Ok(())
}

#[test]
fn test_memory_controller() -> io::Result<()> {
    let ctx = Builder::new().build()?;

    let memory_controller = ctx.memory_controller().unwrap();

    memory_controller.limit_in_bytes().write(&(128 * 1024))?;
    assert_eq!(memory_controller.limit_in_bytes().read()?, 128 * 1024);

    memory_controller.limit_in_bytes().write(&(128 * 1024))?;
    assert_eq!(memory_controller.limit_in_bytes().read()?, 128 * 1024);

    assert_eq!(memory_controller.usage_in_bytes()?, 0);
    assert_eq!(memory_controller.max_usage_in_bytes()?, 0);
    assert_eq!(memory_controller.failcnt()?, 0);

    Ok(())
}
