<table width="100%">
  <tr>
    <td align="left" valign="middle" style="font-size: 2em; font-weight: bold;">
      Cave CLI
    </td>
    <td align="right" valign="middle" style="width: 150px;">
      <img src="assets/logo-simvia.svg" alt="Simvia Logo" width="120" />
    </td>
  </tr>
</table>


[![Version](https://img.shields.io/badge/version-0.1.0-blue)](https://github.com/simvia/cave/releases)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)

> CLI tool to manage code_aster versions.

---
<!-- 
## 🎬 Demo

![Cave CLI demo](assets/cave.gif)

--- -->

## ⚡ Features

- `cave run` – run Docker images
- `cave list` – list available images
- `cave available` – list images on DockerHub
- `cave config` – configure Cave
- `cave use` / `cave pin` – manage versions
- Full autocompletion for Bash/Zsh
- Man page available

---

## 🛠 Installation

### Prerequisites

You need to install the Protocol Buffers compiler (`protoc`) to build this project from source:

```bash
sudo apt-get install protobuf-compiler
```

### Binary Package Installation

You can install cave via the prebuilt .deb (Debian/Ubuntu) or .rpm (Fedora/CentOS/RHEL) package. This method automatically configures the PATH and installs the man page.

#### Debian / Ubuntu (.deb)

1. Download the latest .deb from the official release page.

2. Install it using apt to automatically resolve dependencies:
```bash
sudo dpkg -i cave_0.1.0-1_amd64.deb
```
cave binary will be installed in /usr/bin/cave.

Verify installation:
```bash
cave --version
cave --help
man cave
```
#### RedHat / Fedora (.rpm)

1. Download the latest .rpm from the official release page.

2. Install it using dnf or yum:

```bash
sudo dnf install cave-1.0.0-1.x86_64.rpm
# or for older systems
sudo yum install cave-1.0.0-1.x86_64.rpm
```

Verify installation:
```bash
cave --version
cave --help
man cave
```

### Shell Completion

For `zsh` users, you can enable auto-completion as follows:

- Download the appropriate auto-completion script for your distribution from [URL to be provided].
- Add the following line to your shell configuration file (e.g., `~/.bashrc`):

```bash
source /full/path/to/<auto-completion-script>
```

- Restart your shell or run:

```bash
source ~/.bashrc
```

This setup does not require administrator privileges and keeps the configuration local to your user. After setup, typing `cave <TAB>` will suggest available subcommands and arguments.

### Configuration Files

- `~/.caveconfig`: User-level configuration file
- `./.cave`: Project-level pinned version configuration (local override)


## Usage

For detailed user documentation, please visit our [User Documentation](https://github.com/yourusername/yourrepo).

## Telemetry 

`cave` includes optional telemetry features to help improve the tool by collecting anonymous usage data. You can control telemetry settings via the configuration commands.

By default, version usage tracking is **enabled**, sending anonymous data about which code_aster versions you run. You can disable this tracking if you prefer.

To deactivate telemetry, use:

```bash
cave config disable-usage-tracking
```
Telemetry respects your privacy and does not collect sensitive information.

## See Also

- [Code_Aster](https://www.code-aster.org)
- [Docker](https://www.docker.com)
- [Simvia Docker Hub](https://hub.docker.com/r/simvia/code_aster)


## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

