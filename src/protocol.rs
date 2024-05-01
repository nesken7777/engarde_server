use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize, Serializer};
use serde_aux::prelude::*;
use serde_json::Value;
use serde_with::skip_serializing_none;

use crate::{errors::Errors, game::Board};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerID {
    Zero,
    One,
}

impl PlayerID {
    pub fn denote(&self) -> u8 {
        match self {
            PlayerID::Zero => 0,
            PlayerID::One => 1,
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            PlayerID::Zero => PlayerID::One,
            PlayerID::One => PlayerID::Zero,
        }
    }
}

impl<'de> Deserialize<'de> for PlayerID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MyEnumVisitor;

        impl<'de> serde::de::Visitor<'de> for MyEnumVisitor {
            type Value = PlayerID;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("0 or 1 or Zero or One")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v.trim() {
                    "0" => Ok(PlayerID::Zero),
                    "1" => Ok(PlayerID::One),
                    "Zero" => Ok(PlayerID::Zero),
                    "One" => Ok(PlayerID::One),
                    _ => Err(E::invalid_value(serde::de::Unexpected::Str(v), &self)),
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(v.as_str())
            }
        }

        deserializer.deserialize_string(MyEnumVisitor)
    }
}

impl Serialize for PlayerID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.denote().to_string().as_str())
    }
}

fn serialize_u8_as_string<S>(num: &u8, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&num.to_string())
}

fn serialize_i8_as_string<S>(num: &i8, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&num.to_string())
}

fn serialize_option_u8_as_string<S>(num: &Option<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match num {
        Some(num) => serializer.serialize_str(&num.to_string()),
        None => serializer.serialize_none(),
    }
}

fn serialize_u32_as_string<S>(num: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&num.to_string())
}

#[derive(Serialize, Debug)]
pub struct BoardInfo {
    #[serde(rename = "Type")]
    typ: &'static str,
    #[serde(rename = "From")]
    from: &'static str,
    #[serde(rename = "To")]
    to: &'static str,
    #[serde(
        rename = "PlayerPosition_0",
        deserialize_with = "deserialize_number_from_string",
        serialize_with = "serialize_u8_as_string"
    )]
    pub player_position_0: u8,
    #[serde(
        rename = "PlayerPosition_1",
        deserialize_with = "deserialize_number_from_string",
        serialize_with = "serialize_u8_as_string"
    )]
    pub player_position_1: u8,
    #[serde(
        rename = "PlayerScore_0",
        deserialize_with = "deserialize_number_from_string",
        serialize_with = "serialize_u32_as_string"
    )]
    pub player_score_0: u32,
    #[serde(
        rename = "PlayerScore_1",
        deserialize_with = "deserialize_number_from_string",
        serialize_with = "serialize_u32_as_string"
    )]
    pub player_score_1: u32,
    #[serde(
        rename = "NumofDeck",
        deserialize_with = "deserialize_number_from_string",
        serialize_with = "serialize_u8_as_string"
    )]
    pub num_of_deck: u8,
    #[serde(rename = "CurrentPlayer", default)]
    pub current_player: PlayerID,
}

impl BoardInfo {
    pub fn from_board(board: &Board) -> Self {
        BoardInfo {
            typ: "BoardInfo",
            from: "Server",
            to: "Client",
            player_position_0: board.pos(PlayerID::Zero),
            player_position_1: board.pos(PlayerID::One),
            player_score_0: board.score(PlayerID::Zero),
            player_score_1: board.score(PlayerID::One),
            num_of_deck: board.yamafuda().len() as u8,
            current_player: board.current_player(),
        }
    }
}

