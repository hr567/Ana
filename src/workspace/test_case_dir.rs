use std::path::Path;

pub trait TestCaseDir {
    fn path(&self) -> Box<Path>;

    fn input_file(&self) -> Box<Path> {
        self.path().join("input").into_boxed_path()
    }

    fn output_file(&self) -> Box<Path> {
        self.path().join("output").into_boxed_path()
    }

    fn answer_file(&self) -> Box<Path> {
        self.path().join("answer").into_boxed_path()
    }
}

impl TestCaseDir for Path {
    fn path(&self) -> Box<Path> {
        self.to_path_buf().into_boxed_path()
    }
}
