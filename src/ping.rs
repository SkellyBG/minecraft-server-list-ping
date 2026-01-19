use std::{
    io::{Read, Write},
    net::TcpStream,
};

use serde::Deserialize;

use anyhow::Error;

const SERVER_IP: &str = "192.9.189.202";
const SERVER_PORT: u16 = 25565;

const SEGMENT_BITS: i32 = 0x7F;
const CONTINUE_BIT: i32 = 0x80;

#[derive(Debug, Deserialize)]

struct StatusResponse {
    players: StatusResponsePlayers,
}

#[derive(Debug, Deserialize)]
struct StatusResponsePlayers {
    sample: Option<Vec<StatusResponsePlayer>>,
}

#[derive(Debug, Deserialize)]
struct StatusResponsePlayer {
    name: String,
}

pub fn minecraft_ping() -> Result<Vec<String>, Error> {
    let mut stream = TcpStream::connect(format!("{SERVER_IP}:{SERVER_PORT}")).unwrap();

    // handshake
    let mut handshake = vec![0x00, 0x00];
    handshake.extend_from_slice(&pack_data(SERVER_IP.as_bytes()));
    handshake.extend_from_slice(&SERVER_PORT.to_be_bytes());
    handshake.push(0x01);
    stream.write_all(&pack_data(&handshake))?;
    // status request
    stream.write_all(&pack_data(&[0x00]))?;

    unpack_var_int(&mut stream);
    unpack_var_int(&mut stream);
    let body_length = unpack_var_int(&mut stream);

    let mut buf = vec![0; body_length as usize];

    stream.read_exact(&mut buf)?;
    let body = serde_json::from_slice::<StatusResponse>(&buf)?;

    let players = body
        .players
        .sample
        .unwrap_or_default()
        .into_iter()
        .map(|player| player.name)
        .collect();

    Ok(players)
}

fn pack_data(data: &[u8]) -> Vec<u8> {
    let mut packet = pack_var_int(data.len().try_into().unwrap());
    packet.extend_from_slice(data);

    packet
}

fn pack_var_int(mut value: i32) -> Vec<u8> {
    let mut res = Vec::new();
    loop {
        if value & !SEGMENT_BITS == 0 {
            res.push(value.to_le_bytes()[0]);
            return res;
        }
        res.push(((value & SEGMENT_BITS) | CONTINUE_BIT).to_le_bytes()[0]);

        // cast to u32 for logical right shift
        value = ((value as u32) >> 7) as i32;
    }
}

fn unpack_var_int(stream: &mut TcpStream) -> i32 {
    let mut value = 0;
    let mut shift = 0;
    loop {
        let mut buf = [0];
        stream.read_exact(&mut buf).unwrap();

        let byte = i32::from(buf[0]);

        value |= (byte & SEGMENT_BITS) << shift;
        if byte & CONTINUE_BIT == 0 {
            return value;
        }

        shift += 7;
    }
}

#[cfg(test)]
mod test {
    use crate::ping::pack_var_int;
    // use crate::unpack_var_int;

    #[test]
    fn to_var_int_tests() {
        let cases = [
            (0, vec![0x00]),
            (1, vec![0x01]),
            (2, vec![0x02]),
            (127, vec![0x7F]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xFF, 0x01]),
            (25565, vec![0xdd, 0xc7, 0x01]),
            (2147483647, vec![0xff, 0xff, 0xff, 0xff, 0x07]),
            (-1, vec![0xff, 0xff, 0xff, 0xff, 0x0f]),
        ];

        for (input, expected) in cases {
            assert_eq!(
                pack_var_int(input),
                expected,
                "With input {input}, expected {expected:?}"
            );
        }
    }

    // #[test]
    // fn from_var_int_tests() {
    //     let cases = [
    //         (0, vec![0x00]),
    //         (1, vec![0x01]),
    //         (2, vec![0x02]),
    //         (127, vec![0x7F]),
    //         (128, vec![0x80, 0x01]),
    //         (255, vec![0xFF, 0x01]),
    //         (25565, vec![0xdd, 0xc7, 0x01]),
    //         (2147483647, vec![0xff, 0xff, 0xff, 0xff, 0x07]),
    //         (-1, vec![0xff, 0xff, 0xff, 0xff, 0x0f]),
    //     ];

    //     for (expected, input) in cases {
    //         assert_eq!(
    //             unpack_var_int(&input),
    //             expected,
    //             "With input {input:?}, expected {expected}"
    //         );
    //     }
    // }
}
