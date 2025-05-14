#!/bin/bash
# Test script om de prestaties van de heretekd server te valideren

HOST="127.0.0.1"
PORT="2222"
NUM_REQUESTS=10  # Aantal verzoeken voor de test

# Functie om de tijd te meten voor een commando
measure_time() {
    local cmd="$1"
    echo "$cmd" > /tmp/heretek-test-cmd
    
    # Meet de tijd die nodig is om het commando te versturen en antwoord te krijgen
    start_time=$(date +%s.%N)
    cat /tmp/heretek-test-cmd | nc $HOST $PORT > /dev/null
    end_time=$(date +%s.%N)
    
    # Bereken het verschil
    execution_time=$(echo "$end_time - $start_time" | bc)
    echo "$execution_time seconden voor: '$cmd'"
}

# Verzend meerdere gelijksoortige verzoeken en meet de tijd
test_multiple_requests() {
    local name="$1"
    local cmd="$2"
    
    echo "🔍 Test: $name (met $NUM_REQUESTS verzoeken)"
    
    # Eerste verzoek - mogelijk langzaam vanwege configuratie laden
    echo "Eerste verzoek (initialisatie):"
    measure_time "$cmd"
    
    echo "Vervolgreeks ($NUM_REQUESTS verzoeken):"
    total_time=0
    
    for i in $(seq 1 $NUM_REQUESTS); do
        time=$(measure_time "$cmd" | cut -d' ' -f1)
        total_time=$(echo "$total_time + $time" | bc)
    done
    
    avg_time=$(echo "scale=6; $total_time / $NUM_REQUESTS" | bc)
    echo "Gemiddelde uitvoertijd: $avg_time seconden"
    echo "------------------------"
}

# Check of de server draait
check_server() {
    echo "pct list" | nc -z -w 1 $HOST $PORT 2>/dev/null
    
    if [ $? -ne 0 ]; then
        echo "⚠️ De heretekd server lijkt niet te draaien op $HOST:$PORT"
        echo "Start de server eerst met: cargo run -p heretekd"
        exit 1
    fi
    
    echo "✅ Verbinding met heretekd server op $HOST:$PORT is succesvol"
    echo "------------------------"
}

# Begin van het testscript
echo "🚀 HERETEK SERVER PRESTATIE TEST"
echo "------------------------"

# Controleer of de server draait
check_server

# Test verschillende commando's
test_multiple_requests "Container listing" "pct list"
test_multiple_requests "VM listing" "qm list"
test_multiple_requests "Storage listing" "zfs list"
test_multiple_requests "Version info" "pveversion"
test_multiple_requests "Container actie" "pct start 100"
test_multiple_requests "Onbekend commando" "onbekend commando"

echo "✅ Alle tests voltooid"
echo "------------------------"
echo "Opmerking: Het eerste verzoek kan langzamer zijn vanwege initialisatie."
echo "Vervolgverzoeken zouden significant sneller moeten zijn dankzij de caching."