/**
 *  includes as in gdal/autotest/cpp/test_include_from_c_file.c
 */

#include "cpl_atomic_ops.h"
#include "cpl_conv.h"
#include "cpl_csv.h"
#include "cpl_error.h"
#include "cpl_hash_set.h"
#include "cpl_list.h"
#include "cpl_minixml.h"
#include "cpl_port.h"
#include "cpl_progress.h"
#include "cpl_quad_tree.h"
#include "cpl_vsi.h"
#include "gdal_alg.h"
#include "gdal_version.h"
#include "gdal.h"
#include "gdal_utils.h"
#include "ogr_api.h"
#include "ogr_core.h"
#include "ogr_srs_api.h"

/**
 *  extra includes
 */
#include "gdalwarper.h"
#include "gdal_vrt.h"

/**
 * Include ArrowArrayStream header for recent GDAL versions
 */
#if GDAL_VERSION_NUM >= GDAL_COMPUTE_VERSION(3,6,0)
    #include "ogr_recordbatch.h"
#endif

/**
 * Type for a OGR error
 *
 * <div rustbindgen replaces="OGRErr"></div>
 */
typedef enum
{
    /**
     * Success
     *
     * <div rustbindgen replaces="OGRERR_NONE"></div>
     */
    STRICT_OGRERR_NONE,
    /**
     * Not enough data to deserialize
     *
     * <div rustbindgen replaces="OGRERR_NOT_ENOUGH_DATA"></div>
     */
    STRICT_OGRERR_NOT_ENOUGH_DATA,
    /**
     * Not enough memory
     *
     * <div rustbindgen replaces="OGRERR_NOT_ENOUGH_MEMORY"></div>
     */
    STRICT_OGRERR_NOT_ENOUGH_MEMORY,
    /**
     * Unsupported geometry type
     *
     * <div rustbindgen replaces="OGRERR_UNSUPPORTED_GEOMETRY_TYPE"></div>
     */
    STRICT_OGRERR_UNSUPPORTED_GEOMETRY_TYPE,
    /**
     * Unsupported operation
     *
     * <div rustbindgen replaces="OGRERR_UNSUPPORTED_OPERATION"></div>
     */
    STRICT_OGRERR_UNSUPPORTED_OPERATION,
    /**
     * Corrupt data
     *
     * <div rustbindgen replaces="OGRERR_CORRUPT_DATA"></div>
     */
    STRICT_OGRERR_CORRUPT_DATA,
    /**
     * Failure
     *
     * <div rustbindgen replaces="OGRERR_FAILURE"></div>
     */
    STRICT_OGRERR_FAILURE,
    /**
     * Unsupported SRS
     *
     * <div rustbindgen replaces="OGRERR_UNSUPPORTED_SRS"></div>
     */
    STRICT_OGRERR_UNSUPPORTED_SRS,
    /**
     * Invalid handle
     *
     * <div rustbindgen replaces="INVALID_HANDLE"></div>
     */
    STRICT_OGRERR_INVALID_HANDLE,
#if GDAL_VERSION_NUM >= GDAL_COMPUTE_VERSION(2,0,0)
    /**
     * Non existing feature. Added in GDAL 2.0
     *
     * <div rustbindgen replaces="NON_EXISTING_FEATURE"></div>
     */
    STRICT_OGRERR_NON_EXISTING_FEATURE,
#endif
} StrictOGRErr;
