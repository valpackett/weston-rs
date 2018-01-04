use libc;
use std::env;
use std::io::Write;
use std::ffi::CStr;
use std::os::raw;
use std::os::unix::io::RawFd;
use ::WestonObject;
use ::compositor::Compositor;
use ::launcher::Launcher;

use wayland_sys::server::signal::wl_signal_emit;
use wayland_server::sources;
use loginw::protocol::*;
use loginw::socket::*;

pub struct LoginwLauncher {
    sock: Socket,
    tty_fd: RawFd,
    vt_num: libc::c_int,
}

impl Launcher for LoginwLauncher {
    fn connect(compositor: &Compositor, _tty: libc::c_int, _seat_id: &CStr, _sync_drm: bool) -> Option<Self> {
        env::var("LOGINW_FD").ok().and_then(|fdstr| fdstr.parse::<RawFd>().ok()).map(|fd| {
            let sock = Socket::new(fd);
            let mut req = LoginwRequest::new(LoginwRequestType::LoginwAcquireVt);
            sock.sendmsg(&req, None).expect(".sendmsg()");
            let (resp, tty_fd) = sock.recvmsg::<LoginwResponse>().expect(".recvmsg()");
            assert!(resp.typ == LoginwResponseType::LoginwPassedFd);

            compositor.get_display().get_event_loop().add_fd_event_source(
                sock.fd,
                sources::FdEventSourceImpl {
                    ready: |_, &mut (ref sock, ref compositor): &mut (Socket, Compositor), fd, _| {
                        let (resp, _) = sock.recvmsg::<LoginwResponse>().expect(".recvmsg()");
                        match resp.typ {
                            LoginwResponseType::LoginwActivated => {
                                compositor.set_session_active(true);
                                unsafe { wl_signal_emit(compositor.session_signal(), compositor.ptr() as *mut raw::c_void); }
                            },
                            LoginwResponseType::LoginwDeactivated => {
                                compositor.set_session_active(false);
                                unsafe { wl_signal_emit(compositor.session_signal(), compositor.ptr() as *mut raw::c_void); }
                            },
                            _ => {
                            },
                        }
                    },
                    error: |_, _, fd, _| {
                        // TODO: restore the tty
                    },
                },
                (sock.temp_clone(), compositor.temp_clone()),
                sources::FdInterest::READ,
            );

            LoginwLauncher {
                sock,
                tty_fd: tty_fd.expect("tty_fd"),
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

    fn restore(&mut self) {
    }

    fn get_vt(&mut self) -> libc::c_int {
        self.vt_num
    }
}
