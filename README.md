# Heretek - Proxmox Mockup

Heretek is een lokale mockup-server en CLI-toolset geschreven in Rust. Het biedt een lichtgewicht manier om commando's te simuleren, testen of in de toekomst door te sturen naar een echte Proxmox-server via SSH.

## 🔧 Doel

- Simuleren van pct, zfs en andere Proxmox-achtige commando's.
- Ontwikkel- en testomgeving zonder directe toegang tot Proxmox.
- Ondersteuning voor verschillende Proxmox versies (6.x, 7.x, 8.x).
- In de toekomst uitbreidbaar naar echte verbindingen via SSH.

## 📦 Opbouw

1. heretekctl (de "control plane")

   - Luistert op poort 9001.
   - Ontvangt en verwerkt commando's.
   - Implementeert basis mock-functies voor pct en zfs commando's.

2. heretekd (de "dispatcher")
   - Luistert op poort 2222.
   - Ontvangt commando's van een client (bijv. een bash-script).
   - Gebruikt MockServer om commando's af te handelen of stuurt ze door naar heretekctl.
   - Ondersteunt verschillende Proxmox versies via configuratie.

## ▶️ Voorbeeldflow (lokaal)

[ bash script ] —> [ heretekd (port 2222) ] —> [ MockServer of heretekctl (port 9001) ]

## 🚀 Setup

### Vereisten

- Rust toolchain: https://rustup.rs
- cargo CLI
- netcat (voor tests via bash)

### Configuratie

In het bestand `config.toml` kun je de volgende instellingen aanpassen:

```toml
[heretekd]
host = "127.0.0.1"  # Server hostname
port = 2222         # Server poort

[heretekctl]
host = "127.0.0.1"  # Control hostname
port = 9001         # Control poort

[proxmox]
version = "8"       # Proxmox versie (6, 7 of 8)
```

Je kunt de huidige configuratie controleren met:

```bash
./check-version.sh
```

## Installatie

1. Clone deze repo:

```bash
git clone https://github.com/djverlinden/Heretek
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

# Voorbeeld om de Proxmox versie te wijzigen:
# RESPONSE=$(echo "heretek-setversion 7" | nc "$HOST" "$PORT")
# echo "Versie gewijzigd: $RESPONSE"
```

## 🧪 Testen

De codebase bevat verschillende testen:

- Unit tests: `cargo test`
- Integration tests: `cargo test -- --ignored`

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

- Elke component logt naar stdout met timestamp en emoji's voor visuele herkenning.
- Verbindingen, commando's en fouten worden live getoond.
- Bij het opstarten van de server wordt de gesimuleerde Proxmox versie weergegeven.

## 🌐 Roadmap

- SSH-ondersteuning voor verbinding met echte Proxmox-server
- Command history + replay
- WebSocket-ondersteuning
- Uitbreidbare mock-modules (pct, zfs, pveam, etc.)
- Uitgebreidere versie-specifieke configuratie en commando-ondersteuning

## 🔄 Proxmox Versies

Heretek ondersteunt het simuleren van drie Proxmox VE versies:

- **Proxmox VE 6.x** - Basis functionaliteit, sommige commando's zijn niet beschikbaar
- **Proxmox VE 7.x** - Uitgebreidere functionaliteit met meer commando-ondersteuning
- **Proxmox VE 8.x** - Alle geïmplementeerde functionaliteit beschikbaar (standaard)

Je kunt de versie op verschillende manieren wijzigen:

1. Via het `config.toml` bestand (permanent):
   ```toml
   [proxmox]
   version = "7"  # Verander naar 6, 7 of 8
   ```

2. Via een commando vanuit een extern programma (bijvoorbeeld in een bash script):
   ```bash
   # Wijzig naar Proxmox versie 7
   echo "heretek-setversion 7" | nc "$HOST" "$PORT"
   
   # Controleer huidige versie
   echo "pveversion" | nc "$HOST" "$PORT"
   ```

## 👨‍💻 Voor wie?

- Infra-architecten
- Systeembeheerders die lokaal willen testen
- Developers die willen mocken zonder side-effects op echte servers

## 📝 Beschikbare speciale commando's

Naast reguliere Proxmox commando's, ondersteunt Heretek ook speciale commando's:

- `pveversion` - Toont informatie over de huidige gesimuleerde Proxmox-versie
- `heretek-setversion <versie>` - Wijzigt de actieve Proxmox-versie (6, 7 of 8)

⸻

© 2023 - DJ Verlinden
