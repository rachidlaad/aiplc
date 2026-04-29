<p align="center"><code>npm i -g @uxarion/aiplc</code><br />or <code>brew install --cask aiplc</code></p>
<p align="center"><strong>AIPLC CLI</strong> is a coding agent from Uxarion that runs locally on your computer.
<p align="center">
  <img src="https://github.com/rachidlaad/aiplc/blob/main/.github/codex-cli-splash.png" alt="AIPLC CLI splash" width="80%" />
</p>
</br>
Use AIPLC locally from your terminal, or integrate it into your editor and automation workflows through this repository’s CLI and SDK surfaces.</p>

---

## Quickstart

### Installing and running AIPLC CLI

Install globally with your preferred package manager:

```shell
# Install using npm
npm install -g @uxarion/aiplc
```

```shell
# Install using Homebrew
brew install --cask aiplc
```

Then simply run `aiplc` to get started.

<details>
<summary>You can also go to the <a href="https://github.com/rachidlaad/aiplc/releases/latest">latest GitHub Release</a> and download the appropriate binary for your platform.</summary>

Each GitHub Release contains many executables, but in practice, you likely want one of these:

- macOS
  - Apple Silicon/arm64: `aiplc-aarch64-apple-darwin.tar.gz`
  - x86_64 (older Mac hardware): `aiplc-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `aiplc-x86_64-unknown-linux-musl.tar.gz`
  - arm64: `aiplc-aarch64-unknown-linux-musl.tar.gz`

Each archive contains a single entry with the platform baked into the name (e.g., `aiplc-x86_64-unknown-linux-musl`), so you likely want to rename it to `aiplc` after extracting it.

</details>

### Using AIPLC with your ChatGPT plan

Run `aiplc` and select **Sign in with ChatGPT**. You can also use API-key-based access if your deployment is configured for it.

You can also use AIPLC with an API key, but this requires [additional setup](./docs/authentication.md).

## Docs

- [**AIPLC Docs**](./docs/install.md)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Open source fund**](./docs/open-source-fund.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).
