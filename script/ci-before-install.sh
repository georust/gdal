#!/usr/bin/env bash
set -e

if [ -z $CI_USE_GDAL_VERSION ]; then
    echo "Error: CI_USE_GDAL_VERSION was not set."
    exit 1
fi

if [ -z $TRAVIS_OS_NAME ]; then
    echo "Error: TRAVIS_OS_NAME was not set."
    exit 1
fi

function gdal_is_installed {
    GDAL_VERSION=$1
    NEXT_VERSION=$(python -c "import math; print(math.floor(${GDAL_VERSION} + 1))")
    if pkg-config --exists "gdal >= ${GDAL_VERSION}; gdal < ${NEXT_VERSION}" --print-errors 
    then
        echo "Correct version of GDAL found"
        return 0
    else
        echo "CI sanity check failed: pkg-config could not find specified version of GDAL. CI_USE_GDAL_VERSION: ${CI_USE_GDAL_VERSION}"
        return 1
    fi
}

if gdal_is_installed $CI_USE_GDAL_VERSION
then
    echo "Already installed. CI_USE_GDAL_VERSION: ${CI_USE_GDAL_VERSION}"
else
    case $CI_USE_GDAL_VERSION in
        2)
            if [ $TRAVIS_OS_NAME = 'osx' ]; then
                # gdal 2.4.2 is already installed on travis's default osx image
                brew info gdal
            else
                sudo add-apt-repository ppa:ubuntugis/ppa -y
                sudo apt-get update -q
                sudo apt-get install -y libgdal-dev libgdal20
            fi
            ;;
        3.0|3.1)
            if [ $TRAVIS_OS_NAME = 'osx' ]; then
                # gdal 3.1.2 is already installed on travis's "xcode12" osx_image
                brew info gdal
            else
                sudo add-apt-repository ppa:ubuntugis/ubuntugis-unstable -y
                sudo apt-get update -q
                # sudo apt-get install -y gdal # not found
                sudo apt-get install -y libgdal-dev
            fi
            ;;
        *)
            echo "Unknown gdal version specified. CI_USE_GDAL_VERSION: ${CI_USE_GDAL_VERSION}"
            exit 1
            ;;
    esac

    # Sanity Check
    if gdal_is_installed $CI_USE_GDAL_VERSION
    then
        echo "Successfully installed. CI_USE_GDAL_VERSION: ${CI_USE_GDAL_VERSION}"
    else
        echo "Failed to install. CI_USE_GDAL_VERSION: ${CI_USE_GDAL_VERSION}"
        exit 1
    fi
fi

