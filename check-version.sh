#!/bin/bash

# check-version.sh - Controleert de geconfigureerde Proxmox-versie in Heretek

CONFIG_FILE="config.toml"
DEFAULT_VERSION="8"

# ANSI kleuren
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Heretek Proxmox Version Checker${NC}"
echo "-----------------------------"
echo "Configuratiebestand: $CONFIG_FILE"

# Controleer of het configuratiebestand bestaat
if [ ! -f "$CONFIG_FILE" ]; then
    echo -e "${YELLOW}Waarschuwing:${NC} Configuratiebestand '$CONFIG_FILE' niet gevonden."
    echo -e "Standaard versie: ${GREEN}Proxmox VE $DEFAULT_VERSION${NC}"
    exit 0
fi

# Controleer of de proxmox sectie bestaat
if ! grep -q "^\[proxmox\]" "$CONFIG_FILE"; then
    echo -e "${YELLOW}Waarschuwing:${NC} Geen [proxmox] sectie gevonden in configuratiebestand."
    echo -e "Standaard versie: ${GREEN}Proxmox VE $DEFAULT_VERSION${NC}"
    exit 0
fi

# Lees de versie uit het configuratiebestand
VERSION=$(awk '/^\[proxmox\]/{flag=1; next} /^\[.*\]/{flag=0} flag && /version *=/{gsub(/[^0-9]/, "", $3); print $3; exit}' "$CONFIG_FILE")

if [ -z "$VERSION" ]; then
    echo -e "${YELLOW}Waarschuwing:${NC} Geen versie gevonden in configuratiebestand."
    echo -e "Standaard versie: ${GREEN}Proxmox VE $DEFAULT_VERSION${NC}"
else
    if [ "$VERSION" = "6" ] || [ "$VERSION" = "7" ] || [ "$VERSION" = "8" ]; then
        echo -e "Geconfigureerde versie: ${GREEN}Proxmox VE $VERSION${NC}"
    else
        echo -e "${RED}Fout:${NC} Ongeldige versie '$VERSION'. Moet 6, 7 of 8 zijn."
        echo -e "Standaard versie: ${GREEN}Proxmox VE $DEFAULT_VERSION${NC}"
    fi
fi

# Toon informatie over versieverschillen
echo ""
echo -e "${BLUE}Versie-informatie:${NC}"
echo "- Proxmox VE 6: Oudere versie met beperkte ondersteuning"
echo "- Proxmox VE 7: Verbeterde functionaliteit en ondersteuning"
echo "- Proxmox VE 8: Nieuwste versie met volledige functionaliteit"

echo ""
echo "Om de versie te wijzigen, pas de 'version' waarde aan in $CONFIG_FILE:"
echo -e "${YELLOW}[proxmox]${NC}"
echo -e "${YELLOW}version = \"7\"${NC} <- wijzig dit nummer naar 6, 7 of 8"