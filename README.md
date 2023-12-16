# mcrputil

![license](https://img.shields.io/badge/License-Apache_2.0-blue.svg)
![version](https://img.shields.io/badge/Version-1.2.0-green.svg)

Minecraft Resource Pack Util for encrypting and decrypting resource packs.

## Usage

mcrputil is a command-line tool, and does not have a GUI.

```
Usage: mcrputil <COMMAND>

Commands:
  encrypt  Encrypts the folder with a given or auto-generated key
  decrypt  Decrypts the folder with a given key
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help information
```

### Encryption

```
Usage: mcrputil encrypt [OPTIONS] <INPUT> <OUTPUT>

Arguments:
  <INPUT>   Input folder
  <OUTPUT>  Output folder

Options:
  -k, --key <KEY>          Key used for encryption
  -e, --exclude <EXCLUDE>  Specifies files which should not be encrypted
  -h, --help               Print help information
```

#### Step-by-step

1. Make sure your pack is unzipped, and in your pack directory should be a manifest.json
2. ./mcrputil encrypt <path to your unzipped pack directory> <path to your unzipped pack/output directory>
3. A contents.json should now be in your pack/output directory, the key will be displayed in the console after "Encryption finished, key: <your key>"
4. Zip your output directory contents (not the directory itself), and copy your key depending on the software used to <your pack zip file>.key.

### Decryption

```
Usage: mcrputil decrypt [OPTIONS] <INPUT> <OUTPUT>

Arguments:
  <INPUT>   Input folder
  <OUTPUT>  Output folder

Options:
  -k, --key <KEY>  Key used for decryption
  -h, --help       Print help information
```

Please make sure, to not publish any of the resulting files, or only with the consent of the copyright holder, and note
that there will be no support for decrypting resource packs.

[![Lang-简体中文](https://img.shields.io/badge/Lang-%E7%AE%80%E4%BD%93%E4%B8%AD%E6%96%87-red)](README-zh_CN.md)
