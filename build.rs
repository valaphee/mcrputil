use std::io;
#[cfg(windows)] use winres::WindowsResource;

fn main() -> io::Result<()> {
    #[cfg(windows)] {
        WindowsResource::new()
            .set("FileDescription", "Encrypt, minify or decrypt, pretty-print resource packs for Minecraft.")
            .set("ProductName", "mcrputil")
            .set("OriginalFilename", "mcrputil.exe")
            .set("LegalCopyright", "Copyright (c) 2022, Valaphee.")
            .set("CompanyName", "Valaphee")
            .set("InternalName", "mcrputil.exe")
            .set_icon("mcrputil.ico")
            /*.set_resource_file("mcrputil.rc")*/
            .compile()?;
    }
    Ok(())
}
