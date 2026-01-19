#!/bin/bash
# Script to regenerate prebuilt bindings for all supported GDAL versions.
# Run from host machine (outside Docker). Each GDAL version uses its corresponding Docker image.
#
# Usage:
#   ./generate_all_bindings.sh           # Generate bindings for all versions
#   ./generate_all_bindings.sh 3_11      # Generate bindings for a specific version
#
# GDAL Docker images and their Ubuntu bases:
#   GDAL 3.4.x  -> Ubuntu 20.04 (mingw gcc 9)
#   GDAL 3.5.x  -> Ubuntu 20.04 (mingw gcc 9)
#   GDAL 3.6.x  -> Ubuntu 22.04 (mingw gcc 10)
#   GDAL 3.7.x  -> Ubuntu 22.04 (mingw gcc 10)
#   GDAL 3.8.x  -> Ubuntu 22.04 (mingw gcc 10)
#   GDAL 3.9.x  -> Ubuntu 24.04 (mingw gcc 13)
#   GDAL 3.10.x -> Ubuntu 24.04 (mingw gcc 13)
#   GDAL 3.11.x -> Ubuntu 24.04 (mingw gcc 13)
#   GDAL 3.12.x -> Ubuntu 24.04 (mingw gcc 13)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GDAL_SYS_DIR="$(dirname "$SCRIPT_DIR")"

# Function to get Docker image tag for a binding version folder
get_docker_tag() {
    case "$1" in
        3_4)  echo "3.4.0" ;;
        3_5)  echo "3.5.0" ;;
        3_6)  echo "3.6.0" ;;
        3_7)  echo "3.7.0" ;;
        3_8)  echo "3.8.0" ;;
        3_9)  echo "3.9.0" ;;
        3_10) echo "3.10.0" ;;
        3_11) echo "3.11.0" ;;
        3_12) echo "3.12.0" ;;
        *)    echo "" ;;
    esac
}

generate_bindings() {
    local version_folder="$1"
    local docker_tag
    docker_tag=$(get_docker_tag "$version_folder")
    
    if [ -z "$docker_tag" ]; then
        echo "Error: Unknown version '$version_folder'"
        echo "Available versions: 3_4 3_5 3_6 3_7 3_8 3_9 3_10 3_11 3_12"
        return 1
    fi
    
    local image="ghcr.io/osgeo/gdal:ubuntu-full-$docker_tag"
    
    echo "========================================"
    echo "Generating bindings for GDAL $docker_tag (folder: $version_folder)"
    echo "Using image: $image"
    echo "========================================"
    
    # Force x86_64 platform to ensure consistent struct layouts (e.g., struct stat)
    # across different host architectures. Without this, ARM64 Macs would generate
    # bindings with ARM64 glibc struct layouts, which would be incorrect for x86_64 Linux.
    docker run --platform linux/amd64 --rm \
        -v "$GDAL_SYS_DIR:/gdal-sys" \
        -e GDAL_VERSION="$version_folder" \
        "$image" \
        bash /gdal-sys/scripts/generate_bindings.sh
    
    echo "Completed bindings for GDAL $docker_tag"
    echo ""
}

# If a specific version is provided, generate only that one
if [ -n "$1" ]; then
    generate_bindings "$1"
else
    # Generate bindings for all versions (sorted)
    echo "Generating prebuilt bindings for all GDAL versions..."
    echo ""
    for version_folder in 3_4 3_5 3_6 3_7 3_8 3_9 3_10 3_11 3_12; do
        generate_bindings "$version_folder"
    done
    
    echo "========================================"
    echo "All bindings generated successfully!"
    echo "========================================"
fi
