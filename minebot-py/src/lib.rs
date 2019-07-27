#[macro_use] extern crate cpython;

use cpython::*;
use minebot;

py_module_initializer!(libminebot, initlibminebot, PyInit_libminebot, |py, m| {
    stderrlog::new()
            .verbosity(3)
            .init()
            .unwrap();

    m.add(py, "__doc__", "This module is implemented in Rust.")?;
    m.add(py, "connect", py_fn!(py, connect(host: String, port: u16, username: String)))?;
    Ok(())
});

fn connect(_py: Python, host: String, port: u16, username: String) -> PyResult<Option<i32>> {
    println!("Hello!");
    let client = minebot::MinebotClient::connect(host, port, username).unwrap();
    Ok(None)
}