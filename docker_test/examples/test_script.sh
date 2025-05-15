#!/bin/bash
# test_script.sh - Voorbeeld script voor testen met Docker
set -e

# Controleer of we in een container draaien
if [ -f "/.dockerenv" ]; then
    echo "Dit script draait in een Docker container."
else
    echo "Dit script draait niet in een Docker container."
fi

# Toon systeem informatie
echo "-- Systeem Informatie --"
uname -a
cat /etc/os-release

# Test commando's
echo "-- Test Commando's --"
echo "Huidige datum: $(date)"
echo "Mapstructuur:"
ls -la

# Test bestandsbewerking
echo "-- Bestandsbewerking Test --"
TESTFILE="/tmp/test_file.txt"
echo "Dit is een test" > "$TESTFILE"
echo "Bestand aangemaakt: $TESTFILE"
cat "$TESTFILE"

# Test environment variabelen
echo "-- Environment Variabelen --"
if [ ! -z "$TEST_VAR" ]; then
    echo "TEST_VAR is ingesteld op: $TEST_VAR"
else
    echo "TEST_VAR is niet ingesteld."
fi

echo "Test succesvol afgerond."
exit 0