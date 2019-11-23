use std::fs::{create_dir, read_to_string, write};
use std::io;
use std::marker::PhantomData;
use std::path::Path;
use std::time::Duration;

use super::*;

pub struct CpuController<'a, T: 'a + AsRef<Path>> {
    inner: T,
    _mark: PhantomData<&'a ()>,
}

impl<'a, T: 'a + AsRef<Path>> CpuController<'a, T> {
    pub fn period(&'a self) -> Box<dyn AttrFile<'a, Duration, Duration> + 'a> {
        Box::new(CpuTimeFile {
            inner: self.inner.as_ref().join("cpu.cfs_period_us"),
            _mark: PhantomData,
        })
    }

    pub fn quota(&'a self) -> Box<dyn AttrFile<'a, Duration, Duration> + 'a> {
        Box::new(CpuTimeFile {
            inner: self.inner.as_ref().join("cpu.cfs_quota_us"),
            _mark: PhantomData,
        })
    }
}

impl<'a> Controller<'a> for CpuController<'a, PathBuf> {
    const NAME: &'static str = "cpu";

    fn from_ctx(context: &Context) -> CpuController<PathBuf> {
        CpuController {
            inner: Context::root().join(Self::NAME).join(&context.name),
            _mark: PhantomData,
        }
    }

    fn initialize(&self) -> io::Result<()> {
        match create_dir(&self.inner) {
            Ok(_) => {}
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
        Ok(())
    }
}

impl<'a, T: 'a + AsRef<Path>> AsRef<Path> for CpuController<'a, T> {
    fn as_ref(&self) -> &Path {
        self.inner.as_ref()
    }
}

struct CpuTimeFile<'a, T: 'a + AsRef<Path>> {
    inner: T,
    _mark: PhantomData<&'a T>,
}

impl<'a, T: 'a + AsRef<Path>> AttrFile<'a, Duration, Duration> for CpuTimeFile<'a, T> {
    fn read(&self) -> io::Result<Duration> {
        let attr: u64 = read_to_string(&self.inner)?.trim().parse().unwrap();
        Ok(Duration::from_micros(attr))
    }

    fn write(&mut self, attr: &Duration) -> io::Result<()> {
        write(&self.inner, attr.as_micros().to_string())?;
        Ok(())
    }
}
