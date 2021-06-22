# ruzito
![TravisCI](https://api.travis-ci.com/cemozden/ruzito.svg?branch=master)

*ruzito* is a CLI tool to manage archive files which is written in Rust. Currently, ruzito supports only **ZIP** format but ***RAR*** and ***7z*** formats are on the way. 

### ZIP features
* ruzito is able to zip, extract as well as list the files with high speed compression/decompression features.
* ZipCrypto support. (Strong encryption and WinZip AES-256 implementations are on the way.)

***

## Usage
To zip a file/folder with ruzito run the following command
```bash
# Zipping Documents folder
# The below command will generate the ZIP file with the name "Documents.zip" if destination path is not specified.
ruzito zip -z Documents\

# Zipping Documents folder with verbose mode
ruzito zip -z Documents\ -v

# Choosing the output name
ruzito zip -z Documents\ -d C:\zip_files\my_documents.zip

# Zipping Documents folder with encryption enabled.
ruzito zip -z Documents\ -e # E flag will prompt user to provide a password.

# Zipping Documents folder with encryption enabled and password provided.
ruzito zip -z Documents\ -p mypassword
```

To extract a ZIP file, you can run the following commands

```bash
ruzito zip -x my_zip_file.zip

# Extract files with verbose mode
ruzito zip -x my_zip_file.zip -v

# Choose the destination path of the extracted file(s)
ruzito zip -x my_zip_file.zip -d C:\my_path
```

To list content the ZIP file, run the following command
```bash
ruzito zip -l my_zip_file.zip
```
## License
2021, MIT License, see [LICENSE](https://github.com/cemozden/ruzito/blob/master/LICENSE).