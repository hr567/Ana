use std::fs;
use std::path::Path;

use crate::mtp;

pub trait SourceDir {
    fn path(&self) -> Box<Path>;

    fn language_file(&self) -> Box<Path> {
        self.path().join("lang").into_boxed_path()
    }

    fn source_file(&self) -> Box<Path> {
        self.path().join("source").into_boxed_path()
    }

    fn executable_file(&self) -> Box<Path> {
        self.path().join("main").into_boxed_path()
    }

    fn init_source_dir(&self, source: mtp::Source) {
        fs::write(self.language_file(), source.language).unwrap();
        fs::write(self.source_file(), source.code).unwrap();
    }

    fn get_language(&self) -> String {
        let language = fs::read(self.language_file()).unwrap();
        String::from_utf8(language).unwrap()
    }
}

impl SourceDir for Path {
    fn path(&self) -> Box<Path> {
        self.to_path_buf().into_boxed_path()
    }
}