#[derive(Serialize)]
pub struct HandInfo {
    #[serde(rename = "Type")]
    typ: &'static str,
    #[serde(rename = "From")]
    from: &'static str,
    #[serde(rename = "To")]
    to: &'static str,
    #[serde(
        rename = "Hand1",
        deserialize_with = "deserialize_number_from_string",
        serialize_with = "serialize_u8_as_string"
    )]
    pub hand1: u8,
    #[serde(
        rename = "Hand2",
        deserialize_with = "deserialize_number_from_string",
        serialize_with = "serialize_u8_as_string"
    )]
    pub hand2: u8,
    #[serde(
        rename = "Hand3",
        deserialize_with = "deserialize_number_from_string",
        serialize_with = "serialize_u8_as_string"
    )]
    pub hand3: u8,
    #[serde(
        rename = "Hand4",
        serialize_with = "serialize_option_u8_as_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub hand4: Option<u8>,
    #[serde(
        rename = "Hand5",
        serialize_with = "serialize_option_u8_as_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub hand5: Option<u8>,
}

impl HandInfo {
    pub fn from_vec(v: &[u8]) -> Self {
        Self {
            typ: "HandInfo",
            from: "Server",
            to: "Client",
            hand1: v.first().copied().unwrap(),
            hand2: v.get(1).copied().unwrap(),
            hand3: v.get(2).copied().unwrap(),
            hand4: v.get(3).copied(),
            hand5: v.get(4).copied(),
        }
    }
}

#[derive(Serialize)]
pub struct DoPlay {
    #[serde(rename = "Type")]
    typ: &'static str,
    #[serde(rename = "From")]
    from: &'static str,
    #[serde(rename = "To")]
    to: &'static str,
    #[serde(rename = "MessageID", serialize_with = "serialize_u8_as_string")]
    pub message_id: u8,
    #[serde(rename = "Message")]
    message: &'static str,
}

impl DoPlay {
    pub fn new() -> Self {
        Self {
            typ: "DoPlay",
            from: "Server",
            to: "Client",
            message_id: 101,
            message: "a",
        }
    }
}

#[derive(Serialize)]
pub struct Accept {
    #[serde(rename = "Type")]
    typ: String,
    #[serde(rename = "From")]
    from: String,
    #[serde(rename = "To")]
    to: String,
    #[serde(rename = "MessageID")]
    message_id: String,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Serialize)]
pub enum Direction {
    Forward,
    Back,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forward => write!(f, "F"),
            Self::Back => write!(f, "B"),
        }
    }
}

