#![allow(unused_variables, unused_imports)]
use gilrs::{Axis, Button, EventType, Gilrs};
use hex;
use std::any::type_name;
use std::io::Result;
use std::io::{self, ErrorKind, Write};
use std::net::TcpStream;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, UdpSocket};
use std::time::Duration;
use stick::Event;

struct Camera {
    socket: TcpStream,
    zoom_stop: String,
    pt_stop: String,
    pan_bytes: String,
    pt_bytes_previous: String,
    zoom_bytes_previous: String,
    pan_speed: String,
    tilt_speed: String,
    pan_direction: String,
    tilt_direction: String,
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
            zoom_stop: "81 01 04 07 00 FF".to_string(),
            pt_stop: "81 01 06 01 01 01 03 03 FF".to_string(),
            pan_bytes: "".to_string(),
            pt_bytes_previous: "".to_string(),
            zoom_bytes_previous: "".to_string(),
            pan_speed: "01".to_string(),
            tilt_speed: "00".to_string(),
            pan_direction: "00".to_string(),
            tilt_direction: "00".to_string(),
            z_command: "81 01 04 07 30 FF".to_string(),
            pt_command: "81 01 06 01 00 00 00 00 FF ".to_string(),
            socket,
        })
    }
    // Left 01 Right 02 Stop 03
    // Up 01 Down 02 Stop 03

    fn left(&mut self, speed: f32) {
        self.pt_command.replace_range(12..13, "04");
        self.pt_command.replace_range(18..22, "01 03");
    }

    fn right(&mut self, speed: f32) {
        self.pt_command.replace_range(12..13, "04"); // Sets speed
        self.pt_command.replace_range(18..22, "02 03"); // Sets direction
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


    fn stop(&mut self) -> std::io::Result<()> {
        let stop = "00 0B 81 01 06 01 04 00 03 03 FF";
        let byte_array =
            hex::decode(stop).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
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
                EventType::AxisChanged(axis, value, id) => {
                    println!("Gamepad {} axis {:?} changed to {}", id, axis, value);
                    match axis {
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
                                if value * value * 24.0 < 0.5 {
                                    "03"
                                } else {
                                    if value > 0.001 {
                                        "01"
                                    } else {
                                        "02"
                                    }
                                },
                            );
                            let speed = (value.abs() * 24.0 + 0.5) as i8;
                            let hex_speed = format!("{:02X}", speed);
                            camera.pt_command.replace_range(15..17, &hex_speed);
                            let _ = camera.send();
                        }


                        _ => println!("Unkown Axis"),
                    }
                    // let a = if "01".to_string();
                }
                EventType::ButtonChanged(button, value, id) => {
                    println!("Gamepad {} axis {:?} changed to {}", id, button, value);
                }
                _ => {
                    println!("Unknown Event: {:?}", event.event);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
}
