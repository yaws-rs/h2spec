//! Rick Roll as a Test (RRaaT)
//!
//! Because the whole idea of Rick Roll is to get everyone Rick'd anywhere and everywhere we should be able to test it!
//!
//! Given the Rick is available to us in it's pure form (GIF) we can just check we received his pure form everywhere and any time..
//!
//! We are also testing the intermediate HTTP protocol functionality so that it does not compromise the pureness of getting Rick'd.

// This should reflect the size of the Rick we've helped ourselves to from the internet

use crate::AbIoLeft;
use blueprint::NoRight;
use blueprint::{BluePrint, Orbit};
//use blueprint_h11spec::H11Serving;
//use blueprint_h11spec::H11SpecServer;
//use yaws_app_rick_roll::RickRoll;

use blueprint_h2spec::H2Serving;
use blueprint_h2spec::H2SpecServer;
use blueprint_h2spec::Reset;

#[derive(Default)]
struct RickRoll;
impl Reset for RickRoll {
    fn reset(&mut self) {}
}

// Normally you would use a runtime to inject through the composition but we can just roll with it like this to limit the test surface to
// only test both the HTTP and Application layers without things like I/O and TLS.
fn init_orbit() -> H2Serving<RickRoll> {
    H2SpecServer::with_defaults().unwrap()
}

struct NothingBurger;

#[test]
fn lets_get_rickd() {
    // Let's get Rick ready to serve over HTTP/2
    let mut h2orbit = init_orbit();

    // Let's make up the Left side we feed in as input
    let mut left = AbIoLeft::default();

    // Let's give Rick some room to grow
    left.out_bytes = vec![0; 8192];

    // Let's copy what we intend to want from Rick

    let input = "GET / HTTP/1.1\r\nHost: localhost:8080\r\nUser-Agent: curl/8.11.1\r\nAccept: */*\r\nConnection: Upgrade, HTTP2-Settings\r\nUpgrade: h2c\r\nHTTP2-Settings: AAMAAABkAAQAAQAAAAIAAAAA\r\n\r\n".as_bytes();

    let copied = left.copy_in(&input);
    assert_eq!(copied, input.len());

    // Spin the Rick's fortune wheel with what we have!
    let _pos = h2orbit
        .advance_with(&mut NothingBurger, &mut left, &mut NoRight)
        .unwrap();

    // Because we know where the HTTP Body starts we can just cheat and get the minimal HTTP response out from first spin
    let start = String::from_utf8_lossy(&left.out_bytes[0..43]);
    // We should have Rick's response back for the HTTP Layer as follows
    assert_eq!(
        start,
        "HTTP/1.1 200 OK\r\nContent-Length: 461688\r\n\r\n".to_string()
    );

    /*
    let mut rick_in: Vec<u8> = Vec::with_capacity(RICK_SIZE);
    rick_in.extend(&left.out_bytes[43..left.out_bytes_len]);
    left.out_bytes = vec![0; 8192];
    left.out_bytes_len = 0;

    // Now let's get rest of the Rick we expect..
    loop {
        let _pos = h11orbit
            .advance_with(&mut NothingBurger, &mut left, &mut NoRight)
            .unwrap();

        if left.out_bytes_len == 0 {
            break;
        }

        rick_in.extend(&left.out_bytes[0..left.out_bytes_len]);

        left.out_bytes = vec![0; 8192];
        left.out_bytes_len = 0;
    }

    assert_eq!(rick_in.len(), RICK_SIZE);
    */
}
