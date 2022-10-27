use std::io;
#[cfg(windows)] use winres::WindowsResource;

fn main() -> io::Result<()> {
    #[cfg(windows)] {
        WindowsResource::new()
            .set("FileDescription", "Minecraft Resource Pack Util for encrypting, decrypting, signing and verifying resource packs.")
            .set("ProductName", "Minecraft Resource Pack Util")
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
