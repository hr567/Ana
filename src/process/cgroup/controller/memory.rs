use std::fs::{create_dir, read_to_string};
use std::io;
use std::marker::PhantomData;
use std::path::Path;

use super::*;

pub struct MemoryController<'a, T: 'a + AsRef<Path>> {
    inner: T,
    _mark: PhantomData<&'a ()>,
}

impl<'a, T: AsRef<Path>> MemoryController<'a, T> {
    pub fn usage_in_bytes(&self) -> io::Result<usize> {
        let file = self.inner.as_ref().join("memory.usage_in_bytes");
        let usage = read_to_string(&file)?.trim().parse().unwrap();
        Ok(usage)
    }

    pub fn max_usage_in_bytes(&self) -> io::Result<usize> {
        let file = self.inner.as_ref().join("memory.max_usage_in_bytes");
        let usage = read_to_string(&file)?.trim().parse().unwrap();
        Ok(usage)
    }

    pub fn limit_in_bytes(&'a self) -> Box<dyn AttrFile<'a, usize, usize> + 'a> {
        Box::new(self.inner.as_ref().join("memory.limit_in_bytes"))
    }

    pub fn failcnt(&self) -> io::Result<usize> {
        let file = self.inner.as_ref().join("memory.failcnt");
        let count = read_to_string(&file)?.trim().parse().unwrap();
        Ok(count)
    }

    pub fn swappiness(&'a self) -> Box<dyn AttrFile<'a, usize, usize> + 'a> {
        Box::new(self.inner.as_ref().join("memory.swappiness"))
    }
}

impl<'a> Controller<'a> for MemoryController<'a, PathBuf> {
    const NAME: &'static str = "memory";

    fn from_ctx(context: &Context) -> MemoryController<PathBuf> {
        MemoryController {
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

impl<'a, T: 'a + AsRef<Path>> AsRef<Path> for MemoryController<'a, T> {
    fn as_ref(&self) -> &Path {
        self.inner.as_ref()
    }
}
