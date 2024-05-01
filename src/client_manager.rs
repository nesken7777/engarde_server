use std::{
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
};

use serde::Serialize;

use crate::protocol::PlayerID;

pub struct Client {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl Client {
    pub fn new(reader: BufReader<TcpStream>, writer: BufWriter<TcpStream>) -> Self {
        Self { reader, writer }
    }

    pub fn send<T>(&mut self, info: &T) -> io::Result<()>
    where
        T: Serialize,
    {
        let string = format!("{}\r\n", serde_json::to_string(info)?);
        self.writer.write_all(string.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn read(&mut self) -> io::Result<String> {
        let mut string = String::new();
        self.reader.read_line(&mut string)?;
        Ok(string.trim().to_string())
    }
}

pub struct ClientManager {
    client0: Client,
    client1: Client,
}

impl ClientManager {
    pub fn new(client0: Client, client1: Client) -> Self {
        Self { client0, client1 }
    }

    pub fn client(&mut self, id: PlayerID) -> &mut Client {
        match id {
            PlayerID::Zero => &mut self.client0,
            PlayerID::One => &mut self.client1,
        }
    }

    pub fn send<T>(&mut self, id: PlayerID, info: &T) -> io::Result<()>
    where
        T: Serialize,
    {
        self.client(id).send(info)?;
        Ok(())
    }

    pub fn read(&mut self, id: PlayerID) -> io::Result<String> {
        self.client(id).read()
    }
}
