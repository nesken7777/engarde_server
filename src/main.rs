mod errors;
mod game;
mod protocol;
use std::{
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};

use errors::Errors;
use game::{Board, GameManager, Kekka};
use protocol::{
    BoardInfo, ConnectionStart, DoPlay, GameEnd, HandInfo, Messages, PlayedAttack, PlayedMoveMent,
    PlayerID, RoundEnd, ServerError,
};
use serde::Serialize;

fn read_stream<T>(bufreader: &mut BufReader<T>) -> io::Result<String>
where
    T: Read,
{
    let mut string = String::new();
    bufreader.read_line(&mut string)?;
    Ok(string.trim().to_string())
}

fn send_info<W, T>(writer: &mut BufWriter<W>, info: &T) -> Result<(), Errors>
where
    W: Write,
    T: Serialize,
{
    let string = format!("{}\r\n", serde_json::to_string(info)?);
    writer.write_all(string.as_bytes())?;
    writer.flush()?;
    Ok(())
}

enum ProcessResult {
    ReTry,
    Success(Kekka),
}

fn process_turn(
    game_manager: &mut GameManager,
    current_player: PlayerID,
    player_reader: &mut BufReader<TcpStream>,
    player_writer: &mut BufWriter<TcpStream>,
    opposite_writer: &mut BufWriter<TcpStream>,
) -> Result<ProcessResult, Errors> {
    send_info(
        player_writer,
        &HandInfo::from_vec(game_manager.player(current_player).hand()),
    )?;
    send_info(player_writer, &DoPlay::new())?;

    match Messages::parse(&read_stream(player_reader)?)? {
        Messages::Eval(_) => match Messages::parse(&read_stream(player_reader)?)? {
            Messages::Eval(_) => {
                send_info(player_writer, &ServerError::new("行動してください"))?;
                Ok(ProcessResult::ReTry)
            }
            Messages::PlayM(movement) => {
                match game_manager.play_movement(current_player, &movement) {
                    r @ Ok(Kekka::Continue) => {
                        send_info(opposite_writer, &PlayedMoveMent::new(movement))?;
                        Ok(r.map(ProcessResult::Success)?)
                    }
                    r @ Ok(Kekka::REnd(None)) => Ok(r.map(ProcessResult::Success)?),
                    r @ Ok(Kekka::REnd(Some(_))) => Ok(r.map(ProcessResult::Success)?),
                    Err(string) => {
                        send_info(player_writer, &ServerError::new(string))?;
                        Ok(ProcessResult::ReTry)
                    }
                }
            }
            Messages::PlayA(attack) => match game_manager.play_attack(current_player, &attack) {
                r @ Ok(Kekka::Continue) => {
                    send_info(opposite_writer, &PlayedAttack::new(attack))?;
                    Ok(r.map(ProcessResult::Success)?)
                }
                r @ Ok(Kekka::REnd(None)) => Ok(r.map(ProcessResult::Success)?),
                r @ Ok(Kekka::REnd(Some(_))) => Ok(r.map(ProcessResult::Success)?),
                Err(string) => {
                    send_info(player_writer, &ServerError::new(string))?;
                    Ok(ProcessResult::ReTry)
                }
            },
        },
        Messages::PlayM(_) => {
            send_info(player_writer, &ServerError::new("先にEvalしてください"))?;
            Ok(ProcessResult::ReTry)
        }
        Messages::PlayA(_) => {
            send_info(player_writer, &ServerError::new("先にEvalしてください"))?;
            Ok(ProcessResult::ReTry)
        }
    }
}

