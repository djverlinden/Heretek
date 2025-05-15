# Heretek - Proxmox Mockup

Heretek is a local mockup server and CLI toolset written in Rust. It provides a lightweight way to simulate, test commands or in the future forward them to a real Proxmox server via SSH.

## 🔧 Goal

- Simulate pct, zfs and other Proxmox-like commands.
- Development and test environment without direct access to Proxmox.
- Support for different Proxmox versions (6.x, 7.x, 8.x).
- Expandable to real connections via SSH in the future.

## 📦 Structure

1. heretekctl (the "control plane")

   - Listens on port 9001.
   - Receives and processes commands.
   - Implements basic mock functions for pct and zfs commands.

2. heretekd (the "dispatcher")
   - Listens on port 2222.
   - Receives commands from a client (e.g., a bash script).
   - Uses MockServer to handle commands or forwards them to heretekctl.
   - Supports different Proxmox versions via configuration.

## ▶️ Example Flow (local)

[ bash script ] —> [ heretekd (port 2222) ] —> [ MockServer or heretekctl (port 9001) ]

## 🚀 Setup

### Requirements

- Rust toolchain: https://rustup.rs
- cargo CLI
- netcat (for tests via bash)

### Configuration

In the `config.toml` file you can adjust the following settings:

```toml
[heretekd]
host = "127.0.0.1"  # Server hostname
port = 2222         # Server port

[heretekctl]
host = "127.0.0.1"  # Control hostname
port = 9001         # Control port

[proxmox]
version = "8"       # Proxmox version (6, 7 or 8)
```

You can check the current configuration with:

```bash
./check-version.sh
```

## Installation

1. Clone this repo:

```bash
git clone https://github.com/djverlinden/Heretek
cd heretek
```

2. Add dependencies:

```bash
cargo add tokio --features full
cargo add chrono
cargo add serde
```

3. Start both services:

Terminal 1 – start heretekctl:

```bash
cargo run -p heretekctl
```

Terminal 2 – start heretekd:

```bash
cargo run -p heretekd
```

Test via bash:

```bash
./test-command.sh
```

Content of test-command.sh:

```bash
#!/bin/bash
HOST="127.0.0.1"
PORT="2222"
COMMAND="pct list"

echo "🔧 Sending command: '$COMMAND' to $HOST:$PORT"
echo "$COMMAND" | nc "$HOST" "$PORT"
echo "✅ Command sent"

# Example to change the Proxmox version:
# RESPONSE=$(echo "heretek-setversion 7" | nc "$HOST" "$PORT")
# echo "Version changed: $RESPONSE"
```

## 🧪 Testing

The codebase contains various tests:

- Unit tests: `cargo test`
- Integration tests: `cargo test -- --ignored`

### Writing tests

1. **MockServer tests:**

   ```rust
   #[test]
   fn test_new_function() {
       let mock = MockServer::new();
       let response = mock.handle_command("new command");
       assert!(response.contains("expected_output"));
   }
   ```

2. **Client-server tests:**
   ```rust
   #[test]
   #[ignore] // Use --ignored flag during test execution
   fn test_server_response() {
       let mut stream = TcpStream::connect("127.0.0.1:2222").unwrap();
       // Test logic here
   }
   ```

## 🔍 Logging & Debugging

- Each component logs to stdout with timestamp and emojis for visual recognition.
- Connections, commands, and errors are displayed live.
- When starting the server, the simulated Proxmox version is displayed.

## 🌐 Roadmap

- SSH support for connection with real Proxmox server
- Command history + replay
- WebSocket support
- Extensible mock modules (pct, zfs, pveam, etc.)
- More extensive version-specific configuration and command support

## 🔄 Proxmox Versions

Heretek supports simulating three Proxmox VE versions:

- **Proxmox VE 6.x** - Basic functionality, some commands are not available
- **Proxmox VE 7.x** - More extensive functionality with more command support
- **Proxmox VE 8.x** - All implemented functionality available (default)

You can change the version in different ways:

1. Via the `config.toml` file (permanent):
   ```toml
   [proxmox]
   version = "7"  # Change to 6, 7 or 8
   ```

2. Via a command from an external program (for example in a bash script):
   ```bash
   # Change to Proxmox version 7
   echo "heretek-setversion 7" | nc "$HOST" "$PORT"
   
   # Check current version
   echo "pveversion" | nc "$HOST" "$PORT"
   ```

## 👨‍💻 For whom?

- Infrastructure architects
- System administrators who want to test locally
- Developers who want to mock without side effects on real servers

## 📝 Available special commands

Besides regular Proxmox commands, Heretek also supports special commands:

- `pveversion` - Shows information about the current simulated Proxmox version
- `heretek-setversion <version>` - Changes the active Proxmox version (6, 7 or 8)

⸻

© 2023 - DJ Verlinden
