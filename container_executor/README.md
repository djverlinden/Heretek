# Container Executor Module for Heretek

This module provides a simple way to execute bash scripts in containers with Alpine Linux or Proxmox templates via SCP. It is designed to enable isolated execution without affecting the main application.

## Requirements

- Docker installed on your system
- Rust and Cargo
- sshpass (for password-based SCP)
- OpenSSH Client (scp and ssh commands)
- Optional: Proxmox VM templates (.tar.xz format)

## Usage

1. Install the required tools:
   ```
   # macOS
   brew install hudochenkov/sshpass/sshpass

   # Ubuntu/Debian
   apt-get install sshpass
   ```

2. Build the Docker image:
   ```
   cd container_executor
   docker build -t heretek-test-alpine .
   ```

3. Or use a Proxmox template:
   ```
   # Place the Proxmox template file in the container_executor directory
   # For example: alpine-3.12-default_2020-04-29_amd64.tar.xz
   
   # And run the example
   cargo run --example use_proxmox_template
   ```

4. Run the standard program:
   ```
   cargo run
   ```

## Functionality

- Execute bash scripts in an isolated container environment
- Support for Proxmox VM templates (Alpine, Debian)
- Scripts are sent to the container via SCP
- Container executes scripts via SSH
- Mount local directories in the container
- Error handling and logging
- Timeout mechanism for scripts

## Integration

This module is currently standalone and not yet integrated with the rest of the Heretek project. It can be used as an execution engine for container-based scripts in the Heretek project.

## How it works

1. Starts a Docker container with SSH server (standard or from Proxmox template)
2. Copies your bash script via SCP to the container
3. Executes the script via SSH in the container
4. Returns the output
5. Cleans up the container afterwards

## Proxmox Templates

The module can work directly with Proxmox VM templates:

1. Supported formats: .tar, .tar.gz, .tar.xz
2. Place your template file in the project directory
3. Use the `DockerTest::with_proxmox_template()` function
4. The module will automatically build a Docker image from the template