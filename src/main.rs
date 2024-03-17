use gilrs::{Axis, Button, EventType, Gilrs};
use hex;
use std::io::Result;
use std::io::Write;
use std::net::TcpStream;

struct Camera {
    socket: TcpStream,
    z_command: String,
    pt_command: String,
}

impl Camera {
    fn new(addr: &str) -> Result<Camera> {
        let socket = match TcpStream::connect(&addr) {
            Ok(s) => s,
            Err(e) => {
                println!("Failed to connect: {:?}", e);
                return Err(e); // Or handle error appropriately.
            }
        };

        Ok(Camera {
            z_command: "81 01 04 07 30 FF".to_string(),
            pt_command: "81 01 06 01 00 00 00 00 FF ".to_string(),
            socket,
        })
    }

    fn send(&mut self) -> std::io::Result<()> {
        let command = ("00 0B ".to_string() + &self.pt_command).replace(" ", "");
        println!("{}", command);
        let byte_array: Vec<u8> = match hex::decode(command) {
            Ok(b) => b,
            Err(e) => Vec::new(),
        };
        let _ = self.socket.write_all(&byte_array);
        let _ = self.socket.flush();
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let mut gilrs = Gilrs::new().unwrap();
    for (id, gamepad) in gilrs.gamepads() {
        println!("Gamepad {}: {} is initially connected", id, gamepad.name());
    }
    let mut camera = Camera::new("192.168.250.52:5002")?;
    loop {
        while let Some(event) = gilrs.next_event() {
            match event.event {
                EventType::Connected => {
                    let gamepad = gilrs.gamepad(event.id);
                    println!("Gamepad {} is now connected: {}", event.id, gamepad.name());
                }
                EventType::Disconnected => {
                    println!("Gamepad {} is disconnected", event.id);
                }
                EventType::AxisChanged(axis, value, id) => match axis {
                    Axis::RightStickX => {
                        camera.pt_command.replace_range(
                            18..20,
                            if value.abs() * 24.0 < 0.5 {
                                "03"
                            } else {
                                if value > 0.001 {
                                    "02"
                                } else {
                                    "01"
                                }
                            },
                        );
                        let speed = (value * value * 24.0 + 0.5) as i8;
                        let hex_speed = format!("{:02X}", speed);
                        camera.pt_command.replace_range(12..14, &hex_speed);
                        let _ = camera.send();
                    }
                    Axis::RightStickY => {
                        camera.pt_command.replace_range(
                            21..23,
                            if value.abs() * 24.0 < 0.5 {
                                "03"
                            } else {
                                if value > 0.001 {
                                    "01"
                                } else {
                                    "02"
                                }
                            },
                        );
                        let speed = (value * value * 24.0 + 0.5) as i8;
                        let hex_speed = format!("{:02X}", speed);
                        camera.pt_command.replace_range(15..17, &hex_speed);
                        let _ = camera.send();
                    }

                    _ => println!("Unkown Axis"),
                },
                EventType::ButtonChanged(button, value, id) => match button {
                    Button::LeftTrigger2 => {
                        let cvalue = value - 0.5;
                        camera.pt_command.replace_range(
                            21..23,
                            if cvalue.abs() * 24.0 < 0.5 {
                                "03"
                            } else {
                                if cvalue > 0.001 {
                                    "01"
                                } else {
                                    "02"
                                }
                            },
                        );
                        let speed = (cvalue * cvalue * 24.0 + 0.5) as i8;
                        let hex_speed = format!("{:02X}", speed);
                        camera.pt_command.replace_range(15..17, &hex_speed);
                        let _ = camera.send();
                    }
                    _ => println!(
                        "Gamepad {} with unknown axis {:?} changed to {}",
                        id, button, value
                    ),
                },
                _ => {
                    println!("Unknown Event: {:?}", event.event);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
}
