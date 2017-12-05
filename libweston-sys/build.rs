extern crate pkg_config;
extern crate cc;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate bindgen;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use regex::Regex;
use pkg_config::Config;

macro_rules! include_pkg {
    ($build:ident << $pkg:ident) => {
        for path in &$pkg.include_paths {
            $build.include(path);
        }
    }
}

lazy_static! {
    static ref PROTO_NAME_RE: Regex = Regex::new(r"(?P<name>[a-z\-]+)-[a-z]+-v[0-9]+").unwrap();
}

fn wayland_scan(scanner: &str, in_path: PathBuf, proto: &str) {
    Command::new(scanner)
        .args(&["client-header", in_path.to_str().expect("path"), &format!("protos/{}-client-protocol.h", proto)])
        .status().expect("wayland-scanner");
    Command::new(scanner)
        .args(&["server-header", in_path.to_str().expect("path"), &format!("protos/{}-server-protocol.h", proto)])
        .status().expect("wayland-scanner");
    Command::new(scanner)
        .args(&["code", in_path.to_str().expect("path"), &format!("protos/{}-protocol.c", proto)])
        .status().expect("wayland-scanner");
}

fn wayland_scan_pkg(scanner: &str, protos: &str, proto: &str) {
    let in_path = Path::new(protos)
        .join(if proto.contains("unstable") { "unstable" } else { "stable" })
        .join(PROTO_NAME_RE.captures(proto).map(|x| x["name"].to_owned()).unwrap_or(proto.to_owned()))
        .join(format!("{}.xml", proto));
    wayland_scan(scanner, in_path, proto)
}

fn wayland_scan_local(scanner: &str, proto: &str) {
    let in_path = Path::new("weston/protocol").join(format!("{}.xml", proto));
    wayland_scan(scanner, in_path, proto)
}

