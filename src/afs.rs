use std::collections::HashSet;
use std::fs::read_dir;
use std::io;
use std::path::Path;

pub trait AbstractFilesystem {
    fn file_names_in(&self, rel_path: &str) -> io::Result<HashSet<Box<str>>>;
}

pub struct Filesystem<'a> {
    path: &'a Path,
}

impl<'a> Filesystem<'a> {
    pub fn new(path: &'a Path) -> Self {
        Self { path }
    }
}

impl<'a> AbstractFilesystem for Filesystem<'a> {
    fn file_names_in(&self, rel_path: &str) -> io::Result<HashSet<Box<str>>> {
        Ok(read_dir(self.path.join(rel_path))?.filter_map(|entry| {
            entry.ok().map(|e| {
                e.file_name().to_string_lossy().to_string().into_boxed_str()
            })
        })
        .collect())
    }
}
