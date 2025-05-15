#!/bin/bash
# Helper script for testing container executor functionality

# Set default values
TEMPLATE="alpine"
VERSION="3.12"
SCRIPT_FILE=""
MOUNT_PATH=""
TIMEOUT=30

# Function to show usage
show_help() {
    echo "Usage: $0 [options] script_file"
    echo "Options:"
    echo "  -t, --template   Template to use (alpine|debian) [default: alpine]"
    echo "  -v, --version    Version of template [default: 3.12]"
    echo "  -m, --mount      Directory to mount in container"
    echo "  --timeout        Script execution timeout in seconds [default: 30]"
    echo "  -h, --help       Show this help message"
    exit 1
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--template)
            TEMPLATE="$2"
            shift 2
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -m|--mount)
            MOUNT_PATH="$2"
            shift 2
            ;;
        --timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            ;;
        *)
            SCRIPT_FILE="$1"
            shift
            ;;
    esac
done

# Check if script file is provided
if [ -z "$SCRIPT_FILE" ]; then
    echo "Error: No script file provided"
    show_help
fi

# Check if script file exists
if [ ! -f "$SCRIPT_FILE" ]; then
    echo "Error: Script file '$SCRIPT_FILE' not found"
    exit 1
fi

# Build mount option
MOUNT_OPT=""
if [ ! -z "$MOUNT_PATH" ]; then
    MOUNT_OPT="--mount $MOUNT_PATH"
fi

# Ensure template is installed
echo "Ensuring $TEMPLATE version $VERSION is installed..."
cargo run -- install $TEMPLATE $VERSION

# Run the script
echo "Running script: $SCRIPT_FILE"
echo "Template: $TEMPLATE $VERSION"
[ ! -z "$MOUNT_PATH" ] && echo "Mount: $MOUNT_PATH"
echo "Timeout: ${TIMEOUT}s"
echo "---"

cargo run -- run "$SCRIPT_FILE" \
    --template $TEMPLATE \
    --version $VERSION \
    --timeout $TIMEOUT \
    $MOUNT_OPT

exit_code=$?
echo "---"
echo "Exit code: $exit_code"
exit $exit_code