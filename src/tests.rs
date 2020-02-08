use std::fs;
use std::io;
use std::path::Path;

use tempfile;
use tokio::stream::StreamExt;
use tokio::sync::mpsc;

use crate::judge::{judge, ResultType};
use crate::workspace::Workspace;

#[tokio::test]
async fn test_normal_c() -> io::Result<()> {
    const EXAMPLE_WORKSPACE: &str = "examples/workspace/normal_c";
    let workspace = tempfile::tempdir()?;
    copy_dir(EXAMPLE_WORKSPACE, workspace.path())?;
    let workspace = Workspace::from_path(workspace.path())?;
    test_workspace(workspace).await?;
    Ok(())
}

#[tokio::test]
async fn test_spj_c() -> io::Result<()> {
    const EXAMPLE_WORKSPACE: &str = "examples/workspace/spj_c";
    let workspace = tempfile::tempdir()?;
    copy_dir(EXAMPLE_WORKSPACE, workspace.path())?;
    let workspace = Workspace::from_path(workspace.path())?;
    test_workspace(workspace).await?;
    Ok(())
}

#[tokio::test]
async fn test_custom_script() -> io::Result<()> {
    const EXAMPLE_WORKSPACE: &str = "examples/workspace/custom_script";
    let workspace = tempfile::tempdir()?;
    copy_dir(EXAMPLE_WORKSPACE, workspace.path())?;
    let workspace = Workspace::from_path(workspace.path())?;
    test_workspace(workspace).await?;
    Ok(())
}

fn copy_dir<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    let (src, dst) = (src.as_ref(), dst.as_ref());
    let mut rd = fs::read_dir(src)?;
    while let Some(de) = rd.next() {
        let dir_entry = de?;
        let dst = dst.join(dir_entry.path().strip_prefix(src).unwrap());
        let file_type = dir_entry.file_type()?;
        if file_type.is_dir() {
            fs::create_dir_all(&dst)?;
            copy_dir(dir_entry.path(), &dst)?;
        } else if file_type.is_file() || file_type.is_symlink() {
            fs::copy(dir_entry.path(), &dst)?;
        } else {
            eprintln!("File type of {:?} is not recognized.", dir_entry.path());
        }
    }
    Ok(())
}

async fn test_workspace(workspace: Workspace) -> io::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();
    judge(workspace, tx).await?;
    assert!(
        rx.all(|report| dbg!(report).result == ResultType::Accepted)
            .await
    );
    Ok(())
}
