use std::io;

#[cfg(windows)]
use winres::WindowsResource;

fn main() -> io::Result<()> {
    #[cfg(windows)]
    {
        WindowsResource::new()
            .set(
                "FileDescription",
                "Minecraft Resource Pack Util for encrypting and decrypting resource packs.",
            )
            .set("ProductName", "mcrputil")
            .set("OriginalFilename", "mcrputil.exe")
            .set("LegalCopyright", "Copyright (c) 2023, Valaphee.")
            .set("CompanyName", "Valaphee")
            .set("InternalName", "mcrputil.exe")
            .set_icon("mcrputil.ico")
            .compile()?;
    }
    Ok(())
}
