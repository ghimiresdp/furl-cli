# fURL (`furl-cli`)

A fast, multithreaded CLI downloader built in Rust.

furl is a high-performance command-line tool designed to download files faster
by utilizing multiple threads to fetch chunks of data concurrently. Inspired by
the simplicity of cURL and the robustness of wget.

![Example image](res/images/example.png)

> [!NOTE]
> furl is a successor to my previous project
> [ghimiresdp/rust-raid](https://github.com/ghimiresdp/rust-raid)'s
> `cget` download manager.
> It incorporates refined logic and improved multithreading from that original
> implementation.

## ✨ Features

- **Parallel Downloads**: Automatically splits large files into chunks and
  downloads them across multiple threads.
- **Modern Async**: Built on top of `tokio` and `reqwest` for maximum efficiency.
- **Visual Progress**: Beautiful, real-time progress bars using `indicatif`.
- **Rust Powered**: Memory-safe and "fearless" concurrency.

## 🚀 Installation

### From Crates.io (Recommended)

```bash
cargo install furl-cli
```

### From Source

```bash
git clone https://github.com/ghimiresdp/furl-cli.git
cd furl
cargo build --release
```

## 🛠 Usage

### without output directory

when no output directory is passed, it automatically downloads the file in the
current terminal directory (PWD).

```bash
furl [URL]
```

**Example:**

```bash
furl https://example.com/large-file.iso
```

### with output directory

```bash
furl [URL] -o [path/to/the/directory]
```

**Example:**

```bash
furl https://example.com/large-file.iso -o ~/Downloads
```

## 🗺 Roadmap

- [x] Multithreaded chunk downloading
- [x] Customize number of threads with arguments
- [ ] Basic CLI argument parsing (clap)
- [x] Real-time progress bars
- [ ] Resume interrupted downloads (Checkpoints)
- [ ] Support for Proxy and Basic Auth
- [ ] Config file support (furl.toml)

## 🤝 Contributing

Contributions are welcome! Since this is a WIP, please open an issue first to
discuss the changes you'd like to make.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)`
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

For more details, please check [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

This project is licensed under the Apache License 2.0.
See the [LICENSE](LICENSE) file for details.

### Third-Party Licenses

This software uses several open-source components. You can view the full list of
dependencies and their licenses using `cargo-license`:
