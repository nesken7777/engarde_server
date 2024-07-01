use crate::protocol::{Direction, PlayAttack, PlayMovement, PlayerID};
use rand::prelude::SliceRandom;

const MOST_LEFT_SIDE: u8 = 1;
const MOST_RIGHT_SIDE: u8 = 23;

struct Yamafuda;
impl Yamafuda {
    fn create() -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let mut v = (1..=5).map(|i| [i; 5]).collect::<Vec<_>>().concat();
        v.shuffle(&mut rng);
        v
    }
}

#[derive(Debug)]
pub struct Board {
    p0_pos: u8,
    p1_pos: u8,
    p0_score: u32,
    p1_score: u32,
    //この中のu8はカード番号
    yamafuda: Vec<u8>,
    current_player: PlayerID,
}

impl Board {
    fn new() -> Self {
        Self {
            p0_pos: MOST_LEFT_SIDE,
            p1_pos: MOST_RIGHT_SIDE,
            p0_score: 0,
            p1_score: 0,
            yamafuda: Yamafuda::create(),
            current_player: PlayerID::Zero,
        }
    }

    pub fn pos(&self, id: PlayerID) -> u8 {
        match id {
            PlayerID::Zero => self.p0_pos,
            PlayerID::One => self.p1_pos,
        }
    }

    pub fn score(&self, id: PlayerID) -> u32 {
        match id {
            PlayerID::Zero => self.p0_score,
            PlayerID::One => self.p1_score,
        }
    }

    pub fn yamafuda(&self) -> &[u8] {
        &self.yamafuda
    }

    pub fn current_player(&self) -> PlayerID {
        self.current_player
    }

    fn score_mut(&mut self, id: PlayerID) -> &mut u32 {
        match id {
            PlayerID::Zero => &mut self.p0_score,
            PlayerID::One => &mut self.p1_score,
        }
    }
}

pub enum Kekka {
    REnd(Option<PlayerID>),
    Continue,
}

#[derive(Debug)]
pub struct Player {
    id: PlayerID,
    hand: Vec<u8>,
}

impl Player {
    pub fn can_move(&self, board: &Board, card: u8, direction: Direction) -> bool {
        match (self.id, direction) {
            (PlayerID::Zero, Direction::Back) => {
                board.pos(self.id).saturating_sub(card) >= MOST_LEFT_SIDE
            }
            (PlayerID::Zero, Direction::Forward) => {
                board.pos(self.id) + card < board.pos(self.id.opposite())
            }
            (PlayerID::One, Direction::Back) => board.pos(self.id) + card <= MOST_RIGHT_SIDE,
            (PlayerID::One, Direction::Forward) => {
                board.pos(self.id).saturating_sub(card) > board.pos(self.id.opposite())
            }
        }
    }
    pub fn can_attack(&self, board: &Board, card: u8) -> bool {
        match self.id {
            PlayerID::Zero => board.pos(self.id.opposite()) - board.pos(self.id) == card,
            PlayerID::One => board.pos(self.id) - board.pos(self.id.opposite()) == card,
        }
    }
    pub fn can_actions(&self, board: &Board) -> bool {
        let attacks = self.hand().iter().any(|&card| self.can_attack(board, card));
        let move_forwards = self
            .hand()
            .iter()
            .any(|&card| self.can_move(board, card, Direction::Forward));
        let move_backwards = self
            .hand()
            .iter()
            .any(|&card| self.can_move(board, card, Direction::Back));
        attacks || move_forwards || move_backwards
    }
    pub fn card_pos(&self, card: u8) -> Option<usize> {
        self.hand.iter().position(|&x| x == card)
    }
    pub fn card_positions(&self, card: u8) -> Vec<usize> {
        self.hand
            .iter()
            .enumerate()
            .filter(|(_, &v)| v == card)
            .map(|(i, _)| i)
            .collect::<Vec<usize>>()
    }
    pub fn remove_card(&mut self, index: usize) {
        self.hand.remove(index);
    }
    pub fn push_card(&mut self, card: u8) {
        self.hand.push(card);
    }
    pub fn count_card(&self, card: u8) -> usize {
        self.hand.iter().filter(|&&x| x == card).count()
    }
    pub fn hand(&self) -> &[u8] {
        &self.hand
    }
}

