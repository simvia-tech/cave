## User Documentation

`cave` is a command-line tool to manage multiple versions of the **code_aster** finite element solver via Docker, enabling easy version switching, reproducibility, and automation.

### Synopsis

```bash
cave [-h|--help] [-V|--version] <subcommand> [options]
```


### Description

`cave` simplifies using different versions of code_aster by leveraging Docker images from Docker Hub. It is designed for environments where version control, reproducibility, and automation of simulations are critical.

***

### Global Options

- `-h`, `--help`
Show help information and exit.
- `-V`, `--version`
Display the current version of the `cave` binary.

***

### Subcommands

#### `use`

Set a global default code_aster version for all projects (unless overridden locally).

```bash
cave use <version>
```

- `<version>` can be `stable`, `testing`, or a specific version like `17.2.24`.
- Stores preference in a `.cave` file in your home directory.

**Example:**

```bash
cave use 17.3.1
```

Makes version `17.3.1` the default for all future `cave run` commands unless a project version is pinned.

***

#### `pin`

Pin a specific code_aster version to the current working directory, overriding the global version.

```bash
cave pin <version>
```

- `<version>` can be `stable`, `testing`, or a specific version like `17.2.24`.
- Creates or updates a local `.cave` file for that project folder.

**Example:**

```bash
cave pin testing
```

After pinning, all runs in this directory use the pinned version.

***

#### `run`

Run a simulation using the pinned or global code_aster version.

```bash
cave run -- [code_aster options/arguments]
```

- Pass code_aster CLI arguments after `--`.
- Requires Docker to be installed and running.
- Simulation output is streamed live.

**Example:**

```bash
cave run -- calcul.export
cave run -- -i
```


***

#### `list`

Show locally downloaded code_aster versions (Docker images).

```bash
cave list [version_prefix]
```

Filter by optional version prefix:

```bash
cave list 17.2
```


***

#### `available`

List all officially published code_aster versions on Docker Hub.

```bash
cave available [version_prefix]
```

Versions already downloaded are shown in blue. The stable and testing tags are also highlighted.

Filter example:

```bash
cave available 17.2
```


***

#### `config`

Manage persistent global configuration settings stored in `~/.caveconfig`.

```bash
cave config <option>
```

Available options:

- `enable-auto-update`
Enable automatic version updates for stable or testing versions.

```bash
cave config enable-auto-update
```

- `disable-auto-update`
Disable automatic update checks (default behavior).

```bash
cave config disable-auto-update
```

- `enable-usage-tracking` (default)
Enables anonymous telemetry that tracks which code_aster versions you run. This data helps improve `cave`. No personal or sensitive data is collected.

```bash
cave config enable-usage-tracking
```

- `disable-usage-tracking`
Turns off telemetry if you do not wish to share usage information.

```bash
cave config disable-usage-tracking
```

These configuration options are stored in your user-level configuration file `~/.caveconfig` and apply to all projects globally.

If you need more detailed help on any command, run:

```bash
cave <subcommand> --help
```
