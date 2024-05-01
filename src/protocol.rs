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
        serializer.serialize_u8(self.denote())
    }
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
        deserialize_with = "deserialize_number_from_string"
    )]
    pub player_position_0: u8,
    #[serde(
        rename = "PlayerPosition_1",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub player_position_1: u8,
    #[serde(
        rename = "PlayerScore_0",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub player_score_0: u32,
    #[serde(
        rename = "PlayerScore_1",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub player_score_1: u32,
    #[serde(
        rename = "NumofDeck",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub num_of_deck: u8,
    #[serde(rename = "CurrentPlayer", default)]
    pub current_player: Option<PlayerID>,
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
            current_player: None,
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
    #[serde(rename = "Hand1", deserialize_with = "deserialize_number_from_string")]
    pub hand1: u8,
    #[serde(rename = "Hand2", deserialize_with = "deserialize_number_from_string")]
    pub hand2: u8,
    #[serde(rename = "Hand3", deserialize_with = "deserialize_number_from_string")]
    pub hand3: u8,
    #[serde(
        rename = "Hand4",
        default,
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub hand4: Option<u8>,
    #[serde(
        rename = "Hand5",
        default,
        deserialize_with = "deserialize_option_number_from_string"
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
    #[serde(
        rename = "MessageID",
        deserialize_with = "deserialize_number_from_string"
    )]
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
    pub card: u8,
    pub direction: Direction,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Attack {
    pub card: u8,
    pub quantity: u8,
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
    #[serde(
        rename = "PlayCard",
        deserialize_with = "deserialize_number_from_string"
    )]
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
    #[serde(
        rename = "PlayCard",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub play_card: u8,
    #[serde(
        rename = "NumOfCard",
        deserialize_with = "deserialize_number_from_string"
    )]
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
    #[serde(
        rename = "RWinner",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub round_winner: i8,
    #[serde(rename = "Score0", deserialize_with = "deserialize_number_from_string")]
    pub score_0: u32,
    #[serde(rename = "Score1", deserialize_with = "deserialize_number_from_string")]
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
    #[serde(rename = "Winner", deserialize_with = "deserialize_number_from_string")]
    pub winner: u8,
    #[serde(rename = "Score0", deserialize_with = "deserialize_number_from_string")]
    pub score_0: u32,
    #[serde(rename = "Score1", deserialize_with = "deserialize_number_from_string")]
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
    pub typ: &'static str,
    #[serde(rename = "From")]
    pub from: &'static str,
    #[serde(rename = "To")]
    pub to: &'static str,
    #[serde(rename = "Name")]
    pub name: String,
}

#[derive(Serialize, Debug)]
pub struct NameReceived {
    #[serde(rename = "Type")]
    typ: String,
    #[serde(rename = "From")]
    from: String,
    #[serde(rename = "To")]
    to: String,
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct Evaluation {
    #[serde(rename = "Type")]
    pub typ: String,
    #[serde(rename = "From")]
    pub from: String,
    #[serde(rename = "To")]
    pub to: String,
    #[serde(rename = "1F", default)]
    pub eval_1f: Option<String>,
    #[serde(rename = "1B", default)]
    pub eval_1b: Option<String>,
    #[serde(rename = "2F", default)]
    pub eval_2f: Option<String>,
    #[serde(rename = "2B", default)]
    pub eval_2b: Option<String>,
    #[serde(rename = "3F", default)]
    pub eval_3f: Option<String>,
    #[serde(rename = "3B", default)]
    pub eval_3b: Option<String>,
    #[serde(rename = "4F", default)]
    pub eval_4f: Option<String>,
    #[serde(rename = "4B", default)]
    pub eval_4b: Option<String>,
    #[serde(rename = "5F", default)]
    pub eval_5f: Option<String>,
    #[serde(rename = "5B", default)]
    pub eval_5b: Option<String>,
}

#[derive(Deserialize)]
pub struct PlayMovement {
    #[serde(rename = "Type")]
    pub typ: String,
    #[serde(rename = "From")]
    pub from: String,
    #[serde(rename = "To")]
    pub to: String,
    #[serde(rename = "MessageID")]
    pub message_id: String,
    #[serde(
        rename = "PlayCard",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub play_card: u8,
    #[serde(rename = "Direction")]
    pub direction: Direction,
}

#[derive(Deserialize)]
pub struct PlayAttack {
    #[serde(rename = "Type")]
    pub typ: String,
    #[serde(rename = "From")]
    pub from: String,
    #[serde(rename = "To")]
    pub to: String,
    #[serde(rename = "MessageID")]
    pub message_id: String,
    #[serde(
        rename = "PlayCard",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub play_card: u8,
    #[serde(
        rename = "NumOfCard",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub num_of_card: u8,
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