pub struct GameManager {
    p0: Player,
    p1: Player,
    board: Board,
    first_player: PlayerID,
    game_end: Option<PlayerID>,
    max_win: u32,
}

impl GameManager {
    pub fn new(max_win: u32) -> Self {
        let mut board = Board::new();
        let p0_hand = board.yamafuda.split_off(board.yamafuda.len() - 5);
        let p1_hand = board.yamafuda.split_off(board.yamafuda.len() - 5);
        Self {
            p0: Player {
                id: PlayerID::Zero,
                hand: p0_hand,
            },
            p1: Player {
                id: PlayerID::One,
                hand: p1_hand,
            },
            first_player: PlayerID::Zero,
            board,
            game_end: None,
            max_win,
        }
    }
    pub fn change_first_player(&mut self) {
        self.first_player = self.first_player.opposite();
        *self.current_playerid_mut() = self.first_player;
    }
    pub fn player(&self, id: PlayerID) -> &Player {
        match id {
            PlayerID::Zero => &self.p0,
            PlayerID::One => &self.p1,
        }
    }
    pub fn player_mut(&mut self, id: PlayerID) -> &mut Player {
        match id {
            PlayerID::Zero => &mut self.p0,
            PlayerID::One => &mut self.p1,
        }
    }
    pub fn board(&self) -> &Board {
        &self.board
    }
    pub fn current_playerid_mut(&mut self) -> &mut PlayerID {
        &mut self.board.current_player
    }
    pub fn ended(&self) -> Option<PlayerID> {
        self.game_end
    }
    pub fn reset_round(&mut self) {
        let mut yamafuda = Yamafuda::create();
        self.p0.hand = yamafuda.split_off(yamafuda.len() - 5);
        self.p1.hand = yamafuda.split_off(yamafuda.len() - 5);
        self.board.p0_pos = MOST_LEFT_SIDE;
        self.board.p1_pos = MOST_RIGHT_SIDE;
        self.board.yamafuda = yamafuda;
    }
    fn move_player(&mut self, id: PlayerID, direction: Direction, card: u8) {
        match (id, direction) {
            (PlayerID::Zero, Direction::Back) => self.board.p0_pos -= card,
            (PlayerID::Zero, Direction::Forward) => self.board.p0_pos += card,
            (PlayerID::One, Direction::Back) => self.board.p1_pos += card,
            (PlayerID::One, Direction::Forward) => self.board.p1_pos -= card,
        }
    }
    fn round_end_yamafuda(&mut self) -> Kekka {
        let distance = self.board.pos(PlayerID::One) - self.board.pos(PlayerID::Zero);
        let have0 = self.player(PlayerID::Zero).count_card(distance);
        let have1 = self.player(PlayerID::One).count_card(distance);
        match have0.cmp(&have1) {
            std::cmp::Ordering::Less => {
                self.board.p1_score += 1;
                if self.board.p1_score >= self.max_win {
                    self.game_end = Some(PlayerID::One);
                }
                Kekka::REnd(Some(PlayerID::One))
            }
            std::cmp::Ordering::Greater => {
                self.board.p0_score += 1;
                if self.board.p0_score >= self.max_win {
                    self.game_end = Some(PlayerID::Zero);
                }
                Kekka::REnd(Some(PlayerID::Zero))
            }
            std::cmp::Ordering::Equal => {
                let distance_from_opposite_0 = MOST_RIGHT_SIDE - self.board.pos(PlayerID::Zero);
                let distance_from_opposite_1 = self.board.pos(PlayerID::One) - MOST_LEFT_SIDE;
                match distance_from_opposite_0.cmp(&distance_from_opposite_1) {
                    std::cmp::Ordering::Less => {
                        self.board.p0_score += 1;
                        if self.board.p0_score >= self.max_win {
                            self.game_end = Some(PlayerID::Zero);
                        }
                        Kekka::REnd(Some(PlayerID::Zero))
                    }
                    std::cmp::Ordering::Greater => {
                        self.board.p1_score += 1;
                        if self.board.p1_score >= self.max_win {
                            self.game_end = Some(PlayerID::One);
                        }
                        Kekka::REnd(Some(PlayerID::One))
                    }
                    std::cmp::Ordering::Equal => Kekka::REnd(None),
                }
            }
        }
    }
    fn round_end_attack(&mut self, id: PlayerID) -> Kekka {
        *self.board.score_mut(id) += 1;
        if self.board.score(id) >= self.max_win {
            self.game_end = Some(id);
        }
        Kekka::REnd(Some(id))
    }
    fn round_end_tumi(&mut self, id: PlayerID) -> Kekka {
        *self.board.score_mut(id) += 1;
        if self.board.score(id) >= self.max_win {
            self.game_end = Some(id);
        }
        Kekka::REnd(Some(id))
    }
    pub fn play_movement(
        &mut self,
        id: PlayerID,
        movement: &PlayMovement,
    ) -> Result<Kekka, &'static str> {
        match self.player(id).card_pos(movement.play_card()) {
            Some(index) => {
                if self
                    .player(id)
                    .can_move(&self.board, movement.play_card(), movement.direction())
                {
                    self.player_mut(id).remove_card(index);
                    self.move_player(id, movement.direction(), movement.play_card());
                    // 相手の詰み確認
                    if !self.player(id.opposite()).can_actions(&self.board) {
                        return Ok(self.round_end_tumi(id));
                    }
                    // 回収作業
                    while self.player(id).hand().len() < 5 {
                        match self.board.yamafuda.pop() {
                            Some(card) => {
                                self.player_mut(id).push_card(card);
                            }
                            None => return Ok(self.round_end_yamafuda()),
                        }
                        if self.board.yamafuda.is_empty() {
                            return Ok(self.round_end_yamafuda());
                        }
                    }
                    Ok(Kekka::Continue)
                } else {
                    Err("そちらへは動けません!")
                }
            }
            None => Err("そのカードは持ってません!"),
        }
    }
    pub fn play_attack(
        &mut self,
        id: PlayerID,
        attack: &PlayAttack,
    ) -> Result<Kekka, &'static str> {
        let indicies = self.player(id).card_positions(attack.play_card());
        if indicies.len() as u8 >= attack.num_of_card()
            && self.player(id).can_attack(&self.board, attack.play_card())
        {
            let indicies_opposite = self
                .player(id.opposite())
                .card_positions(attack.play_card());
            if indicies_opposite.len() >= indicies.len() {
                indicies_opposite
                    .into_iter()
                    .rev()
                    .take(indicies.len())
                    .for_each(|index| self.player_mut(id.opposite()).remove_card(index));
                indicies
                    .into_iter()
                    .rev()
                    .for_each(|index| self.player_mut(id).remove_card(index));

                // 相手の詰み確認
                if !self.player(id.opposite()).can_actions(&self.board) {
                    return Ok(Kekka::REnd(Some(id)));
                }
                // 回収作業
                while self.player(id).hand().len() < 5 {
                    match self.board.yamafuda.pop() {
                        Some(card) => {
                            self.player_mut(id).push_card(card);
                        }
                        None => return Ok(self.round_end_yamafuda()),
                    }
                    if self.board.yamafuda.is_empty() {
                        return Ok(self.round_end_yamafuda());
                    }
                }
                Ok(Kekka::Continue)
            } else {
                Ok(self.round_end_attack(id))
            }
        } else {
            Err("攻撃はとどかないか、そんなに枚数持っていません！")
        }
    }
}
