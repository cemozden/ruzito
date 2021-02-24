use std::{ffi::OsString, path::Path};

pub fn get_path<P>(path: P) -> Option<OsString> where P: AsRef<Path> {
    let file_path = path.as_ref();
    if !file_path.exists() {
        let current_dir = match std::env::current_dir() {
            Ok(path) => path,
            Err(err) => {
                eprintln!("An error occured! Error: {}", err);
                return None;
            }
        };
        let file_path = Path::new(current_dir.as_path()).with_file_name(file_path);
        if file_path.exists() {
            return Some(OsString::from(file_path.as_os_str()));
        }
        else {
            eprintln!(r"Unable to find the given path {:?}", file_path);
            return None;
        }
    }
    else {
        return Some(OsString::from(file_path.as_os_str()));
    }
}