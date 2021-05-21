#![doc(html_root_url = "https://docs.rs/teldump/0.0.1")]
#![warn(clippy::pedantic)]
#![allow(clippy::single_match_else)]

#[cfg(doctest)]
pub mod readme {
	doc_comment::doctest!("../README.md");
}

use std::time::Duration;

use telnet::{Telnet, TelnetEvent};

fn main() {
	let mut address = 0x8000_0000_i64;
	let mut read_size = 0x1000_i64;

	address -= read_size;

	'reconnect: while read_size > 0 {
		eprint!("Connecting....");
		let mut telnet = match Telnet::connect(("192.168.1.1", 23), 256) {
			Ok(telnet) => {
				eprintln!("OK");
				telnet
			}
			Err(_) => {
				const WAIT: Duration = Duration::from_secs(1);
				eprintln!("Failure to connect. Retrying in {:?}.", WAIT);
				std::thread::sleep(WAIT);
				continue 'reconnect;
			}
		};

		let mut no_data = 0;
		loop {
			match match telnet.read_nonblocking() {
				Ok(ok) => ok,
				Err(_) => {
					eprintln!("Failed to read. The device likely reset due to an out-of-bounds read. Retrying with smaller read size.");
					read_size /= 2;
					continue 'reconnect;
				}
			} {
				TelnetEvent::Data(data) => {
					print!("{}", String::from_utf8_lossy(&*data))
				}
				TelnetEvent::UnknownIAC(_) => {
					todo!("UnknownIAC");
				}
				TelnetEvent::Negotiation(_, _) => {
					todo!("Negotiation");
				}
				TelnetEvent::Subnegotiation(_, _) => {
					todo!("Subnegotiation");
				}
				TelnetEvent::TimedOut => {
					eprintln!("Timed out. The device likely reset due to an out-of-bounds read. Retrying with smaller read size.");
					read_size /= 2;
					continue 'reconnect;
				}
				TelnetEvent::NoData => {
					no_data += 1;
					if no_data >= 10 {
						no_data = 0;

						eprint!("Requesting more data: ");
						address += read_size;
						let query = format!("memory {:#x} 8 {:#x}\r\n", address, read_size);
						eprint!("{}", query);

						match telnet.write(query.as_bytes()) {
							Ok(written) => {
								assert_eq!(written, query.as_bytes().len());
							}
							Err(_) => {
								eprintln!("Could not write. The device likely reset due to an out-of-bounds read. Retrying with smaller read size.");
								address -= read_size;
								read_size /= 2;
							}
						}
					} else {
						std::thread::sleep(Duration::from_millis(5))
					}
				}
				TelnetEvent::Error(_) => {}
			}
		}
	}

	eprintln!(
		"Read size has decreased to {}. Exiting. Have a good day.",
		read_size
	);
}
