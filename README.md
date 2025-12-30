<p align="center"><img src="assets/logo-simvia.svg" alt="Simvia Logo" width="50%" /></p>

<p align="center">
  <a href="https://github.com/simvia/cave/releases"><img src="https://img.shields.io/badge/version-0.1.0-blue" alt="Version" /></a>
  <a href="https://github.com/simvia-tech/cave/actions/workflows/pr.yml"><img src="https://github.com/simvia-tech/cave/actions/workflows/pr.yml/badge.svg" alt="CI-CD" /></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-MIT-green" alt="License" /></a>
</p>

# CAVE CLI

CAVE is a CLI tool that leverages Docker to conveniently manage multiple code_aster versions on your local environment, thanks to [Simvia](https://simvia.tech)'s [code_aster docker images](https://hub.docker.com/r/simvia/code_aster) .

## ⚡ Features

- `cave run` – run Docker images
- `cave list` – list available images
- `cave available` – list images on DockerHub
- `cave config` – configure Cave
- `cave use` / `cave pin` – manage versions
- Man page available

## 🛠 Installation

> This software require to first having installed docker.

### Installation with linux packages

#### Debian based

Fetch the [latest release](https://github.com/simvia-tech/cave/releases) .deb file, then run :

```bash
sudo dpkg -i cave_version.deb
```

#### RedHat / Fedora (.rpm)

Fetch the [latest release](https://github.com/simvia-tech/cave/releases) .rpm file, then run :

```bash
sudo dnf install cave_version.rpm
# or for older systems
sudo yum install cave_version.rpm
```

### Binary install

Download the pre‑built tarball, extract the `cave` binary .tar.gz

```bash
tar -xzf ./cave_version.tar.gz -C /tmp
sudo mv /tmp/cave /usr/local/bin/
```

### Verify installation:

```bash
cave --version
cave --help
man cave
```

At each run, `cave` will check for updates and notify you if a new version is available. To disable this feature, use:

```bash
cave config disable-update-check
```

## Usage

For detailed user documentation, please visit our [User Documentation](https://github.com/simvia-tech/cave/blob/dev/docs/man.md).

## Telemetry 

`cave` includes optional telemetry features to help improve the tool by collecting anonymous usage data. You can control telemetry settings via the configuration commands.

By default, version usage tracking is **enabled**, sending anonymous data (see [Telemetry](https://github.com/simvia-tech/cave/blob/dev/docs/telemetry.md)) about which code_aster versions you run. You can disable this tracking if you prefer.

To deactivate telemetry, use:

```bash
cave config disable-usage-tracking
```
Telemetry respects your privacy and does not collect sensitive information.

## Contributors

- Lucas S.
- Basile M.
- Hadrien R.

## See Also

- [code_aster](https://www.code-aster.org)
- [Simvia Docker Hub](https://hub.docker.com/r/simvia/code_aster)
- [code_aster catalog](https://simvia-tech.github.io/code-aster-dockerhub/)

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

# Reach Us

We love feedback.
Don't hesitate to open a [Github issue](https://github.com/simvia-tech/cave/issues/new) or
feel free to reach us on our website [https://simvia.tech/fr#contact](https://simvia.tech/fr#contact)