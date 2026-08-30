use std::{io, path::Path, process::Command};

pub fn reveal_in_folder(path: &Path) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        if !path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("output file does not exist: {}", path.display()),
            ));
        }
        // Explorer's unusual command-line grammar requires `/select,` to be
        // its own argument. Combining it with a path containing spaces makes
        // Windows quote the whole value, which Explorer rejects and responds
        // to by opening the default Documents folder.
        Command::new("explorer.exe")
            .arg("/select,")
            .arg(path)
            .spawn()?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("xdg-open")
            .arg(path.parent().unwrap_or(path))
            .spawn()?;
        Ok(())
    }
}
