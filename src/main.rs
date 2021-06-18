#![doc(html_root_url = "https://docs.rs/teldump/0.0.1")]
#![warn(clippy::pedantic)]
#![allow(clippy::single_match_else)]

#[cfg(doctest)]
pub mod readme {
	doc_comment::doctest!("../README.md");
}

use std::{io::Write, time::Duration};
use telnet::{Telnet, TelnetEvent};

fn main() {
	// return checkmem();

	// let mut address = 0xa000_0000_i64; // RAM
	// let mut address = 0x8000_0000_i64;
	// let mut address = 0xBF_C0_00_00_i64; // Boot ROM
	let mut address = 0xBF_00_00_00_i64;
	let mut read_size = 0x1000_i64;

	address -= read_size;

	'reconnect: while read_size > 0 {
		eprint!("Connecting....");
		let mut telnet = match Telnet::connect(("192.168.1.1", 23), 0x1_00_00_00) {
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
					no_data = 0;
					print!("{}", String::from_utf8_lossy(&*data));
					std::io::stdout().flush().unwrap();
					eprint!("Read. ")
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
					eprint!(".");
					no_data += 1;
					if no_data >= 10 {
						no_data = 0;

						eprint!("\nRequesting more data: ");
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
								address += read_size;
								continue 'reconnect;
							}
						}
					} else {
						std::thread::sleep(Duration::from_millis(1))
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

#[allow(dead_code)]
fn checkmem() {
	let base = 0xA0_00_00_00_i64; // Unmapped uncached kseg1.
	let test_range = 0_i64..0x2000_0000;
	let step = 0x0100_0000;

	'next_address: for address in test_range.step_by(step) {
		eprintln!();
		eprint!("{:#x}", address);
		let mut telnet = loop {
			match Telnet::connect(("192.168.1.1", 23), 0x1_00_00_00) {
				Ok(telnet) => {
					break telnet;
				}
				Err(_) => {
					const WAIT: Duration = Duration::from_secs(1);
					std::thread::sleep(WAIT);
				}
			};
		};

		if telnet
			.write(format!("memory {:#x} 8 {:#x}\r\n", base + address, 1).as_bytes())
			.is_err()
		{
			continue;
		}

		while let Ok(event) = telnet.read_timeout(Duration::from_secs(1)) {
			match event {
				TelnetEvent::Data(data) => {
					let data = String::from_utf8_lossy(&*data);
					eprint!("{:?}", data);
					if data.contains("memory OK") {
						eprint!(" OK");
						continue 'next_address;
					}
				}
				TelnetEvent::UnknownIAC(_)
				| TelnetEvent::Negotiation(_, _)
				| TelnetEvent::Subnegotiation(_, _)
				| TelnetEvent::TimedOut
				| TelnetEvent::NoData
				| TelnetEvent::Error(_) => {}
			}
		}
	}
}
