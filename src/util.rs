use std::io::{Error, Write};

pub fn read_pass() -> Result<String, Error> {
    print!("Enter password: ");
    if let Err(err) = std::io::stdout().flush() {
        return Err(err)
    }
    let pass = match rpassword::read_password() {
        Ok(pass) => pass,
        Err(err) => return Err(err)
    };

    Ok(pass)
}