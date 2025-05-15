# Docker Test Module voor Heretek

Deze module biedt een eenvoudige manier om bash scripts te testen in een Docker container met Alpine Linux. Het is ontworpen om geïsoleerde tests uit te voeren zonder de hoofdapplicatie te beïnvloeden.

## Vereisten

- Docker geïnstalleerd op je systeem
- Rust en Cargo

## Gebruik

1. Bouw de Docker image:
   ```
   cd docker_test
   docker build -t heretek-test-alpine .
   ```

2. Voer het testprogramma uit:
   ```
   cargo run
   ```

## Functionaliteit

- Uitvoeren van bash scripts in een geïsoleerde Alpine Linux omgeving
- Mounten van lokale mappen in de container
- Foutafhandeling en logging

## Integratie

Deze module is momenteel standalone en nog niet geïntegreerd met de rest van het Heretek-project. Het kan worden gebruikt als sjabloon voor toekomstige integratie.