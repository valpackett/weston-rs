#define _GNU_SOURCE 1

/* Build the DRM compositor */
#define BUILD_DRM_COMPOSITOR 1

/* Build the headless compositor */
#define BUILD_HEADLESS_COMPOSITOR 1

/* Build the Wayland (nested) compositor */
#define BUILD_WAYLAND_COMPOSITOR 1

/* Build Weston with EGL support */
#define ENABLE_EGL 1

/* Define to 1 if you have the <execinfo.h> header file. */
#define HAVE_EXECINFO_H 1

/* gbm supports dmabuf import */
#define HAVE_GBM_FD_IMPORT 1

#define LIBWESTON_MODULEDIR ""

#ifndef __linux__
#include <stdlib.h>
#define program_invocation_short_name getprogname()
#endif
