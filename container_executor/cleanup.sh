#!/bin/bash
# Cleanup script voor Heretek container_executor

# Bepaal de data directory
HOME_DIR="${HOME:-/tmp}"
DATA_DIR="${HOME_DIR}/.heretek"
RESOURCE_DB="${DATA_DIR}/resources.db"

echo "🧹 Cleaning up all Docker resources created by container_executor..."

# Check of sled database bestaat
if [ ! -d "$RESOURCE_DB" ]; then
    echo "Database niet gevonden op ${RESOURCE_DB}"
    echo "Forceer verwijdering van alle containers die beginnen met 'heretek-'"
    docker ps -a --filter "name=heretek-" -q | xargs -r docker rm -f
    docker images "heretek-*" -q | xargs -r docker rmi -f
    exit 0
fi

# Verwijder database en laat Docker opschonen
echo "Verwijder alle containers in tracker..."
for container in $(docker ps -a -q); do
    echo "Stopping and removing container: $container"
    docker stop $container 2>/dev/null || true
    docker rm -f $container 2>/dev/null || true
done

echo "Verwijder alle images..."
for image in $(docker images "heretek-*" -q); do
    echo "Removing image: $image"
    docker rmi -f $image 2>/dev/null || true
done

# Verwijder de database na cleanup
echo "Verwijder database op ${RESOURCE_DB}"
rm -rf "$RESOURCE_DB"

echo "✅ Cleanup voltooid!"
echo ""
echo "Om de container_executor opnieuw te initialiseren, voer uit:"
echo "  cargo run -- install alpine"