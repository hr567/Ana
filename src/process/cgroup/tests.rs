use super::*;

use std::iter::FromIterator;
use std::time::Duration;

#[tokio::test]
async fn test_cgroup_path() -> io::Result<()> {
    let ctx = Builder::new().build().await?;
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

#[tokio::test]
async fn test_cpu_controller() -> io::Result<()> {
    let ctx = Builder::new().build().await?;

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

#[tokio::test]
async fn test_cpuacct_controller() -> io::Result<()> {
    let ctx = Builder::new().build().await?;

    let cpuacct_controller = ctx.cpuacct_controller().unwrap();
    let cpu_usage = cpuacct_controller.usage()?;
    assert_eq!(cpu_usage, Duration::from_secs(0));

    Ok(())
}

#[tokio::test]
async fn test_cpuset_controller() -> io::Result<()> {
    let request = 2;
    let ctx = Builder::new().cpuset_controller(true, request).build().await?;

    let cpuset_controller = ctx.cpuset_controller().unwrap();
    let cpuset_allocated = cpuset_controller.allocated()?;
    let mut num = 0;
    cpuset_allocated.iter().for_each(|(start, end)| {
        num += end - start + 1;
    });
    assert_eq!(request, num);
    Ok(())
}

#[tokio::test]
async fn test_memory_controller() -> io::Result<()> {
    let ctx = Builder::new().build().await?;

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
