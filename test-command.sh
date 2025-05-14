#!/bin/bash
HOST="127.0.0.1"
PORT="2222"

# Optionele Proxmox versie instellen (6, 7, 8)
PROXMOX_VERSION=${1:-"8"}

# Functie om een commando te versturen en het resultaat te tonen
send_command() {
    local cmd="$1"
    echo "🔧 Verstuur commando: '$cmd' naar $HOST:$PORT"
    echo "$cmd" | nc "$HOST" "$PORT"
    echo "✅ Command verzonden"
    echo "------------------------"
}

# Functie om een categorie commando's te testen
test_category() {
    local category="$1"
    echo "🧪 $category COMMANDO'S TESTEN"
    shift
    
    for cmd in "$@"; do
        send_command "$cmd"
    done
    
    echo ""
}

# Proxmox versie testen
echo "📋 PROXMOX VERSIE INFORMATIE (versie: $PROXMOX_VERSION)"
send_command "pveversion"
send_command "pveversion -v"
echo ""

# Containerbeheer (pct) commando's
test_category "CONTAINER (PCT)" \
    "pct list" \
    "pct start 100" \
    "pct stop 100" \
    "pct start 101" \
    "pct stop 101" \
    "pct create 103" \
    "pct destroy 103" \
    "pct snapshot 100 snap1" \
    "pct rollback 100 snap1" \
    "pct migrate 100 node2"

# VM-beheer (qm) commando's
test_category "VM (QM)" \
    "qm list" \
    "qm start 200" \
    "qm stop 200" \
    "qm start 201" \
    "qm stop 201" \
    "qm create 202" \
    "qm destroy 202" \
    "qm snapshot 200 snap1" \
    "qm rollback 200 snap1" \
    "qm clone 200 203" \
    "qm migrate 200 node2"

# Storage commando's
test_category "STORAGE" \
    "zfs list" \
    "zfs create rpool/test" \
    "pvesm status" \
    "pvesm alloc local-zfs 204 10G"

# Template commando's
test_category "TEMPLATE" \
    "pveam available" \
    "pveam download local ubuntu-20.04-standard_20.04-1_amd64.tar.gz" \
    "pveam list"

# Netwerk commando's
test_category "NETWERK" \
    "ifreload -a" \
    "ip addr" \
    "ip route"

# Cluster commando's
test_category "CLUSTER" \
    "pvecm status" \
    "pvecm nodes"

# Backup en restore commando's
test_category "BACKUP & RESTORE" \
    "vzdump 100" \
    "qmrestore /var/lib/vz/dump/vzdump-qemu-200.vma.gz 205" \
    "pct restore 105 /var/lib/vz/dump/vzdump-lxc-100.tar.gz"

# Onbekende commando's testen
test_category "ONBEKENDE COMMANDO'S" \
    "onbekend commando" \
    "pct foobar 100"

# Incompatibele commando's testen indien oudere versie
if [ "$PROXMOX_VERSION" = "6" ]; then
    echo "🧪 VERWACHTE INCOMPATIBELE COMMANDO'S TESTEN (V6)"
    send_command "qm clone 200 203"
    send_command "pct rollback 100 snap1"
    send_command "pvecm nodes"
fi

echo "✅ Alle tests voltooid voor Proxmox VE $PROXMOX_VERSION"
