set -e

if [ -z CI_USE_GDAL_VERSION ]; then
    echo "Error: did not specify which version of GDAL to test. Please specify with CI_USE_GDAL_VERSION"
    exit 1
fi

case $CI_USE_GDAL_VERSION in
    2)
        if [ $TRAVIS_OS_NAME = 'osx' ]; then
            # gdal is already installed on default macos image (v2.4.2_3)
            brew info gdal
        else
            sudo add-apt-repository ppa:ubuntugis/ppa -y
            sudo apt-get update -q
            sudo apt-get install -y libgdal-dev libgdal20
        fi
        ;;
    3)
        if [ $TRAVIS_OS_NAME = 'osx' ]; then
            brew upgrade gdal
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


NEXT_VERSION=$((CI_USE_GDAL_VERSION + 1));
pkg-config --exists "gdal >= ${CI_USE_GDAL_VERSION}; gdal < ${NEXT_VERSION}" --print-errors


if pkg-config --exists "gdal >= ${CI_USE_GDAL_VERSION}; gdal < ${NEXT_VERSION}" 
then
  echo "Correct version of GDAL found"
else
  echo "CI sanity check failed: pkg-config could not find specified version of GDAL. CI_USE_GDAL_VERSION: ${CI_USE_GDAL_VERSION}"
  exit 1
fi
