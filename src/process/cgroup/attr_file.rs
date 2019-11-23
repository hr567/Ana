//! AttrFile trait provide an interface to
//! read or write to a file with a given
//! attribute.
//!
//! This is widely used in cgroup filesystem.
use std::fmt::Debug;
use std::fs::{read_to_string, write};
use std::io;
use std::path::Path;
use std::str::FromStr;

/// A file which can be written and read.
pub trait AttrFile<'a, T, U> {
    /// Write a attribute to the file.
    fn write(&mut self, attr: &T) -> io::Result<()>;

    /// Write a attribute to the file.
    fn read(&self) -> io::Result<U>;
}

/// There is a default implementation of `AttrFile`
/// for the type which has `ToString` and `FromStr` trait.
impl<'a, T, U, P> AttrFile<'a, T, U> for P
where
    T: ToString,
    U: FromStr,
    U::Err: Debug,
    P: AsRef<Path>,
{
    fn write(&mut self, attr: &T) -> io::Result<()> {
        write(&self, &attr.to_string())?;
        Ok(())
    }

    fn read(&self) -> io::Result<U> {
        let attr = read_to_string(&self)?
            .trim()
            .parse()
            .expect("Failed to read the value from the given file");
        Ok(attr)
    }
}
