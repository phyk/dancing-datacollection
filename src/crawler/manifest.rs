use std::path::Path;

pub struct Manifest;

impl Manifest {
    pub fn is_already_downloaded(dir: &Path, filename: &str) -> bool {
        dir.join(filename).exists()
    }
}
