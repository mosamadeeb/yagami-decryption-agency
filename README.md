# yagami-decryption-agency

Decrypts/encrypts Judgment and Lost Judgment PC chara.par archives

# Installation
Download the [latest release](https://github.com/SutandoTsukai181/yagami-decryption-agency/releases/latest).

# Usage

```
USAGE:
    yagami-decryption-agency.exe [OPTIONS] <INPUT> [ARGS]

ARGS:
    <INPUT>       Path to input file
    <OUTPUT>      Path to output file. Defaults to input with ".decrypted.par" as the extension
    <MODE>        Operation mode [default: auto] [possible values: auto, decrypt, encrypt]
    <PAR_TYPE>    Type of the encrypted PAR file [default: auto] [possible values: auto, chara,
                  chara2]

OPTIONS:
    -h, --help         Print help information
    -o, --overwrite    Overwrite files without asking
    -V, --version      Print version information
```
