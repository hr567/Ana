use super::Workspace;

use std::io;

#[test]
fn test_normal_c() -> io::Result<()> {
    let _workspace = Workspace::from_path("examples/workspace/normal_c/")?;
    Ok(())
}

#[test]
fn test_spj_c() -> io::Result<()> {
    let _workspace = Workspace::from_path("examples/workspace/spj_c/")?;
    Ok(())
}

#[test]
fn test_custom_script() -> io::Result<()> {
    let _workspace = Workspace::from_path("examples/workspace/custom_script/")?;
    Ok(())
}
