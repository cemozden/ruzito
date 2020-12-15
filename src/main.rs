extern crate byteorder;

mod zip;

fn main() {
    let _ = zip::ZipFile::new(r"C:\eula.zip");
}