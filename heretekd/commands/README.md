# Proxmox Simulatie Commando's

Deze map bevat definities van commando's die gebruikt worden in de Proxmox-simulatieomgeving. De commando's zijn georganiseerd in verschillende structuren die specifieke versies en functionaliteit reflecteren.

## Mapstructuur

De simulatie ondersteunt Proxmox VE versies 6, 7 en 8. De commando's zijn als volgt georganiseerd:

```
commands/
├── common/        # Commando's gemeenschappelijk voor alle versies
├── v6/            # Proxmox VE 6.x specifieke commando's
├── v7/            # Proxmox VE 7.x specifieke commando's
└── v8/            # Proxmox VE 8.x specifieke commando's
```

## Commando Categorieën

In elke map zijn commando's gegroepeerd per categorie in verschillende YAML-bestanden:

- **container.yaml**: LXC container-gerelateerde commando's (pct)
- **storage.yaml**: Opslag-gerelateerde commando's (zfs, pvesm)
- **system.yaml**: Systeem-gerelateerde commando's (netwerk, cluster)
- **version.yaml**: Versie-specifieke informatie (pveversion)
- **vm.yaml**: VM-gerelateerde commando's (qm)

## Gebruik

Bij het uitvoeren van een commando zoekt het systeem eerst naar een exacte match in de versie-specifieke map. Als het commando daar niet wordt gevonden, wordt gezocht in de `common`-map voor commando's die voor alle versies gelden.

Wanneer nieuwe functionaliteit alleen in een specifieke Proxmox-versie beschikbaar is, moet deze in de betreffende versie-map worden toegevoegd.

## Formaat

Elk YAML-bestand gebruikt het volgende format:

```yaml
# Beschrijving van de commando categorie

"commando string": |
  output regel 1
  output regel 2
  ...

"ander commando": |
  andere output
  ...
```

## Versies Bijwerken

Bij het toevoegen van ondersteuning voor een nieuwe Proxmox-versie:
1. Maak een nieuwe map (bijvoorbeeld `v9/`)
2. Kopieer relevante YAML-bestanden uit de vorige versie
3. Update/voeg versie-specifieke commando's toe
4. Update eventueel de shared commando's in de `common`-map