# Proxmox Simulation Commands

This directory contains definitions of commands used in the Proxmox simulation environment. The commands are organized in different structures that reflect specific versions and functionality.

## Directory Structure

The simulation supports Proxmox VE versions 6, 7, and 8. The commands are organized as follows:

```
commands/
├── common/        # Commands common to all versions
├── v6/            # Proxmox VE 6.x specific commands
├── v7/            # Proxmox VE 7.x specific commands
└── v8/            # Proxmox VE 8.x specific commands
```

## Command Categories

In each directory, commands are grouped by category in different YAML files:

- **container.yaml**: LXC container-related commands (pct)
- **storage.yaml**: Storage-related commands (zfs, pvesm)
- **system.yaml**: System-related commands (network, cluster)
- **version.yaml**: Version-specific information (pveversion)
- **vm.yaml**: VM-related commands (qm)

## Usage

When executing a command, the system first looks for an exact match in the version-specific directory. If the command is not found there, it searches the `common` directory for commands that apply to all versions.

When new functionality is only available in a specific Proxmox version, it should be added to the corresponding version directory.

## Format

Each YAML file uses the following format:

```yaml
# Description of the command category

"command string": |
  output line 1
  output line 2
  ...

"another command": |
  other output
  ...
```

## Updating Versions

When adding support for a new Proxmox version:
1. Create a new directory (for example `v9/`)
2. Copy relevant YAML files from the previous version
3. Update/add version-specific commands
4. Update shared commands in the `common` directory if necessary