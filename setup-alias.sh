#!/bin/bash
# setup-alias.sh - Voegt alias toe aan bash/zsh configuratie voor Heretek

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BASH_PROFILE="$HOME/.bash_profile"
BASHRC="$HOME/.bashrc"
ZSHRC="$HOME/.zshrc"

# ANSI kleuren
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Alias definities
HERETEK_VERSION_ALIAS="alias heretek-version='$SCRIPT_DIR/check-version.sh'"
HERETEK_START_ALIAS="alias heretek-start='cd $SCRIPT_DIR && cargo run -p heretekd'"

echo -e "${BLUE}Heretek Alias Setup${NC}"
echo "--------------------"

# Functie om alias toe te voegen aan configuratiebestand
add_alias_to_file() {
    local file=$1
    local alias_line=$2
    local description=$3
    
    if [ -f "$file" ]; then
        if grep -q "$alias_line" "$file"; then
            echo -e "${YELLOW}Info:${NC} $description is al aanwezig in $file"
        else
            echo "$alias_line" >> "$file"
            echo -e "${GREEN}Succes:${NC} $description toegevoegd aan $file"
        fi
    fi
}

# Detecteer de shell en voeg aliases toe aan het juiste configuratiebestand
CURRENT_SHELL=$(basename "$SHELL")

if [ "$CURRENT_SHELL" = "zsh" ]; then
    echo "Zsh shell gedetecteerd"
    add_alias_to_file "$ZSHRC" "$HERETEK_VERSION_ALIAS" "Heretek versie alias"
    add_alias_to_file "$ZSHRC" "$HERETEK_START_ALIAS" "Heretek start alias"
    CONFIG_FILE="$ZSHRC"
else
    echo "Bash shell gedetecteerd"
    # Probeer eerst .bash_profile, daarna .bashrc
    if [ -f "$BASH_PROFILE" ]; then
        add_alias_to_file "$BASH_PROFILE" "$HERETEK_VERSION_ALIAS" "Heretek versie alias"
        add_alias_to_file "$BASH_PROFILE" "$HERETEK_START_ALIAS" "Heretek start alias"
        CONFIG_FILE="$BASH_PROFILE"
    else
        add_alias_to_file "$BASHRC" "$HERETEK_VERSION_ALIAS" "Heretek versie alias"
        add_alias_to_file "$BASHRC" "$HERETEK_START_ALIAS" "Heretek start alias"
        CONFIG_FILE="$BASHRC"
    fi
fi

echo ""
echo -e "${BLUE}Beschikbare commando's na herladen van shell configuratie:${NC}"
echo "- heretek-version: Toont de huidige Proxmox versie die wordt gesimuleerd"
echo "- heretek-start: Start de heretekd server"
echo ""
echo -e "Voer het volgende commando uit om de aliases direct te laden:"
echo -e "${YELLOW}source $CONFIG_FILE${NC}"