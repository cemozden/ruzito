extern crate byteorder;
extern crate inflate;

mod zip;

fn main() {
    let zip_file_result = zip::ZipFile::new(r"C:\Users\cem\Downloads\apache-tomcat-9.0.40.zip");

    if let Ok(zip_file) = zip_file_result {
        zip_file.extract_all();
    }
}