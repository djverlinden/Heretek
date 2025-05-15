# Container Executor Module voor Heretek

Deze module biedt een eenvoudige manier om bash-scripts uit te voeren in containers met Alpine Linux of Proxmox-templates. Het is ontworpen voor geïsoleerde uitvoering zonder de hoofdapplicatie te beïnvloeden.

## Vereisten

- Docker geïnstalleerd op je systeem
- Rust en Cargo
- Docker daemon draaiend
- Optioneel: Proxmox VM templates (.tar.xz formaat)

## Gebruik

1. Installeer een template image:
   ```
   cd container_executor
   cargo run -- install alpine      # Installeert alpine:latest
   cargo run -- install debian      # Installeert debian:latest
   cargo run -- install alpine 3.16 # Installeert specifieke versie
   ```

2. Start een langlopende container:
   ```
   # Start een container met standaard instellingen
   cargo run -- start alpine
   
   # Start met poort forwarding
   cargo run -- start alpine --port 8080:80
   
   # Start met custom naam en mount
   cargo run -- start debian --name mijn-container --mount /pad/naar/data
   ```

3. Voer een script uit in een container:
   ```
   # Maak een test script
   echo 'echo "Hello from container"' > test.sh
   
   # Voer het uit in een container
   cargo run -- run test.sh
   
   # Met opties
   cargo run -- run test.sh --mount /pad/naar/data --timeout 30
   ```

4. Opschonen van Docker resources:
   ```
   # Verwijder alleen zwevende images
   cargo run -- cleanup
   
   # Verwijder alle heretek-images
   cargo run -- cleanup --all
   
   # Verwijder ook base images
   cargo run -- cleanup --all --include-base
   
   # Forceer opschonen van draaiende containers
   cargo run -- cleanup --force
   
   # Voor volledig opschonen (inclusief database)
   ./cleanup.sh
   ```

5. Tonen van bijgehouden resources:
   ```
   # Toon alle bijgehouden resources
   cargo run -- list
   
   # Toon alleen containers
   cargo run -- list --containers
   
   # Toon alleen images
   cargo run -- list --images
   
   # Toon gedetailleerde informatie
   cargo run -- list --detailed
   ```

6. Uitvoeren van tests:
   ```
   cargo test
   # Of voor tests die Docker vereisen:
   cargo test -- --ignored
   ```

## Commando-overzicht

### `install` - Installeren van container templates
```
cargo run -- install <TEMPLATE> [VERSION]

Argumenten:
  <TEMPLATE>  Template type (alpine of debian)
  [VERSION]   Versie van de template, bijv. 3.16 [standaard: latest]

Voorbeelden:
  cargo run -- install alpine
  cargo run -- install debian
  cargo run -- install alpine 3.16
```

### `start` - Starten van langlopende containers
```
cargo run -- start [OPTIES] [TEMPLATE] [VERSION]

Argumenten:
  [TEMPLATE]  Template type (alpine of debian)
  [VERSION]   Versie van de template, bijv. 3.16

Opties:
  -m, --mount <PAD>    Mount een directory in de container
  -p, --port <MAPPING> Expose ports (format: host:container, bijv. 8080:80)
  -n, --name <NAAM>    Custom naam voor de container

Voorbeelden:
  cargo run -- start alpine
  cargo run -- start debian --port 8080:80
  cargo run -- start alpine --name mijn-service --mount /pad/naar/data
```

### `run` - Uitvoeren van scripts in containers
```
cargo run -- run [OPTIES] <SCRIPT>

Argumenten:
  <SCRIPT>    Pad naar het bash script dat uitgevoerd moet worden

Opties:
  -t, --template <TEMPLATE>  Template om te gebruiken (alpine of debian)
  -v, --version <VERSION>    Versie van de template, bijv. 3.16
  -t, --timeout <TIMEOUT>    Optionele timeout in seconden [standaard: 30]
  -m, --mount <PAD>          Mount een directory in de container

Voorbeelden:
  cargo run -- run mijn_script.sh
  cargo run -- run test.sh --timeout 60
  cargo run -- run deploy.sh --template debian --version 11
```

### `list` - Tonen van bijgehouden resources
```
cargo run -- list [OPTIES]

Opties:
  -c, --containers  Toon alleen containers
  -i, --images      Toon alleen images
  -d, --detailed    Toon gedetailleerde informatie

Voorbeelden:
  cargo run -- list
  cargo run -- list --containers
  cargo run -- list --detailed
```

### `cleanup` - Opschonen van Docker resources
```
cargo run -- cleanup [OPTIES]

Opties:
  -a, --all           Verwijder alle heretek images, niet alleen zwevende
      --include-base  Ook base images verwijderen (alpine, debian)
  -f, --force         Forceer verwijderen van draaiende containers

Voorbeelden:
  cargo run -- cleanup
  cargo run -- cleanup --all
  cargo run -- cleanup --force
```

## Functionaliteit

- Uitvoeren van bash scripts in een geïsoleerde containeromgeving
- Ondersteuning voor Proxmox VM templates (Alpine, Debian)
- Scripts worden uitgevoerd in de container
- Starten van langlopende containers voor doorlopende taken
- Mounten van lokale mappen in de container
- Foutafhandeling en logging
- Timeout-mechanisme voor scripts
- DockerTest en DockerService API voor testen
- Opschonen van Docker resources (containers en images)
- Automatische tracking van gemaakte resources via een sled database
- Overzicht van bijgehouden resources via list commando
- Cleanup script voor volledig opschonen van alle resources
- Uitgebreide command-line interface met subcommando's

## Integratie

Deze module werkt als een standalone component en kan worden gebruikt als uitvoeringsengine voor containergebaseerde scripts in het Heretek-project.

## Hoe het werkt

1. Start een Docker container (standaard of vanuit Proxmox template)
2. Registreert de container en images in een lokale sled database
3. Containers kunnen tijdelijk zijn (voor scripts) of permanent (voor services)
4. Kopieert je bash script naar de container bij script-uitvoering
5. Geeft de uitvoer terug bij script-uitvoering
6. Ruimt tijdelijke containers automatisch op na gebruik
7. Met het list commando kun je zien welke resources worden bijgehouden
8. Met het start commando kun je langlopende containers starten
9. Met het cleanup commando kunnen alle geregistreerde resources worden opgeschoond
10. Het cleanup.sh script biedt volledige reset van alle resources

## Proxmox Templates

De module kan direct werken met Proxmox VM templates:

1. Ondersteunde formaten: .tar, .tar.gz, .tar.xz
2. Plaats je template bestand in de project directory
3. Gebruik de `DockerTest::with_proxmox_template()` functie
4. De module bouwt automatisch een Docker image van de template

## Test API

De module biedt een API voor het testen van scripts in containers:

1. `DockerTest` - Voor eenmalige script-uitvoering
2. `DockerService` - Voor het uitvoeren van meerdere scripts in dezelfde container
3. `ResourceTracker` - Houdt bij welke containers en images zijn aangemaakt in een persistente database
4. Alle API's hebben timeout-ondersteuning en foutafhandeling ingebouwd