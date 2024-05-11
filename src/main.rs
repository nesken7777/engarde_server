mod client_manager;
mod errors;
mod game;
mod protocol;
use std::{
    env::args,
    io::{self, stdout, BufReader, BufWriter, Write},
    net::{SocketAddr, TcpListener},
    thread,
};

use game::{GameManager, Kekka};
use protocol::{
    BoardInfo, ConnectionStart, DoPlay, GameEnd, HandInfo, Messages, NameReceived, PlayedAttack,
    PlayedMoveMent, PlayerID, RoundEnd, ServerError,
};

use crate::client_manager::{Client, ClientManager};

const MAX_WIN: u32 = 100;

fn print(string: &str) -> io::Result<()> {
    let mut stdout = stdout();
    stdout.write_all(string.as_bytes())?;
    stdout.flush()
}

enum ProcessResult {
    ReTry,
    Success(Kekka),
}

fn process_turn(
    game_manager: &mut GameManager,
    client_manager: &mut ClientManager,
) -> io::Result<ProcessResult> {
    client_manager.send(
        game_manager.board().current_player(),
        &HandInfo::from_vec(
            game_manager
                .player(game_manager.board().current_player())
                .hand(),
        ),
    )?;
    client_manager.send(game_manager.board().current_player(), &DoPlay::new())?;

    match Messages::parse(&client_manager.read(game_manager.board().current_player())?) {
        Ok(message) => match message {
            Messages::Eval(_) => {
                match Messages::parse(&client_manager.read(game_manager.board().current_player())?)
                {
                    Err(e) => {
                        print(format!("受信メッセージエラー: {}", e).as_str())?;
                        client_manager.send(
                            game_manager.board().current_player(),
                            &ServerError::new("送信されたメッセージがおかしいです"),
                        )?;
                        Ok(ProcessResult::ReTry)
                    }
                    Ok(Messages::Eval(_)) => {
                        client_manager.send(
                            game_manager.board().current_player(),
                            &ServerError::new("もうEvalは受け取りました"),
                        )?;
                        Ok(ProcessResult::ReTry)
                    }
                    Ok(Messages::PlayM(movement)) => {
                        match game_manager
                            .play_movement(game_manager.board().current_player(), &movement)
                        {
                            Ok(kekka @ Kekka::Continue) => {
                                client_manager.send(
                                    game_manager.board().current_player().opposite(),
                                    &PlayedMoveMent::new(&movement),
                                )?;
                                Ok(ProcessResult::Success(kekka))
                            }
                            Ok(kekka @ Kekka::REnd(None)) => Ok(ProcessResult::Success(kekka)),
                            Ok(kekka @ Kekka::REnd(Some(_))) => Ok(ProcessResult::Success(kekka)),
                            Err(e) => {
                                client_manager.send(
                                    game_manager.board().current_player(),
                                    &ServerError::new(e),
                                )?;
                                Ok(ProcessResult::ReTry)
                            }
                        }
                    }
                    Ok(Messages::PlayA(attack)) => {
                        match game_manager
                            .play_attack(game_manager.board().current_player(), &attack)
                        {
                            Ok(kekka @ Kekka::Continue) => {
                                client_manager.send(
                                    game_manager.board().current_player().opposite(),
                                    &PlayedAttack::new(&attack),
                                )?;
                                Ok(ProcessResult::Success(kekka))
                            }
                            Ok(kekka @ Kekka::REnd(None)) => Ok(ProcessResult::Success(kekka)),
                            Ok(kekka @ Kekka::REnd(Some(_))) => Ok(ProcessResult::Success(kekka)),
                            Err(e) => {
                                client_manager.send(
                                    game_manager.board().current_player(),
                                    &ServerError::new(e),
                                )?;
                                Ok(ProcessResult::ReTry)
                            }
                        }
                    }
                }
            }
            Messages::PlayM(_) => {
                client_manager.send(
                    game_manager.board().current_player(),
                    &ServerError::new("先にEvalしてください"),
                )?;
                Ok(ProcessResult::ReTry)
            }
            Messages::PlayA(_) => {
                client_manager.send(
                    game_manager.board().current_player(),
                    &ServerError::new("先にEvalしてください"),
                )?;
                Ok(ProcessResult::ReTry)
            }
        },
        Err(e) => {
            print(format!("受信メッセージエラー: {}", e).as_str())?;
            client_manager.send(
                game_manager.board().current_player(),
                &ServerError::new("送信されたメッセージがおかしいです"),
            )?;
            Ok(ProcessResult::ReTry)
        }
    }
}

fn process_round(
    game_manager: &mut GameManager,
    client_manager: &mut ClientManager,
) -> io::Result<()> {
    loop {
        client_manager.send(PlayerID::Zero, &BoardInfo::from_board(game_manager.board()))?;
        client_manager.send(PlayerID::One, &BoardInfo::from_board(game_manager.board()))?;
        let result = process_turn(game_manager, client_manager)?;
        match result {
            ProcessResult::ReTry => {}
            ProcessResult::Success(Kekka::Continue) => {
                *game_manager.current_playerid_mut() =
                    game_manager.board().current_player().opposite();
            }
            ProcessResult::Success(Kekka::REnd(None)) => {
                client_manager.send(PlayerID::Zero, &RoundEnd::hikiwake(game_manager.board()))?;
                client_manager.send(PlayerID::One, &RoundEnd::hikiwake(game_manager.board()))?;
                break;
            }
            ProcessResult::Success(Kekka::REnd(Some(winner))) => {
                client_manager.send(
                    PlayerID::Zero,
                    &RoundEnd::win_lose(game_manager.board(), winner),
                )?;
                client_manager.send(
                    PlayerID::One,
                    &RoundEnd::win_lose(game_manager.board(), winner),
                )?;
                break;
            }
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 12052)))?;
    let (stream0, _) = listener.accept()?;
    let mut client0 = Client::new(
        BufReader::new(stream0.try_clone()?),
        BufWriter::new(stream0),
    );
    let join0 = thread::spawn(move || -> io::Result<Client> {
        client0.send(&ConnectionStart::new(PlayerID::Zero))?;
        client0.read()?;
        client0.send(&NameReceived::new())?;
        Ok(client0)
    });
    let (stream1, _) = listener.accept()?;
    let mut client1 = Client::new(
        BufReader::new(stream1.try_clone()?),
        BufWriter::new(stream1),
    );
    let join1 = thread::spawn(move || -> io::Result<Client> {
        client1.send(&ConnectionStart::new(PlayerID::One))?;
        client1.read()?;
        client1.send(&NameReceived::new())?;
        Ok(client1)
    });
    let client0 = join0.join().expect("join失敗")?;
    let client1 = join1.join().expect("join失敗")?;
    let mut client_manager = ClientManager::new(client0, client1);
    let mut game_manager =
        GameManager::new((|| args().nth(1)?.parse::<u32>().ok())().unwrap_or(MAX_WIN));
    loop {
        process_round(&mut game_manager, &mut client_manager)?;
        game_manager.reset_round();
        match game_manager.ended() {
            None => game_manager.change_first_player(),
            Some(winner) => {
                client_manager.send(PlayerID::Zero, &GameEnd::new(game_manager.board(), winner))?;
                client_manager.send(PlayerID::One, &GameEnd::new(game_manager.board(), winner))?;
                break;
            }
        }
    }
    print("ゲーム終了")?;
    print(
        format!(
            "p0: {}点, p1: {}点",
            game_manager.board().score(PlayerID::Zero),
            game_manager.board().score(PlayerID::One)
        )
        .as_str(),
    )?;
    Ok(())
}
