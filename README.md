# fURL (`furl-cli`)

A fast, multithreaded CLI downloader built in Rust.

furl is a high-performance command-line tool designed to download files faster
by utilizing multiple threads to fetch chunks of data concurrently. Inspired by
the simplicity of cURL and the robustness of wget.

> [!NOTE]
> ⚠️ WORK IN PROGRESS: This project is currently in early development.
> The core multithreaded engine is functional, but features like advanced
> authentication and complex retry logic are still being implemented.
>
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

```bash
furl [URL] -o [FILENAME] -t [THREADS]
```

**Example:**

```bash
furl https://example.com/large-file.iso -o my-file.iso -t 8
```

## 🗺 Roadmap

- [x] Multithreaded chunk downloading
- [ ] Basic CLI argument parsing (clap)
- [x] Real-time progress bars
- [ ] Resume interrupted downloads (Checkpoints)
- [ ] Support for Proxy and Basic Auth
- [ ] Config file support (furl.toml)

## 🤝 Contributing

Contributions are welcome! Since this is a WIP, please open an issue first to discuss the changes you'd like to make.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)`
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

For more details, please check [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.

### Third-Party Licenses

This software uses several open-source components. You can view the full list of dependencies and their licenses using `cargo-license`:
