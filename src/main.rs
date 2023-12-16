use std::{
    borrow::Cow,
    fs::{copy, create_dir_all, File},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    str::from_utf8,
};

use aes::{cipher::KeyIvInit, Aes256};
use cfb8::cipher::AsyncStreamCipher;
use clap::Parser;
use glob::glob;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use wildmatch::WildMatch;

#[derive(Parser)]
#[clap(about)]
enum McrpCommand {
    /// Encrypts the directory with a given or auto-generated key
    Encrypt {
        /// Input directory
        input: String,
        /// Output directory
        output: String,
        /// Key used for encryption
        #[clap(short, long)]
        key: Option<String>,
        /// Specifies files which should not be encrypted
        #[clap(short, long)]
        exclude: Vec<String>,
    },
    /// Decrypts the directory with a given key
    Decrypt {
        /// Input directory
        input: String,
        /// Output directory
        output: String,
        /// Key used for decryption
        #[clap(short, long)]
        key: String,
    },
}

fn main() {
    match McrpCommand::parse() {
        McrpCommand::Encrypt {
            input,
            output,
            key,
            exclude,
        } => {
            let input_path = Path::new(&input);
            let output_path = Path::new(&output);
            let key_bytes = match &key {
                None => {
                    let mut rng = thread_rng();
                    let mut key_buffer = Vec::new();
                    key_buffer
                        .write(
                            (0..32)
                                .map(|_| rng.sample(Alphanumeric) as char)
                                .collect::<String>()
                                .as_bytes(),
                        )
                        .unwrap();
                    Cow::Owned(key_buffer)
                }
                Some(key) => Cow::Borrowed(key.as_bytes()),
            };
            encrypt(input_path, output_path, &key_bytes, exclude)
        }
        McrpCommand::Decrypt { input, output, key } => {
            let input_path = Path::new(&input);
            let output_path = Path::new(&output);
            let key_bytes = key.as_bytes();
            if key_bytes.len() != 32 {
                panic!(
                    "Expected a key with a length of 32, got {}",
                    key_bytes.len()
                );
            }
            decrypt(input_path, output_path, &key_bytes);
        }
    }
}

fn encrypt(input: impl AsRef<Path>, output: impl AsRef<Path>, key: &[u8], exclude: Vec<String>) {
    let input = input.as_ref();
    let output = output.as_ref();
    let always_exclude = vec!["manifest.json", "pack_icon.png", "bug_pack_icon.png"];
    let exclude: Vec<WildMatch> = exclude
        .iter()
        .map(|pattern| WildMatch::new(pattern))
        .collect();

    create_dir_all(output.parent().unwrap()).unwrap();

    // read manifest to verify if its a valid pack and to find content id
    let id = serde_json::from_reader::<_, Manifest>(
        File::open(&input.join("manifest.json")).expect("Failed to open manifest.json"),
    )
    .expect("Failed to parse manifest.json")
    .header
    .uuid;

    // create content list and copy or encrypt content
    let mut content_entries = Vec::new();
    for path in glob(&format!("{}/**/*", input.to_str().unwrap())).unwrap() {
        let input_entry_path = path.unwrap();
        if !input_entry_path.is_file() {
            continue;
        }

        let relative_path = input_entry_path
            .strip_prefix(input.to_str().unwrap())
            .unwrap()
            .to_str()
            .unwrap()
            .replace("\\", "/");
        let output_entry_path = output.join(&relative_path);
        create_dir_all(output_entry_path.parent().unwrap()).unwrap();

        content_entries.push(ContentEntry {
            key: if always_exclude.contains(&relative_path.as_str())
                || exclude
                    .iter()
                    .any(|pattern| pattern.matches(&relative_path))
            {
                if input_entry_path != output_entry_path {
                    if relative_path.ends_with(".json") {
                        // validate and shrink json
                        match serde_json::from_reader::<_, serde_json::Value>(
                            File::open(&input_entry_path).unwrap(),
                        ) {
                            Ok(value) => {
                                serde_json::to_writer(
                                    File::create(output_entry_path).unwrap(),
                                    &value,
                                )
                                .unwrap();
                            }
                            Err(_) => {
                                copy(input_entry_path, output_entry_path).unwrap();
                            }
                        }
                    } else {
                        copy(input_entry_path, output_entry_path).unwrap();
                    }

                    println!("Copied {}", relative_path);
                }
                None
            } else {
                let mut key_buffer = Vec::new();
                let mut rng = thread_rng();
                key_buffer
                    .write(
                        (0..32)
                            .map(|_| rng.sample(Alphanumeric) as char)
                            .collect::<String>()
                            .as_bytes(),
                    )
                    .unwrap();
                let key = from_utf8(&key_buffer).unwrap().to_owned();

                let mut file = File::open(&input_entry_path).unwrap();
                let mut buffer = Vec::new();
                if relative_path.ends_with(".json") {
                    // validate and shrink json
                    match serde_json::from_reader::<_, serde_json::Value>(&file) {
                        Ok(value) => {
                            buffer.append(&mut serde_json::to_vec(&value).unwrap());
                        }
                        Err(_) => {
                            file.rewind().unwrap();
                            file.read_to_end(&mut buffer).unwrap();
                        }
                    }
                } else {
                    file.read_to_end(&mut buffer).unwrap();
                }
                Aes256Cfb8Enc::new_from_slices(&key_buffer, &key_buffer[0..16])
                    .unwrap()
                    .encrypt(&mut buffer);
                File::create(output_entry_path)
                    .unwrap()
                    .write_all(&buffer)
                    .unwrap();

                println!("Encrypted {} with key {}", relative_path, key);

                Some(key)
            },
            path: relative_path,
        })
    }

    let mut file = File::create(output.join("contents.json")).unwrap();
    file.write_all(&[0x00u8, 0x00u8, 0x00u8, 0x00u8]).unwrap(); // version
    file.write_all(&[0xFCu8, 0xB9u8, 0xCFu8, 0x9Bu8]).unwrap(); // magic
    file.seek(SeekFrom::Start(0x10)).unwrap();
    let id_bytes = id.as_bytes();
    file.write_all(&[id_bytes.len() as u8]).unwrap(); // content id length
    file.write_all(id_bytes).unwrap(); // content id

    let content = Content {
        content: content_entries,
    };
    let mut buffer = serde_json::to_vec(&content).unwrap();
    Aes256Cfb8Enc::new_from_slices(&key, &key[0..16])
        .unwrap()
        .encrypt(&mut buffer);
    file.seek(SeekFrom::Start(0x100)).unwrap();
    file.write_all(&buffer).unwrap(); // encrypted content list

    println!("Encryption finished, key: {}", from_utf8(&key).unwrap());
}

