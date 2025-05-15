#!/bin/sh
# Startup script voor Docker container met SSH support

# Start SSH service
/usr/sbin/sshd

# Echo container info
echo "Container gestart met SSH op poort 22"
echo "Gebruikersnaam: root"
echo "Wachtwoord: alpine"

# Om ervoor te zorgen dat de container blijft draaien
if [ "$1" = "sleep" ]; then
  # Blijf draaien in de achtergrond
  echo "Container draait in de achtergrond"
  tail -f /dev/null
else
  # Voer het opgegeven commando uit of start een shell
  if [ $# -gt 0 ]; then
    exec "$@"
  else
    exec /bin/bash
  fi
fi