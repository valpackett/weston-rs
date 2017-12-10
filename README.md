(Work in progress!)

# weston-rs

[Rust] bindings to `libweston` / `libweston-desktop`, the core of [Weston], the reference [Wayland] compositor.
Featuring:

- custom `build.rs` build with libweston statically linked — no autoconf, no shared library plugins
- [slightly modified libweston](https://github.com/valpackett/weston) (aside from the static linking support, it includes support for running on FreeBSD)
- nice Rusty wrappers for stuff

[Rust]: https://www.rust-lang.org
[Weston]: https://cgit.freedesktop.org/wayland/weston/
[Wayland]: https://wayland.freedesktop.org

## Current status

![Screenshot of the simple demo compositor running gtk3-demo](https://unrelentingtech.s3.dualstack.eu-west-1.amazonaws.com/weston-rs-basic-demo.png)

A simple example compositor, basically a reimplementation of [not-a-wm], works!
(Currently hardcoded to run as a Wayland client.)
See `weston-rs/examples/simple.rs`.

[not-a-wm]: https://github.com/sardemff7/not-a-wm

## Contributing

By participating in this project you agree to follow the [Contributor Code of Conduct](https://www.contributor-covenant.org/version/1/4/).

[The list of contributors is available on GitHub](https://github.com/valpackett/weston-rs/graphs/contributors).

## License

MIT License, same as Weston itself.  
For more information, please refer to the `COPYING` file.
