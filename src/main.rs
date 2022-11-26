use std::borrow::Borrow;
use std::fs::{copy, create_dir_all, File};
use std::io;
use std::io::{empty, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::str::from_utf8;

use aes::Aes256;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::KeyIvInit;
use cfb8::cipher::AsyncStreamCipher;
use clap::builder::Str;
use clap::Parser;
use glob::glob;
use mimalloc::MiMalloc;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use serde::{Deserialize, Serialize};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Parser)]
#[clap(about)]
enum McrpCommand {
    /// Encrypts the folder with a given or auto-generated key
    Encrypt {
        /// Input file or folder
        input: String,
        /// Output folder
        output: String,
        /// Key used for encryption
        #[clap(short, long)]
        key: Option<String>,
        /// Specifies files which should not be encrypted
        #[clap(short, long)]
        exclude: Vec<String>
    },
    /// Decrypts the folder with a given key
    Decrypt {
        /// Input file or folder
        input: String,
        /// Output folder
        output: String,
        /// Key used for decryption
        #[clap(short, long)]
        key: Option<String>,
    }
}

type Aes256Cfb8Enc = cfb8::Encryptor<Aes256>;
type Aes256Cfb8Dec = cfb8::Decryptor<Aes256>;

#[derive(Serialize, Deserialize, Debug)]
struct Manifest {
    header: ManifestHeader,
}

#[derive(Serialize, Deserialize, Debug)]
struct ManifestHeader {
    uuid: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Content {
    version: u32,
    content: Vec<ContentEntry>
}

#[derive(Serialize, Deserialize, Debug)]
struct ContentEntry {
    path: String,
    key: Option<String>
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match McrpCommand::parse() {
        McrpCommand::Encrypt { input, output, key, exclude } => {
            let always_exclude = vec!["manifest.json", "pack_icon.png", "bug_pack_icon.png"];

            let input_path = Path::new(&input);
            let output_path = Path::new(&output);

            // Read manifest to verify if its a valid pack and to find content id
            let id = serde_json::from_reader::<_, Manifest>(File::create(output_path.join("manifest.json"))?)?.header.uuid;

            // Generate or use given key, and store it to file
            let mut key_buffer = Vec::new();
            let key_bytes = match key {
                None => {
                    let mut rng = thread_rng();
                    key_buffer.write((0..32).map(|_| rng.sample(Alphanumeric) as char).collect::<String>().as_bytes())?;
                    key_buffer.borrow()
                },
                Some(ref key) => key.as_bytes()
            };
            File::create(format!("{}.key", output))?.write_all(key_bytes)?;

            // Create content list and copy or encrypt content
            let mut content_entries = Vec::new();
            for path in glob(&format!("{}/**/*.*", input))? {
                let input_entry_path = path?;
                let relative_path = input_entry_path.strip_prefix(input_path)?.to_str().unwrap().replace("\\", "/");
                let output_entry_path = output_path.join(&relative_path);

                content_entries.push(ContentEntry {
                    key: if always_exclude.contains(&relative_path.as_str()) || exclude.contains(&relative_path) {
                        if input_entry_path != output_entry_path {
                            if relative_path.ends_with(".json") {
                                serde_json::to_writer(File::create(output_entry_path)?, &serde_json::from_reader::<_, serde_json::Value>(File::open(input_entry_path)?)?)?
                            } else {
                                copy(input_entry_path, output_entry_path)?;
                            }

                            println!("Copied {}", relative_path);
                        }
                        None
                    } else {
                        let mut key_buffer = Vec::new();
                        let mut rng = thread_rng();
                        key_buffer.write((0..32).map(|_| rng.sample(Alphanumeric) as char).collect::<String>().as_bytes())?;
                        let key = from_utf8(&key_buffer)?.to_owned();

                        create_dir_all(output_entry_path.parent().unwrap())?;
                        let mut file = File::open(input_entry_path)?;
                        let mut buffer = Vec::new();
                        if relative_path.ends_with(".json") {
                            buffer.append(&mut serde_json::to_vec(&serde_json::from_reader::<_, serde_json::Value>(file)?)?);
                        } else {
                            file.read_to_end(&mut buffer)?;
                        }
                        Aes256Cfb8Enc::new_from_slices(&key_buffer, &key_buffer[0..16]).unwrap().encrypt(&mut buffer);
                        File::create(output_entry_path)?.write_all(&buffer)?;

                        println!("Encrypted {} with key {}", relative_path, key);

                        Some(key)
                    },
                    path: relative_path
                })
            }

            let mut file = File::create(output_path.join("contents.json"))?;
            file.write_all(&[0x00u8, 0x00u8, 0x00u8, 0x00u8, 0xFCu8, 0xB9u8, 0xCFu8, 0x9Bu8])?; // Magic
            file.seek(SeekFrom::Start(0x10))?;
            let id_bytes = id.as_bytes();
            file.write_all(&[id_bytes.len() as u8])?; // Content Id Length
            file.write_all(id_bytes)?; // Content Id

            let content = Content { version: 1, content: content_entries };
            let mut buffer = serde_json::to_vec(&content)?;
            Aes256Cfb8Enc::new_from_slices(&key_buffer, &key_buffer[0..16]).unwrap().encrypt(&mut buffer);
            file.seek(SeekFrom::Start(0x100))?;
            file.write_all(&buffer)?; // Encrypted content list

            println!("Encryption finished, key: {}", from_utf8(key_bytes)?);
        }
        McrpCommand::Decrypt { input, output, key } => {
            let input_path = Path::new(&input);
            let output_path = Path::new(&output);
            create_dir_all(output_path)?;

            let content = {
                let mut key_buffer = Vec::new();
                let key_bytes = match key {
                    None => {
                        File::open(format!("{}.key", input))?.read_to_end(&mut key_buffer)?;
                        key_buffer.borrow()
                    },
                    Some(ref key) => key.as_bytes()
                };

                let mut file = File::open(input_path.join("contents.json"))?;
                let mut buffer = Vec::new();
                file.seek(SeekFrom::Start(0x100))?;
                file.read_to_end(&mut buffer)?; // Encrypted content list
                Aes256Cfb8Dec::new_from_slices(&key_bytes, &key_bytes[0..16]).unwrap().decrypt(&mut buffer);
                serde_json::from_slice::<Content>(&buffer)?
            };

            // Copy or decrypt content
            for content_entry in &content.content {
                let input_entry_path = input_path.join(&content_entry.path);
                let output_entry_path = output_path.join(&content_entry.path);

                if input_entry_path.is_file() {
                    match &content_entry.key {
                        None => if input_entry_path != output_entry_path {
                            copy(input_entry_path, output_entry_path)?;

                            println!("Copied {}", &content_entry.path);
                        }
                        Some(key) => {
                            let key_bytes = key.as_bytes();

                            create_dir_all(output_entry_path.parent().unwrap())?;

                            let mut file = File::open(input_entry_path)?;
                            let mut buffer = Vec::new();
                            file.read_to_end(&mut buffer)?;
                            Aes256Cfb8Dec::new_from_slices(key_bytes, &key_bytes[0..16]).unwrap().decrypt(&mut buffer);
                            File::create(output_entry_path)?.write_all(&buffer)?;

                            println!("Decrypted {} with key {}", &content_entry.path, key);
                        }
                    }
                }
            }

            println!("Decryption finished");
        }
    }

    Ok(())
}
