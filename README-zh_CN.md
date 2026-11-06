# ualias

`ualias` 根据 `/etc/udev/ualias.d/*` 中的 drop-in 文件查找或追加 USB 设备别名。

## 用法

```bash
ualias [选项]... [序列号 [名称]]
```

选项：

- `-w`, `--vendor ID` — USB 厂商 ID（十六进制）
- `-p`, `--product ID` — USB 产品 ID（十六进制）
- `-s`, `--serial SERIAL` — 设备序列号 / MAC
- `-n`, `--name NAME` — 别名（追加到 `ualias.d/local`）
- `-u`, `--ucfirst` — 将别名首字母大写
- `-h`, `--help`
- `--version`

未指定名称时，输出匹配到的别名（无匹配则退出码 `1`）。指定名称时追加到
`local`；未给出的厂商/产品/序列号字段写为 `*`。

目录中的文件按文件名排序读取（跳过隐藏文件、备份与打包残留）。示例：

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

ESP32-S3（`303a:1001`）若序列号未单独列出，会回落到 `s3g`。

可用 `UALIAS_DIR`（或旧名 `UALIAS_DEVICES`）覆盖目录路径。

## 构建与测试

### 构建依赖

```bash
sudo apt install build-essential meson ninja-build pkgconf cargo rustup gettext asciidoctor
```

### 用 Cargo 构建

```bash
cargo build --release
```

### 用 Meson 构建

```bash
meson setup /build
ninja -C /build
```

### 运行测试

```bash
cargo test
meson test -C /build
```

## 国际化

翻译位于 `po/`。运行时可设 `UALIAS_LOCALEDIR`。

```bash
ninja -C /build posync
LANGUAGE=zh_CN /path/to/ualias -h
```

## 安装

```bash
meson install -C /build
# 或
ninja -C /build install-symlinks
```

## Debian 包

```bash
dpkg-buildpackage -us -uc
```

## 许可证

Copyright (C) 2026 Lenik <ualias@bodz.net>

以 **AGPL-3.0-or-later** 许可发布。
本项目明确反对 AI 剥削与 AI 霸权，并拒绝无脑 MIT 式许可证与政治幼稚的 BSD
式许可证。完整文本与补充条款见 `LICENSE`。