impl<'de> Deserialize<'de> for Direction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MyEnumVisitor;

        impl<'de> serde::de::Visitor<'de> for MyEnumVisitor {
            type Value = Direction;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("0 or 1 or Zero or One")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v.trim() {
                    "B" => Ok(Direction::Back),
                    "Back" => Ok(Direction::Back),
                    "F" => Ok(Direction::Forward),
                    "Forward" => Ok(Direction::Forward),
                    _ => Err(E::invalid_value(serde::de::Unexpected::Str(v), &self)),
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(v.as_str())
            }
        }

        deserializer.deserialize_string(MyEnumVisitor)
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Movement {
    card: u8,
    direction: Direction,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Attack {
    card: u8,
    quantity: u8,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Action {
    Move(Movement),
    Attack(Attack),
}

#[derive(Serialize)]
pub struct PlayedMoveMent {
    #[serde(rename = "Type")]
    typ: &'static str,
    #[serde(rename = "From")]
    from: &'static str,
    #[serde(rename = "To")]
    to: &'static str,
    #[serde(rename = "MessageID")]
    pub message_id: &'static str,
    #[serde(rename = "PlayCard", serialize_with = "serialize_u8_as_string")]
    pub play_card: u8,
    #[serde(rename = "Direction")]
    pub direction: String,
}

impl PlayedMoveMent {
    pub fn new(play: &PlayMovement) -> Self {
        Self {
            typ: "Played",
            from: "Server",
            to: "Client",
            message_id: "101",
            play_card: play.play_card,
            direction: play.direction.to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct PlayedAttack {
    #[serde(rename = "Type")]
    typ: &'static str,
    #[serde(rename = "From")]
    from: &'static str,
    #[serde(rename = "To")]
    to: &'static str,
    #[serde(rename = "MessageID")]
    pub message_id: &'static str,
    #[serde(rename = "PlayCard", serialize_with = "serialize_u8_as_string")]
    pub play_card: u8,
    #[serde(rename = "NumOfCard", serialize_with = "serialize_u8_as_string")]
    pub num_of_card: u8,
}

impl PlayedAttack {
    pub fn new(play: &PlayAttack) -> Self {
        Self {
            typ: "Played",
            from: "Server",
            to: "Client",
            message_id: "102",
            play_card: play.play_card,
            num_of_card: play.num_of_card,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct RoundEnd {
    #[serde(rename = "Type")]
    typ: &'static str,
    #[serde(rename = "From")]
    from: &'static str,
    #[serde(rename = "To")]
    to: &'static str,
    #[serde(rename = "RWinner", serialize_with = "serialize_i8_as_string")]
    pub round_winner: i8,
    #[serde(rename = "Score0", serialize_with = "serialize_u32_as_string")]
    pub score_0: u32,
    #[serde(rename = "Score1", serialize_with = "serialize_u32_as_string")]
    pub score_1: u32,
    #[serde(rename = "Message")]
    pub message: &'static str,
}

impl RoundEnd {
    pub fn hikiwake(board: &Board) -> Self {
        Self {
            typ: "RoundEnd",
            from: "Server",
            to: "Client",
            round_winner: -1,
            score_0: board.score(PlayerID::Zero),
            score_1: board.score(PlayerID::One),
            message: "a",
        }
    }
    pub fn win_lose(board: &Board, winner: PlayerID) -> Self {
        Self {
            typ: "RoundEnd",
            from: "Server",
            to: "Client",
            round_winner: winner.denote() as i8,
            score_0: board.score(PlayerID::Zero),
            score_1: board.score(PlayerID::One),
            message: "a",
        }
    }
}

#[derive(Serialize, Debug)]
pub struct GameEnd {
    #[serde(rename = "Type")]
    typ: &'static str,
    #[serde(rename = "From")]
    from: &'static str,
    #[serde(rename = "To")]
    to: &'static str,
    #[serde(rename = "Winner", serialize_with = "serialize_u8_as_string")]
    pub winner: u8,
    #[serde(rename = "Score0", serialize_with = "serialize_u32_as_string")]
    pub score_0: u32,
    #[serde(rename = "Score1", serialize_with = "serialize_u32_as_string")]
    pub score_1: u32,
    #[serde(rename = "Message")]
    pub message: &'static str,
}

impl GameEnd {
    pub fn new(board: &Board, winner: PlayerID) -> Self {
        Self {
            typ: "GameEnd",
            from: "Server",
            to: "Client",
            winner: winner.denote(),
            score_0: board.score(PlayerID::Zero),
            score_1: board.score(PlayerID::One),
            message: "a",
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename = "Error")]
pub struct ServerError {
    #[serde(rename = "Type")]
    typ: &'static str,
    #[serde(rename = "From")]
    from: &'static str,
    #[serde(rename = "To")]
    to: &'static str,
    #[serde(rename = "Message")]
    message: &'static str,
    #[serde(rename = "MessageID")]
    message_id: &'static str,
}

impl ServerError {
    pub fn new(string: &'static str) -> Self {
        Self {
            typ: "Error",
            from: "Server",
            to: "Client",
            message: string,
            message_id: "111",
        }
    }
}

#[derive(Debug)]
pub struct ParseMessageError {
    invalid_info: String,
}

impl Display for ParseMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MessageParseError, json is {}", self.invalid_info)
    }
}

impl Error for ParseMessageError {}

#[derive(Serialize, Debug)]
pub struct ConnectionStart {
    #[serde(rename = "Type")]
    typ: &'static str,
    #[serde(rename = "From")]
    from: &'static str,
    #[serde(rename = "To")]
    to: &'static str,
    #[serde(rename = "ClientID")]
    pub client_id: PlayerID,
}

impl ConnectionStart {
    pub fn new(id: PlayerID) -> Self {
        Self {
            typ: "ConnectionStart",
            from: "Server",
            to: "Client",
            client_id: id,
        }
    }
}

#[derive(Deserialize)]
pub struct PlayerName {
    #[serde(rename = "Type")]
    typ: &'static str,
    #[serde(rename = "From")]
    from: &'static str,
    #[serde(rename = "To")]
    to: &'static str,
    #[serde(rename = "Name")]
    name: String,
}

#[derive(Serialize, Debug)]
pub struct NameReceived {
    #[serde(rename = "Type")]
    typ: &'static str,
    #[serde(rename = "From")]
    from: &'static str,
    #[serde(rename = "To")]
    to: &'static str,
}

impl NameReceived {
    pub fn new() -> Self {
        Self {
            typ: "NameReceived",
            from: "Server",
            to: "Client",
        }
    }
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct Evaluation {
    #[serde(rename = "Type")]
    typ: String,
    #[serde(rename = "From")]
    from: String,
    #[serde(rename = "To")]
    to: String,
    #[serde(rename = "1F", default)]
    eval_1f: Option<String>,
    #[serde(rename = "1B", default)]
    eval_1b: Option<String>,
    #[serde(rename = "2F", default)]
    eval_2f: Option<String>,
    #[serde(rename = "2B", default)]
    eval_2b: Option<String>,
    #[serde(rename = "3F", default)]
    eval_3f: Option<String>,
    #[serde(rename = "3B", default)]
    eval_3b: Option<String>,
    #[serde(rename = "4F", default)]
    eval_4f: Option<String>,
    #[serde(rename = "4B", default)]
    eval_4b: Option<String>,
    #[serde(rename = "5F", default)]
    eval_5f: Option<String>,
    #[serde(rename = "5B", default)]
    eval_5b: Option<String>,
}

#[derive(Deserialize)]
pub struct PlayMovement {
    #[serde(rename = "Type")]
    typ: String,
    #[serde(rename = "From")]
    from: String,
    #[serde(rename = "To")]
    to: String,
    #[serde(rename = "MessageID")]
    message_id: String,
    #[serde(
        rename = "PlayCard",
        deserialize_with = "deserialize_number_from_string"
    )]
    play_card: u8,
    #[serde(rename = "Direction")]
    direction: Direction,
}

impl PlayMovement {
    pub fn play_card(&self) -> u8 {
        self.play_card
    }
    pub fn direction(&self) -> Direction {
        self.direction
    }
}

#[derive(Deserialize)]
pub struct PlayAttack {
    #[serde(rename = "Type")]
    typ: String,
    #[serde(rename = "From")]
    from: String,
    #[serde(rename = "To")]
    to: String,
    #[serde(rename = "MessageID")]
    message_id: String,
    #[serde(
        rename = "PlayCard",
        deserialize_with = "deserialize_number_from_string"
    )]
    play_card: u8,
    #[serde(
        rename = "NumOfCard",
        deserialize_with = "deserialize_number_from_string"
    )]
    num_of_card: u8,
}

impl PlayAttack {
    pub fn play_card(&self) -> u8 {
        self.play_card
    }

    pub fn num_of_card(&self) -> u8 {
        self.num_of_card
    }
}

pub enum Messages {
    Eval(Box<Evaluation>),
    PlayM(PlayMovement),
    PlayA(PlayAttack),
}

impl Messages {
    pub fn parse(json: &str) -> Result<Messages, Errors> {
        let obj = serde_json::from_str::<Value>(json)?;
        let typ = obj
            .get("Type")
            .ok_or("Typeキー無し")?
            .as_str()
            .ok_or("Typeキーが文字列ではない")?;
        match typ {
            "Evaluation" => Ok(Messages::Eval(serde_json::from_str(json)?)),
            "Play" => {
                let message_id = obj
                    .get("MessageID")
                    .ok_or("MessageID無し")?
                    .as_str()
                    .ok_or("MessageIDが文字列ではない")?;
                match message_id {
                    "101" => Ok(Messages::PlayM(serde_json::from_str(json)?)),
                    "102" => Ok(Messages::PlayA(serde_json::from_str(json)?)),
                    _ => Err(ParseMessageError {
                        invalid_info: json.to_string(),
                    })?,
                }
            }
            _ => Err(ParseMessageError {
                invalid_info: json.to_string(),
            })?,
        }
    }
}
