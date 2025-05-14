# Heretek - Proxmox Mockup

Heretek is een lokale mockup-server en CLI-toolset geschreven in Rust. Het biedt een lichtgewicht manier om commando's te simuleren, testen of in de toekomst door te sturen naar een echte Proxmox-server via SSH.

## 🔧 Doel
* Simuleren van pct, zfs en andere Proxmox-achtige commando's.
* Ontwikkel- en testomgeving zonder directe toegang tot Proxmox.
* In de toekomst uitbreidbaar naar echte verbindingen via SSH.

## 📦 Opbouw

1. heretekctl (de "control plane")
   * Luistert op poort 9001.
   * Ontvangt en verwerkt commando's.
   * Implementeert basis mock-functies voor pct en zfs commando's.

2. heretekd (de "dispatcher")
   * Luistert op poort 2222.
   * Ontvangt commando's van een client (bijv. een bash-script).
   * Gebruikt MockServer om commando's af te handelen of stuurt ze door naar heretekctl.

## ▶️ Voorbeeldflow (lokaal)

[ bash script ] —> [ heretekd (port 2222) ] —> [ MockServer of heretekctl (port 9001) ]

## 🚀 Setup

### Vereisten
* Rust toolchain: https://rustup.rs
* cargo CLI
* netcat (voor tests via bash)

## Installatie

1. Clone deze repo:
```bash
git clone https://github.com/jouwnaam/heretek.git
cd heretek
```

2. Voeg dependencies toe:
```bash
cargo add tokio --features full
cargo add chrono
cargo add serde
```

3. Start beide services:

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

Inhoud van test-command.sh:
```bash
#!/bin/bash
HOST="127.0.0.1"
PORT="2222"
COMMAND="pct list"

echo "🔧 Verstuur commando: '$COMMAND' naar $HOST:$PORT"
echo "$COMMAND" | nc "$HOST" "$PORT"
echo "✅ Command verzonden"
```

## 🧪 Testen

De codebase bevat verschillende testen:
* Unit tests: `cargo test`
* Integration tests: `cargo test -- --ignored`

### Het schrijven van tests

1. **MockServer tests:**
   ```rust
   #[test]
   fn test_nieuwe_functie() {
       let mock = MockServer::new();
       let response = mock.handle_command("nieuw commando");
       assert!(response.contains("verwachte_uitvoer"));
   }
   ```

2. **Client-server tests:**
   ```rust
   #[test]
   #[ignore] // Gebruik --ignored vlag bij test uitvoering
   fn test_server_response() {
       let mut stream = TcpStream::connect("127.0.0.1:2222").unwrap();
       // Test logica hier
   }
   ```

## 🔍 Logging & Debugging
* Elke component logt naar stdout met timestamp en emoji's voor visuele herkenning.
* Verbindingen, commando's en fouten worden live getoond.

## 🌐 Roadmap
* SSH-ondersteuning voor verbinding met echte Proxmox-server
* Command history + replay
* WebSocket-ondersteuning
* Uitbreidbare mock-modules (pct, zfs, pveam, etc.)

## 👨‍💻 Voor wie?
* Infra-architecten
* Systeembeheerders die lokaal willen testen
* Developers die willen mocken zonder side-effects op echte servers

⸻

© 2023 - DJ Verlinden
