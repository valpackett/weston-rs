use libc;
use std::env;
use std::rc::Rc;
use std::io::Write;
use std::ffi::CStr;
use std::os::raw;
use std::os::unix::io::RawFd;
use foreign_types::ForeignTypeRef;
use ::compositor::CompositorRef;
use ::launcher::Launcher;

use wayland_sys::server::signal::wl_signal_emit;
use wayland_server::{sources, EventLoopHandle};
use loginw::protocol::*;
use loginw::socket::*;

pub struct LoginwLauncher {
    sock: Rc<Socket>,
    // tty_fd: RawFd,
    vt_num: libc::c_int,
}

impl Launcher for LoginwLauncher {
    fn connect(compositor: &CompositorRef, _tty: libc::c_int, _seat_id: &CStr, _sync_drm: bool) -> Option<Self> {
        env::var("LOGINW_FD").ok().and_then(|fdstr| fdstr.parse::<RawFd>().ok()).map(|fd| {
            let sock = Rc::new(Socket::new(fd));
            let req = LoginwRequest::new(LoginwRequestType::LoginwAcquireVt);
            sock.sendmsg(&req, None).expect(".sendmsg()");
            let (resp, _tty_fd) = sock.recvmsg::<LoginwResponse>().expect(".recvmsg()");
            assert!(resp.typ == LoginwResponseType::LoginwPassedFd);

            let _ = compositor.get_display().get_event_loop().add_fd_event_source(
                sock.fd,
                sources::FdEventSourceImpl {
                    ready: |_: &mut EventLoopHandle, &mut (ref sock, ref mut compositor): &mut (Rc<Socket>, &mut CompositorRef), _fd, _| {
                        let (resp, _) = sock.recvmsg::<LoginwResponse>().expect(".recvmsg()");
                        match resp.typ {
                            LoginwResponseType::LoginwActivated => {
                                compositor.set_session_active(true);
                                unsafe { wl_signal_emit(compositor.session_signal(), compositor.as_ptr() as *mut raw::c_void); }
                            },
                            LoginwResponseType::LoginwDeactivated => {
                                compositor.set_session_active(false);
                                unsafe { wl_signal_emit(compositor.session_signal(), compositor.as_ptr() as *mut raw::c_void); }
                            },
                            _ => {
                            },
                        }
                    },
                    error: |_: &mut EventLoopHandle, _, _fd, _| {
                        // TODO: restore the tty
                    },
                },
                (Rc::clone(&sock), unsafe { CompositorRef::from_ptr_mut(compositor.as_ptr()) }),
                sources::FdInterest::READ,
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
        self.sock.sendmsg(&req, None).expect(".sendmsg()");
        let (resp, fd) = self.sock.recvmsg::<LoginwResponse>().expect(".recvmsg()");
        assert!(resp.typ == LoginwResponseType::LoginwPassedFd);
        fd.expect("fd")
    }

    fn close(&mut self, fd: RawFd) {
        unsafe { libc::close(fd) };
    }

    fn activate_vt(&mut self, vt: libc::c_int) -> bool {
        let mut req = LoginwRequest::new(LoginwRequestType::LoginwSwitchVt);
        req.dat.u64 = vt as u64;
        self.sock.sendmsg(&req, None).expect(".sendmsg()");
        let (resp, _) = self.sock.recvmsg::<LoginwResponse>().expect(".recvmsg()");
        resp.typ == LoginwResponseType::LoginwDone
    }

    fn get_vt(&mut self) -> libc::c_int {
        self.vt_num
    }
}
