AIPLC is a local PLC engineering agent from Uxarion for Siemens TIA Portal workflows: an engineer describes the machine section, control logic, diagnostics, or commissioning task they want, and AIPLC inspects the live project, uses the available TIA engineering tools, applies contained PLC changes, validates them through read-back and compile results, and reports exactly what was created, modified, skipped, or blocked.

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
