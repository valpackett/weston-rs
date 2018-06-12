use libc;
use std::{env, mem};
use std::sync::Arc;
use std::io::Write;
use std::ffi::CStr;
use std::os::raw;
use std::os::unix::io::{RawFd, FromRawFd, AsRawFd};
use std::collections::HashMap;
use mut_static::MutStatic;
use tiny_nix_ipc::Socket;
use foreign_types::{ForeignType, ForeignTypeRef};
use ::compositor::{Compositor, CompositorRef};
use ::launcher::Launcher;

use wayland_sys::server::signal::wl_signal_emit;
use wayland_server::{sources, EventLoop};
use loginw::protocol::*;

// static closure (or custom impl) is required, so we pass the state through this global thing
lazy_static! {
    static ref LW_STATE: MutStatic<HashMap<RawFd, (Arc<Socket>, Compositor)>> = MutStatic::from(HashMap::new());
}

pub struct LoginwLauncher {
    sock: Arc<Socket>,
    // tty_fd: RawFd,
    vt_num: libc::c_int,
}

impl Launcher for LoginwLauncher {
    fn connect(compositor: &CompositorRef, event_loop: &mut EventLoop, _tty: libc::c_int, _seat_id: &CStr, _sync_drm: bool) -> Option<Self> {
        env::var("LOGINW_FD").ok().and_then(|fdstr| fdstr.parse::<RawFd>().ok()).map(|fd| {
            let mut sock = unsafe { Socket::from_raw_fd(fd) };
            let req = LoginwRequest::new(LoginwRequestType::LoginwAcquireVt);
            sock.send_struct(&req, None).unwrap();
            let (resp, _tty_fd) = sock.recv_struct::<LoginwResponse, [RawFd; 1]>().unwrap();
            assert!(resp.typ == LoginwResponseType::LoginwPassedFd);
            let sock = Arc::new(sock);

            LW_STATE.write().expect("LW_STATE write")
                .insert(fd, (Arc::clone(&sock), unsafe { Compositor::from_ptr(compositor.as_ptr()) }));
            let _ = event_loop.token().add_fd_event_source(
                fd,
                sources::FdInterest::READ,
                |ev, _| {
                    if let sources::FdEvent::Ready { fd, mask: _ } = ev {
                        let mut lw_state = LW_STATE.write().expect("state .write()");
                        let (ref sock, ref mut compositor) = lw_state.get_mut(&fd).expect("state .get_mut()");
                        let mut sock = unsafe { Socket::from_raw_fd(sock.as_raw_fd()) }; // Arc can't be mutable
                        let (resp, _) = sock.recv_struct::<LoginwResponse, [RawFd; 0]>().unwrap();
                        match resp.typ {
                            LoginwResponseType::LoginwActivated => {
                                compositor.set_session_active(true);
                                unsafe { wl_signal_emit(compositor.session_signal(), compositor.as_ptr() as *mut raw::c_void); }
                            },
                            LoginwResponseType::LoginwDeactivated => {
                                compositor.set_session_active(false);
                                unsafe { wl_signal_emit(compositor.session_signal(), compositor.as_ptr() as *mut raw::c_void); }
                            },
                            _ => (),
                        }
                    }
                }
            );

            LoginwLauncher {
                sock,
                // tty_fd: tty_fd.expect("tty_fd"),
                vt_num: unsafe { resp.dat.u64 as libc::c_int },
            }
        })
    }

    fn open(&mut self, path: &CStr, _flags: libc::c_int) -> RawFd {
        let path = path.to_str().expect("to_str");
        let typ = if path.starts_with("/dev/input") {
            LoginwRequestType::LoginwOpenInput
        } else {
            LoginwRequestType::LoginwOpenDrm
        };
        let mut req = LoginwRequest::new(typ);
        write!(unsafe { &mut req.dat.bytes[..] }, "{}", path).expect("write!()");
        let mut sock = mem::ManuallyDrop::new(unsafe { Socket::from_raw_fd(self.sock.as_raw_fd()) });
        sock.send_struct(&req, None).unwrap();
        let (resp, fd) = sock.recv_struct::<LoginwResponse, [RawFd; 1]>().unwrap();
        assert!(resp.typ == LoginwResponseType::LoginwPassedFd);
        fd.expect("fd")[0]
    }

    fn close(&mut self, fd: RawFd) {
        unsafe { libc::close(fd) };
    }

    fn activate_vt(&mut self, vt: libc::c_int) -> bool {
        let mut req = LoginwRequest::new(LoginwRequestType::LoginwSwitchVt);
        req.dat.u64 = vt as u64;
        let mut sock = unsafe { Socket::from_raw_fd(self.sock.as_raw_fd()) };
        sock.send_struct(&req, None).unwrap();
        let (resp, _) = sock.recv_struct::<LoginwResponse, [RawFd; 0]>().unwrap();
        resp.typ == LoginwResponseType::LoginwDone
    }

    fn get_vt(&mut self) -> libc::c_int {
        self.vt_num
    }
}
