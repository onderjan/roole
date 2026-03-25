use std::path::{Path, PathBuf};

use walkdir::WalkDir;

pub fn iterate_smt2_path(path: &Path) -> impl Iterator<Item = walkdir::DirEntry> {
    WalkDir::new(path).into_iter().filter_map(|entry| {
        let entry = match entry {
            Ok(ok) => ok,
            Err(error) => panic!("Error walking directory: {:?}", error),
        };

        if entry.path().extension()? == "smt2" {
            Some(entry)
        } else {
            None
        }
    })
}

pub fn iterate_smt2_paths(paths: &[PathBuf]) -> Box<dyn Iterator<Item = walkdir::DirEntry> + '_> {
    let mut iterator: Box<dyn Iterator<Item = walkdir::DirEntry>> = Box::new(std::iter::empty());
    for path in paths {
        iterator = Box::new(iterator.chain(iterate_smt2_path(path)));
    }
    iterator
}

pub fn compute_output_path(input_root: Option<&Path>, input_path: &Path) -> PathBuf {
    let input_root = input_root
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().expect("Current directory should be valid"));

    let Ok(input_root) = std::path::absolute(&input_root) else {
        panic!(
            "Input root should be expressible as absolute: {:?}",
            input_root
        );
    };

    let Ok(input_path) = std::path::absolute(input_path) else {
        panic!("Path should be expressible as absolute: {:?}", input_path);
    };

    // conver the path relative to input root
    let Some(mut output_path) = pathdiff::diff_paths(&input_path, &input_root) else {
        panic!(
            "Path {:?} should be expressible relative to input root {:?}",
            input_path, input_root
        );
    };

    if output_path.is_absolute() {
        panic!("Path should be relative-expressible: {:?}", output_path);
    }

    if output_path.iter().any(|a| a == "..") {
        panic!(
            "Path expressed relative to input root should not contain '..': {:?}",
            output_path
        );
    }

    output_path.set_extension("out");

    output_path
}
