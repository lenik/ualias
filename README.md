# ualias

`ualias` looks up or appends USB device aliases using drop-in files under
`/etc/udev/ualias.d/*`.

## Usage

```bash
ualias [OPTION]... [SERIAL [NAME]]
```

Options:

- `-w`, `--vendor ID` — USB vendor id (hex)
- `-p`, `--product ID` — USB product id (hex)
- `-s`, `--serial SERIAL` — device serial / MAC
- `-n`, `--name NAME` — alias name (appends to `ualias.d/local`)
- `-u`, `--ucfirst` — uppercase the first letter of the name
- `-h`, `--help`
- `--version`

Without a name, print the matched alias (exit `1` if none). With a name,
append a pattern to `local`; omitted vendor/product/serial fields are stored
as `*`.

Files in the directory are read in sorted name order (hidden / backup /
packaging leftovers skipped). Example layout:

```text
/etc/udev/ualias.d/10-serials
/etc/udev/ualias.d/20-fallback
```

```text
# 10-serials
#VENDOR PRODUCT SERIAL NAME
* * 44:1B:F6:D7:92:5C s3cam
* * 10:20:BA:3D:30:28 s3tlcd3.49in

# 20-fallback
303a 1001 * s3g
```

An ESP32-S3 device (`303a:1001`) whose serial is not listed falls through to
`s3g`.

Override the directory with `UALIAS_DIR` (or legacy `UALIAS_DEVICES`).

## Build and test

### Build dependencies (Debian example)

```bash
sudo apt install build-essential meson ninja-build pkgconf cargo rustup gettext asciidoctor
```

### Configure and build (Cargo)

```bash
cargo build --release
# ./target/release/ualias
```

### Configure and build (Meson)

```bash
meson setup /build
ninja -C /build
```

### Run tests

```bash
cargo test
# or, after a Meson setup
meson test -C /build
```

## i18n (gettext)

Translations live under `po/`. Runtime override: `UALIAS_LOCALEDIR`.

```bash
ninja -C /build posync
LANGUAGE=zh_CN /path/to/ualias -h
```

## Install

```bash
meson install -C /build
# or
ninja -C /build install-symlinks
```

## Debian package

```bash
dpkg-buildpackage -us -uc
```

## License

Copyright (C) 2026 Lenik <ualias@bodz.net>

Licensed under **AGPL-3.0-or-later**.
This project explicitly opposes AI exploitation and AI hegemony, and rejects
mindless MIT-style licensing and politically naive BSD-style licensing.
See `LICENSE` for the full text and supplemental project terms.