fn process_round(
    game_manager: &mut GameManager,
    bufwriter0: &mut BufWriter<TcpStream>,
    bufreader0: &mut BufReader<TcpStream>,
    bufwriter1: &mut BufWriter<TcpStream>,
    bufreader1: &mut BufReader<TcpStream>,
) -> Result<(), Errors> {
    let mut current_player = game_manager.first_player();
    loop {
        send_info(bufwriter0, &BoardInfo::from_board(game_manager.board()))?;
        send_info(bufwriter1, &BoardInfo::from_board(game_manager.board()))?;
        match current_player {
            PlayerID::Zero => {
                let result = process_turn(
                    game_manager,
                    current_player,
                    bufreader0,
                    bufwriter0,
                    bufwriter1,
                )?;
                match result {
                    ProcessResult::ReTry => (),
                    ProcessResult::Success(Kekka::Continue) => {
                        current_player = current_player.opposite()
                    }
                    ProcessResult::Success(Kekka::REnd(None)) => {
                        send_info(bufwriter0, &RoundEnd::hikiwake(game_manager.board()))?;
                        send_info(bufwriter1, &RoundEnd::hikiwake(game_manager.board()))?;
                        game_manager.reset_round();
                        break;
                    }
                    ProcessResult::Success(Kekka::REnd(Some(winner))) => {
                        send_info(
                            bufwriter0,
                            &RoundEnd::win_lose(game_manager.board(), winner),
                        )?;
                        send_info(
                            bufwriter1,
                            &RoundEnd::win_lose(game_manager.board(), winner),
                        )?;
                        game_manager.reset_round();
                        break;
                    }
                }
            }
            PlayerID::One => {
                let result = process_turn(
                    game_manager,
                    current_player,
                    bufreader1,
                    bufwriter1,
                    bufwriter0,
                )?;
                match result {
                    ProcessResult::ReTry => (),
                    ProcessResult::Success(Kekka::Continue) => {
                        current_player = current_player.opposite()
                    }
                    ProcessResult::Success(Kekka::REnd(None)) => {
                        send_info(bufwriter0, &RoundEnd::hikiwake(game_manager.board()))?;
                        send_info(bufwriter1, &RoundEnd::hikiwake(game_manager.board()))?;
                        game_manager.reset_round();
                        break;
                    }
                    ProcessResult::Success(Kekka::REnd(Some(winner))) => {
                        send_info(
                            bufwriter0,
                            &RoundEnd::win_lose(game_manager.board(), winner),
                        )?;
                        send_info(
                            bufwriter1,
                            &RoundEnd::win_lose(game_manager.board(), winner),
                        )?;
                        game_manager.reset_round();
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), Errors> {
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 12052)))?;
    let (stream0, _) = listener.accept()?;
    let (mut bufreader0, mut bufwriter0) = (
        BufReader::new(stream0.try_clone()?),
        BufWriter::new(stream0),
    );
    send_info(&mut bufwriter0, &ConnectionStart::new(0))?;
    read_stream(&mut bufreader0)?;
    send_info(&mut bufwriter0, &"a".to_string())?;
    let (stream1, _) = listener.accept()?;
    let (mut bufreader1, mut bufwriter1) = (
        BufReader::new(stream1.try_clone()?),
        BufWriter::new(stream1),
    );
    send_info(&mut bufwriter1, &ConnectionStart::new(1))?;
    read_stream(&mut bufreader1)?;
    send_info(&mut bufwriter1, &"a".to_string())?;
    let mut board = Board::new();
    let p0_hand = board.yamafuda.split_off(board.yamafuda.len() - 5);
    let p1_hand = board.yamafuda.split_off(board.yamafuda.len() - 5);
    let mut game_manager = GameManager::new(p0_hand, p1_hand, board);
    loop {
        process_round(
            &mut game_manager,
            &mut bufwriter0,
            &mut bufreader0,
            &mut bufwriter1,
            &mut bufreader1,
        )?;
        match game_manager.ended() {
            None => game_manager.change_first_player(),
            Some(winner) => {
                send_info(&mut bufwriter0, &GameEnd::new(game_manager.board(), winner))?;
                send_info(&mut bufwriter1, &GameEnd::new(game_manager.board(), winner))?;
                break;
            }
        }
    }
    println!("ゲーム終了");
    println!(
        "p0: {}点, p1: {}点",
        game_manager.board().score(PlayerID::Zero),
        game_manager.board().score(PlayerID::One)
    );
    Ok(())
}