fn main() {
    let libdrm = Config::new().atleast_version("2.4.30").probe("libdrm").unwrap();
    let libudev = Config::new().atleast_version("136").probe("libudev").unwrap();
    let gbm = Config::new().atleast_version("10.2").probe("gbm").unwrap();
    let libinput = Config::new().atleast_version("0.8.0").probe("libinput").unwrap();
    let pixman = Config::new().atleast_version("0.25.2").probe("pixman-1").unwrap();
    let xkbcommon = Config::new().probe("xkbcommon").unwrap();
    let cairo = Config::new().probe("cairo").unwrap();
    let wayland_scanner = pkg_config::get_variable("wayland-scanner", "wayland_scanner").unwrap();
    let wayland_protos = pkg_config::get_variable("wayland-protocols", "pkgdatadir").unwrap();
    let wayland_server = Config::new().atleast_version("1.12.0").probe("wayland-server").unwrap();
    let wayland_client = Config::new().atleast_version("1.12.0").probe("wayland-client").unwrap();
    let wayland_cursor = Config::new().probe("wayland-cursor").unwrap();
    let wayland_egl = Config::new().probe("wayland-egl").unwrap();
    let egl = Config::new().probe("egl").unwrap();
    let gl = Config::new().probe("gl").unwrap();

    wayland_scan_pkg(&wayland_scanner, &wayland_protos, "linux-dmabuf-unstable-v1");
    wayland_scan_pkg(&wayland_scanner, &wayland_protos, "relative-pointer-unstable-v1");
    wayland_scan_pkg(&wayland_scanner, &wayland_protos, "pointer-constraints-unstable-v1");
    wayland_scan_pkg(&wayland_scanner, &wayland_protos, "text-input-unstable-v1");
    wayland_scan_pkg(&wayland_scanner, &wayland_protos, "input-method-unstable-v1");
    wayland_scan_pkg(&wayland_scanner, &wayland_protos, "fullscreen-shell-unstable-v1");
    wayland_scan_pkg(&wayland_scanner, &wayland_protos, "xdg-shell-unstable-v6");
    wayland_scan_pkg(&wayland_scanner, &wayland_protos, "xdg-shell-unstable-v5");
    wayland_scan_pkg(&wayland_scanner, &wayland_protos, "viewporter");
    wayland_scan_pkg(&wayland_scanner, &wayland_protos, "presentation-time");
    wayland_scan_local(&wayland_scanner, "text-cursor-position");

    let mut libweston_build = cc::Build::new();
    libweston_build.files(vec![
                          "weston/libweston/log.c",
                          "weston/libweston/compositor.c",
                          "weston/libweston/compositor-drm.c",
                          "weston/libweston/compositor-wayland.c",
                          "weston/libweston/compositor-x11.c",
                          "weston/libweston/compositor-headless.c",
                          "weston/libweston/input.c",
                          "weston/libweston/data-device.c",
                          "weston/libweston/screenshooter.c",
                          "weston/libweston/clipboard.c",
                          "weston/libweston/zoom.c",
                          "weston/libweston/bindings.c",
                          "weston/libweston/animation.c",
                          "weston/libweston/noop-renderer.c",
                          "weston/libweston/pixman-renderer.c",
                          "weston/libweston/gl-renderer.c",
                          "weston/libweston/vertex-clipping.c",
                          "weston/libweston/plugin-registry.c",
                          "weston/libweston/timeline.c",
                          "weston/libweston/linux-dmabuf.c",
                          "weston/libweston/pixel-formats.c",
                          "weston/shared/matrix.c",
                          "weston/shared/file-util.c",
                          "weston/shared/cairo-util.c",
                          "weston/shared/os-compatibility.c",
                          "weston/shared/frame.c",
                          "protos/linux-dmabuf-unstable-v1-protocol.c",
                          "protos/relative-pointer-unstable-v1-protocol.c",
                          "protos/pointer-constraints-unstable-v1-protocol.c",
                          "protos/text-input-unstable-v1-protocol.c",
                          "protos/input-method-unstable-v1-protocol.c",
                          "protos/fullscreen-shell-unstable-v1-protocol.c",
                          "protos/viewporter-protocol.c",
                          "protos/presentation-time-protocol.c",
                          "protos/text-cursor-position-protocol.c",
    ]);
    libweston_build.include("config").include("protos").include("weston/shared").include("weston")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-shift-negative-value")
        .flag_if_supported("-Wno-missing-field-initializers")
        .flag_if_supported("-fvisibility=hidden");
    include_pkg!(libweston_build << libdrm);
    include_pkg!(libweston_build << libudev);
    include_pkg!(libweston_build << gbm);
    include_pkg!(libweston_build << libinput);
    include_pkg!(libweston_build << pixman);
    include_pkg!(libweston_build << xkbcommon);
    include_pkg!(libweston_build << cairo);
    include_pkg!(libweston_build << wayland_server);
    include_pkg!(libweston_build << wayland_client);
    include_pkg!(libweston_build << wayland_cursor);
    include_pkg!(libweston_build << wayland_egl);
    include_pkg!(libweston_build << egl);
    include_pkg!(libweston_build << gl);
    if cfg!(not(target_os="linux")) {
        libweston_build.include("/usr/local/include/libepoll-shim");
        println!("cargo:rustc-link-lib=dylib=epoll-shim");
    }
    libweston_build.compile("libweston.a");

    let mut libweston_desktop_build = cc::Build::new();
    libweston_desktop_build.files(vec![
                                  "weston/libweston-desktop/client.c",
                                  "weston/libweston-desktop/libweston-desktop.c",
                                  "weston/libweston-desktop/seat.c",
                                  "weston/libweston-desktop/surface.c",
                                  "weston/libweston-desktop/wl-shell.c",
                                  "weston/libweston-desktop/xdg-shell-v6.c",
                                  "weston/libweston-desktop/xdg-shell-v5.c",
                                  "weston/libweston-desktop/xwayland.c",
                                  "protos/xdg-shell-unstable-v5-protocol.c",
                                  "protos/xdg-shell-unstable-v6-protocol.c",
    ]);
    libweston_desktop_build.include("config").include("protos").include("weston/shared").include("weston/libweston").include("weston")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-shift-negative-value")
        .flag_if_supported("-Wno-missing-field-initializers")
        .flag_if_supported("-fvisibility=hidden");
    include_pkg!(libweston_desktop_build << pixman);
    include_pkg!(libweston_desktop_build << wayland_server);
    libweston_desktop_build.compile("libweston-desktop.a");

    bindgen::Builder::default()
        .header("wrapper.h")
        .blacklist_type(r"^wl_.*$")
        .whitelist_type(r"^weston_.*$")
        .whitelist_function(r"^weston_.*$")
        .ctypes_prefix("libc")
        .clang_args(&["-Iconfig", "-Iprotos", "-Iweston/shared", "-Iweston/libweston", "-Iweston"])
        .clang_args(&[libdrm, libudev, gbm, libinput, pixman, xkbcommon, wayland_server, wayland_client, wayland_cursor, wayland_egl]
                    .iter().flat_map(|p| &p.include_paths).map(|p| format!("-I{}", p.to_str().unwrap())).collect::<Vec<_>>())
        .generate()
        .expect("bindgen")
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("write_to_file");
}
