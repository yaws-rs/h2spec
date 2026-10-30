//! Barebones basic H2 Server Example over std-only TcpListener
//! This does not implement any sort of multiplexing, threading whatever for multiple clients.
//! It is designed to be quick barebone utility to just run the blueprint in the mainthread.

mod ab_io_orbit;
use ab_io_orbit::{AbIoLeft, NoRight};
mod h2app;
use h2app::MyH2App;

use blueprint::{BluePrint, Left, Orbit};
use blueprint_h2spec::{H2Serving, H2SpecServer};

use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::time::Duration;

use log::{debug, info};

use yaws_app_rick_roll::RickRoll;

// Rick Roll is a HTTP/1.1 App but we wrap it into H2Serving
fn init_orbit() -> H2Serving<MyH2App<RickRoll>> {
    H2SpecServer::with_defaults().unwrap()
}

fn handle_client(mut tcp: TcpStream) {
    let mut left = AbIoLeft::default();
    left.in_bytes = vec![0; 8192];
    left.out_bytes = vec![0; 8192];
    left.out_bytes_len = 0;

    tcp.set_nonblocking(true)
        .expect("set_nonblocking should succeed");

    struct NothingBurger;

    #[derive(Debug, Default, PartialEq)]
    struct Activity {
        io_read: bool,
        io_wrote: bool,
        bp_rw: bool,
    }

    impl Activity {
        #[inline]
        fn any(&self) -> bool {
            self.io_read || self.io_wrote || self.bp_rw
        }
    }

    let mut h2orbit = init_orbit();

    loop {
        let mut activity = Activity::default();

        if left.in_bytes_len != 8192 {
            match tcp.read(&mut left.in_bytes[left.in_bytes_len..]) {
                Ok(0) => {
                    info!("Client Disconnected.");
                    break;
                }
                Ok(r) => {
                    let end = left.in_bytes_len + r;
                    debug!("Read[{}] = {:?}", r, &left.in_bytes[left.in_bytes_len..end]);
                    left.in_bytes_len += r;
                    activity.io_read = true;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    //std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    info!("Giving up on client. Read error: {e}.");
                    break;
                }
            }
        }

        let pre_lens = left.left_lens();

        let _pos = h2orbit
            .advance_with(&mut NothingBurger, &mut left, &mut NoRight)
            .unwrap();

        let post_lens = left.left_lens();

        if pre_lens.0 != post_lens.0 || pre_lens.1 != post_lens.1 {
            activity.bp_rw = true;
        }

        if left.out_bytes_len != 0 {
            debug!(
                "Sending out[{}] = {:?}",
                left.out_bytes_len,
                &left.out_bytes[..left.out_bytes_len]
            );
            match tcp.write_all(&mut left.out_bytes[..left.out_bytes_len]) {
                Ok(()) => {
                    left.out_bytes = vec![0; 8192];
                    left.out_bytes_len = 0;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    //std::thread::sleep(Duration::from_millis(10));
                }
                Err(_e) => {
                    info!("Read error. Giving up on client.");
                    break;
                }
            }
        }

        if !activity.any() {
            std::thread::sleep(Duration::from_millis(1000));
        }
    }
}

fn main() {
    env_logger::init();

    info!("Listening for HTTP2 on 127.0.0.1:7878");
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    listener
        .set_nonblocking(true)
        .expect("set_nonblocking should succeed");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                match s.peer_addr() {
                    Ok(addr) => info!("Accepted a client from {:?}", addr),
                    Err(_) => info!("Accept a client from unknown address"),
                }
                handle_client(s);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(e) => panic!("encountered Accept IO error: {e}"),
        }
    }
}