fn decrypt(input: impl AsRef<Path>, output: impl AsRef<Path>, key: &[u8]) {
    let input = input.as_ref();
    let output = output.as_ref();

    let mut content = {
        let mut file = File::open(&input.join("contents.json")).unwrap();
        let mut buffer = Vec::new();
        file.seek(SeekFrom::Start(0x100)).unwrap();
        file.read_to_end(&mut buffer).unwrap(); // encrypted content list
        Aes256Cfb8Dec::new_from_slices(&key, &key[0..16])
            .unwrap()
            .decrypt(&mut buffer);
        serde_json::from_slice::<Content>(&buffer).expect("Failed to parse contents.json, the key might be wrong")
    };

    // ignore all entries but the first
    content.content.sort_by_key(|entry| entry.path.clone());
    content.content.dedup_by_key(|entry| entry.path.clone());

    // copy or decrypt content
    for content_entry in &content.content {
        let input_entry_path = input.join(&content_entry.path);
        if !input_entry_path.is_file() {
            continue;
        }

        let output_entry_path = output.join(&content_entry.path);
        create_dir_all(output_entry_path.parent().unwrap()).unwrap();

        match &content_entry.key {
            None => {
                if input_entry_path != output_entry_path {
                    if content_entry.path.ends_with(".json") {
                        // validate and prettify json
                        match serde_json::from_reader::<_, serde_json::Value>(
                            File::open(&input_entry_path).unwrap(),
                        ) {
                            Ok(value) => {
                                serde_json::to_writer_pretty(
                                    File::create(output_entry_path).unwrap(),
                                    &value,
                                )
                                .unwrap();
                            }
                            Err(_) => {
                                copy(input_entry_path, output_entry_path).unwrap();
                            }
                        }
                    } else {
                        copy(input_entry_path, output_entry_path).unwrap();
                    }

                    println!("Copied {}", &content_entry.path);
                }
            }
            Some(key) => {
                let key_bytes = key.as_bytes();
                if key_bytes.len() != 32 {
                    println!(
                        "Expected a key with a length of 32, got {}. Skipping {}",
                        key_bytes.len(),
                        content_entry.path
                    );
                    continue;
                }

                let mut file = File::open(&input_entry_path).unwrap();
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).unwrap();
                Aes256Cfb8Dec::new_from_slices(key_bytes, &key_bytes[0..16])
                    .unwrap()
                    .decrypt(&mut buffer);
                if content_entry.path.ends_with(".json") {
                    // validate and prettify json
                    match &serde_json::from_slice::<serde_json::Value>(&buffer) {
                        Ok(value) => {
                            serde_json::to_writer_pretty(
                                File::create(output_entry_path).unwrap(),
                                &value,
                            )
                            .unwrap();
                        }
                        Err(_) => {
                            File::create(output_entry_path)
                                .unwrap()
                                .write_all(&buffer)
                                .unwrap();
                        }
                    }
                } else {
                    File::create(output_entry_path)
                        .unwrap()
                        .write_all(&buffer)
                        .unwrap();
                }

                println!("Decrypted {} with key {}", &content_entry.path, key);
            }
        }
    }

    println!("Decryption finished");
}

#[derive(Serialize, Deserialize, Debug)]
struct Manifest {
    header: ManifestHeader,
}

#[derive(Serialize, Deserialize, Debug)]
struct ManifestHeader {
    uuid: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Content {
    content: Vec<ContentEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ContentEntry {
    path: String,
    key: Option<String>,
}

type Aes256Cfb8Enc = cfb8::Encryptor<Aes256>;
type Aes256Cfb8Dec = cfb8::Decryptor<Aes256>;
