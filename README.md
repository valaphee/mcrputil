# mcrputil

![license](https://img.shields.io/badge/License-Apache_2.0-blue.svg)
![version](https://img.shields.io/badge/Version-1.1.5-green.svg)
[![Lang-简体中文](https://img.shields.io/badge/Lang-%E7%AE%80%E4%BD%93%E4%B8%AD%E6%96%87-red)](README-zh_CN.md)

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
  <INPUT>   Input file or folder
  <OUTPUT>  Output folder

Options:
  -k, --key <KEY>          Key used for encryption
  -e, --exclude <EXCLUDE>  Specifies files which should not be encrypted
  -h, --help               Print help information
```

### Decryption
```
Usage: mcrputil decrypt [OPTIONS] <INPUT> <OUTPUT>

Arguments:
  <INPUT>   Input file or folder
  <OUTPUT>  Output folder

Options:
  -k, --key <KEY>  Key used for decryption
  -h, --help       Print help information
```
