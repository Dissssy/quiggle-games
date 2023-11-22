use qg_shared::{
    anyhow::{anyhow, Result},
    colored::Colorize,
    rand::Rng as _,
    serenity::all::*,
};

use serde::{Deserialize, Serialize};

pub fn command() -> SlidingPuzzle {
    SlidingPuzzle
}

pub struct SlidingPuzzle;

#[qg_shared::async_trait]
impl qg_shared::Command for SlidingPuzzle {
    // fn register<'a>(&self, builder: &'a mut CreateApplicationCommand) -> &'a mut CreateApplicationCommand {
    //     let info = self.get_command_info();
    //     builder.name(info.name).description(info.description)
    // }

    fn get_command_info(&self) -> qg_shared::CommandInfo {
        qg_shared::CommandInfo {
            name: String::from("slidingpuzzle"),
            description: String::from("Play a game of sliding puzzle!"),
            options: vec![].into(),
        }
    }

    async fn application_command(&mut self, ctx: &Context, interaction: &mut CommandInteraction, _: &mut qg_shared::OptTrans<'_>) -> Result<()> {
        let game = Game {
            player: Player { id: interaction.user.id },
            gamestate: State::AwaitingApproval(Awaiting { inviter: interaction.user.id }),
            start_time: None,
            moves: 0,
            difficulty: Difficulty::Easy,
            size: Size::Three,
        };

        game.send(ctx, interaction).await?;
        Ok(())
    }

    async fn message_component(&mut self, ctx: &Context, interaction: &mut ComponentInteraction, db: &mut qg_shared::OptTrans<'_>) -> Result<()> {
        let action = match Action::from_custom_id(&interaction.data.custom_id) {
            Some(action) => action,
            None => return Err(qg_shared::anyhow::anyhow!("Invalid action id")),
        };

        // get first line of message content, strip the ``` prefix and deserialize
        let mut game = {
            let mut lines = interaction.message.content.lines();
            let game = lines.next().ok_or(qg_shared::anyhow::anyhow!("No game data found"))?;
            let game = game.strip_prefix("```").ok_or(qg_shared::anyhow::anyhow!("No game data found"))?;
            qg_shared::deserialize::<Game>(game)?
        };

        game.do_action(ctx, interaction, action, db).await?;

        Ok(())
    }
}

pub enum Action {
    SetDifficulty(Difficulty),
    SetSize(Size),
    Start,
    MoveTile(usize, usize),
    InvalidMove(usize),
}

impl Action {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SetDifficulty(_) => "SetDifficulty",
            Self::SetSize(_) => "SetSize",
            Self::Start => "Start",
            Self::MoveTile(_, _) => "MoveTile",
            Self::InvalidMove(_) => "InvalidMove",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Easy => "Easy",
            Self::Medium => "Medium",
            Self::Hard => "Hard",
        }
    }

    pub fn name_with_ansi(&self) -> String {
        match self {
            Self::Easy => "Easy".green().to_string(),
            Self::Medium => "Medium".blue().to_string(),
            Self::Hard => "Hard".red().to_string(),
        }
    }

    fn button_style(&self) -> ButtonStyle {
        match self {
            Self::Easy => ButtonStyle::Success,
            Self::Medium => ButtonStyle::Primary,
            Self::Hard => ButtonStyle::Danger,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Three,
    Four,
    Five,
}

impl Size {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Three => "3x3",
            Self::Four => "4x4",
            Self::Five => "5x5",
        }
    }

    pub fn name_with_ansi(&self) -> String {
        match self {
            Self::Three => "3x3".green().to_string(),
            Self::Four => "4x4".blue().to_string(),
            Self::Five => "5x5".red().to_string(),
        }
    }

    fn numeral(&self) -> usize {
        match self {
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
        }
    }
}

impl Action {
    pub fn from_custom_id(custom_id: &str) -> Option<Self> {
        let mut split = custom_id.split(':').skip(1);
        let action = split.next()?;
        match action {
            "SetDifficulty" => {
                let difficulty = split.next()?;
                Some(Self::SetDifficulty(match difficulty {
                    "Easy" => Difficulty::Easy,
                    "Medium" => Difficulty::Medium,
                    "Hard" => Difficulty::Hard,
                    _ => return None,
                }))
            }
            "SetSize" => {
                let size = split.next()?;
                Some(Self::SetSize(match size {
                    "3x3" => Size::Three,
                    "4x4" => Size::Four,
                    "5x5" => Size::Five,
                    _ => return None,
                }))
            }
            "Start" => Some(Self::Start),
            "MoveTile" => Some(Self::MoveTile(
                {
                    let s = split.next()?;
                    s.parse::<usize>().ok()?
                },
                {
                    let s = split.next()?;
                    s.parse::<usize>().ok()?
                },
            )),
            "InvalidMove" => Some(Self::InvalidMove({
                let s = split.next()?;
                s.parse::<usize>().ok()?
            })),
            _ => None,
        }
    }
    pub fn to_custom_id(&self, command: &str) -> String {
        match self {
            Self::SetDifficulty(difficulty) => format!("{}:SetDifficulty:{}", command, difficulty.name()),
            Self::SetSize(size) => format!("{}:SetSize:{}", command, size.name()),
            Self::Start => format!("{}:Start", command),
            Self::MoveTile(s, f) => format!("{}:MoveTile:{}:{}", command, s, f),
            Self::InvalidMove(i) => format!("{}:InvalidMove:{}", command, i),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    player: Player,
    gamestate: State,
    start_time: Option<u64>,
    moves: u64,
    difficulty: Difficulty,
    size: Size,
}

impl Game {
    pub async fn do_action(&mut self, ctx: &Context, interaction: &mut ComponentInteraction, action: Action, db: &mut qg_shared::OptTrans<'_>) -> Result<()> {
        let shitstarted = qg_shared::current_time()?;
        let mut updatetime = false;
        match self.gamestate {
            State::AwaitingApproval(ref u) => {
                if interaction.user.id != u.inviter {
                    return Err(anyhow!("You are not the player"));
                }
                match action {
                    Action::Start => {
                        self.gamestate = State::InProgress(InProgress {
                            board: Board::new(self.size, self.difficulty),
                        });
                    }
                    Action::SetDifficulty(difficulty) => {
                        self.difficulty = difficulty;
                    }
                    Action::SetSize(size) => {
                        self.size = size;
                    }
                    _ => {
                        return Err(anyhow!("Invalid action"));
                    }
                }
            }
            State::InProgress(ref mut game) => {
                if interaction.user.id != self.player.id {
                    return Err(anyhow!("You are not the player"));
                }
                match action {
                    Action::MoveTile(s, f) => {
                        match self.start_time {
                            None => {
                                self.start_time = Some(qg_shared::current_time()?);
                            }
                            Some(_) => {}
                        };
                        game.board.swap_checked(s, f)?;
                        self.moves += 1;
                        if game.board.check_winner() {
                            self.gamestate = State::Finished(WonGame {
                                winner: Outcome {
                                    elapsed: qg_shared::current_time()? - self.start_time.unwrap_or(qg_shared::current_time()?),
                                    moves: self.moves,
                                    player: self.player,
                                },
                                board: game.board.clone(),
                            });
                            if let Some(db) = db {
                                // ensure the user is in the database first
                                let user = qg_shared::db::User::get_or_create(ctx, &self.player.id, db).await?;
                                // create an entry for the user in the slidingpuzzle table
                                qg_shared::db::SlidingPuzzle::create(
                                    user.id as i32,
                                    self.difficulty as i32,
                                    self.size as i32,
                                    self.moves as i32,
                                    (qg_shared::current_time()? - self.start_time.unwrap_or(qg_shared::current_time()?)) as i32,
                                    db,
                                )
                                .await?;
                            }
                        } else {
                            updatetime = true;
                        }
                    }
                    Action::InvalidMove(_) => {
                        // do nothing
                    }
                    _ => {
                        return Err(anyhow!("Invalid action"));
                    }
                }
            }
            _ => {
                return Err(anyhow!("Invalid action: {}", action.name()));
            }
        }

        if updatetime {
            // add the difference between now and when the function started to the start time
            let mut now = qg_shared::current_time()?;
            now -= shitstarted;
            if let Some(start_time) = self.start_time {
                self.start_time = Some(start_time + now);
            }
        }

        self.render(ctx, interaction, self.start_time).await.map_err(|e| {
            qg_shared::log::error!("Error rendering game: {}", e);
            e
        })?;

        if updatetime {
            // add the difference between now and when the function started to the start time
            let mut now = qg_shared::current_time()?;
            now -= shitstarted;
            if let Some(start_time) = self.start_time {
                self.start_time = Some(start_time + now);
            }
        }

        Ok(())
    }

    async fn render(&self, ctx: &Context, interaction: &mut ComponentInteraction, start_time: Option<u64>) -> Result<()> {
        match &self.gamestate {
            State::AwaitingApproval(ref u) => {
                let mut content = self.title_card()?;
                content.push_str(u.challenge_message().as_str());
                interaction.defer(&ctx.http).await?;
                interaction
                    .edit_response(&ctx.http, {
                        EditInteractionResponse::default().content(content).components({
                            vec![
                                CreateActionRow::Buttons({
                                    let mut buttons = Vec::new();
                                    for size in &[Size::Three, Size::Four, Size::Five] {
                                        buttons.push(
                                            CreateButton::new(Action::SetSize(*size).to_custom_id("slidingpuzzle"))
                                                .style(if *size == self.size { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                                                .label(size.name())
                                                .disabled(*size == self.size),
                                        );
                                    }
                                    buttons
                                }),
                                CreateActionRow::Buttons({
                                    let mut buttons = Vec::new();
                                    for difficulty in &[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
                                        buttons.push(
                                            CreateButton::new(Action::SetDifficulty(*difficulty).to_custom_id("slidingpuzzle"))
                                                .style(if *difficulty == self.difficulty { difficulty.button_style() } else { ButtonStyle::Secondary })
                                                .label(difficulty.name())
                                                .disabled(*difficulty == self.difficulty),
                                        );
                                    }
                                    buttons
                                }),
                                CreateActionRow::Buttons(vec![CreateButton::new(Action::Start.to_custom_id("slidingpuzzle")).style(ButtonStyle::Success).label("Start")]),
                            ]
                            // c.create_action_row(|a| {
                            //     // Size Buttons, the selected one is disabled and Primary, the others are Secondary
                            //     for size in &[Size::Three, Size::Four, Size::Five] {
                            //         a.create_button(|b| {
                            //             b.style(if *size == self.size { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                            //                 .label(size.name())
                            //                 .custom_id(Action::SetSize(*size).to_custom_id("slidingpuzzle"))
                            //                 .disabled(*size == self.size)
                            //         });
                            //     }
                            //     a
                            // })
                            // .create_action_row(|a| {
                            //     // Difficulty Buttons, the selected one is disabled and depends on the difficulty, the others are Secondary
                            //     for difficulty in &[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
                            //         a.create_button(|b| {
                            //             b.style(if *difficulty == self.difficulty { difficulty.button_style() } else { ButtonStyle::Secondary })
                            //                 .label(difficulty.name())
                            //                 .custom_id(Action::SetDifficulty(*difficulty).to_custom_id("slidingpuzzle"))
                            //                 .disabled(*difficulty == self.difficulty)
                            //         });
                            //     }
                            //     a
                            // })
                            // .create_action_row(|a| {
                            //     // Start Button
                            //     a.create_button(|b| b.style(ButtonStyle::Success).label("Start").custom_id(Action::Start.to_custom_id("slidingpuzzle")));
                            //     a
                            // })
                        })
                    })
                    .await?;
            }
            State::InProgress(game) => {
                let mut content = self.title_card()?;
                content.push_str(
                    format!(
                        "```ansi\nTime: {}\nMoves: {}\n```",
                        match start_time {
                            None => "0s (paused)".to_string().red(),
                            Some(start_time) => qg_shared::format_duration(qg_shared::current_time()? - start_time).blue(),
                        },
                        self.moves.to_string().red(),
                    )
                    .as_str(),
                );
                interaction.defer(&ctx.http).await?;
                interaction
                    .edit_response(&ctx.http, {
                        EditInteractionResponse::default().content(content).components({
                            let mut rows = Vec::new();
                            for x in 0..self.size.numeral() {
                                rows.push(CreateActionRow::Buttons({
                                    let mut buttons = Vec::new();
                                    for y in 0..self.size.numeral() {
                                        buttons.push(game.board.button_for(y, x));
                                    }
                                    buttons
                                }))
                                // c.create_action_row(|a| {
                                //     for y in 0..self.size.numeral() {
                                //         a.create_button(|b| {
                                //             game.board.button_for(y, x, b);
                                //             b
                                //         });
                                //     }
                                //     a
                                // });
                            }
                            rows
                        })
                    })
                    .await?;
            }
            State::Finished(won_game) => {
                let mut content = self.title_card()?;
                content.push_str(
                    format!(
                        "{} has won!\n```ansi\nSize: {}\nDifficulty: {}\nTime: {}\nMoves: {}\n```",
                        won_game.winner.player.id.mention(),
                        self.size.name_with_ansi(),
                        self.difficulty.name_with_ansi(),
                        qg_shared::format_duration(won_game.winner.elapsed).blue(),
                        won_game.winner.moves.to_string().red(),
                    )
                    .as_str(),
                );
                interaction.defer(&ctx.http).await?;
                interaction
                    .edit_response(&ctx.http, {
                        EditInteractionResponse::default().content(content).components(vec![])
                        // d.content(content).components(|c| {
                        //     // no components, its a sliding puzzle lol.
                        //     c
                        // })
                    })
                    .await?;
            }
        }
        Ok(())
    }
    async fn send(&self, ctx: &Context, interaction: &mut CommandInteraction) -> Result<()> {
        match self.gamestate {
            State::AwaitingApproval(ref u) => {
                let mut content = self.title_card()?;
                content.push_str(u.challenge_message().as_str());
                interaction
                    .create_response(&ctx.http, {
                        CreateInteractionResponse::Message(CreateInteractionResponseMessage::default().content(content).components({
                            vec![
                                CreateActionRow::Buttons({
                                    let mut buttons = Vec::new();
                                    for size in &[Size::Three, Size::Four, Size::Five] {
                                        buttons.push(
                                            CreateButton::new(Action::SetSize(*size).to_custom_id("slidingpuzzle"))
                                                .style(if *size == self.size { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                                                .label(size.name())
                                                .disabled(*size == self.size),
                                        );
                                    }
                                    buttons
                                }),
                                CreateActionRow::Buttons({
                                    let mut buttons = Vec::new();
                                    for difficulty in &[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
                                        buttons.push(
                                            CreateButton::new(Action::SetDifficulty(*difficulty).to_custom_id("slidingpuzzle"))
                                                .style(if *difficulty == self.difficulty { difficulty.button_style() } else { ButtonStyle::Secondary })
                                                .label(difficulty.name())
                                                .disabled(*difficulty == self.difficulty),
                                        );
                                    }
                                    buttons
                                }),
                                CreateActionRow::Buttons(vec![CreateButton::new(Action::Start.to_custom_id("slidingpuzzle")).style(ButtonStyle::Success).label("Start")]),
                            ]
                            // c.create_action_row(|a| {
                            //     // Size Buttons, the selected one is disabled and Primary, the others are Secondary
                            //     for size in &[Size::Three, Size::Four, Size::Five] {
                            //         a.create_button(|b| {
                            //             b.style(if *size == self.size { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                            //                 .label(size.name())
                            //                 .custom_id(Action::SetSize(*size).to_custom_id("slidingpuzzle"))
                            //                 .disabled(*size == self.size)
                            //         });
                            //     }
                            //     a
                            // })
                            // .create_action_row(|a| {
                            //     // Difficulty Buttons, the selected one is disabled and depends on which difficulty, the others are Secondary
                            //     for difficulty in &[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
                            //         a.create_button(|b| {
                            //             b.style(if *difficulty == self.difficulty { difficulty.button_style() } else { ButtonStyle::Secondary })
                            //                 .label(difficulty.name())
                            //                 .custom_id(Action::SetDifficulty(*difficulty).to_custom_id("slidingpuzzle"))
                            //                 .disabled(*difficulty == self.difficulty)
                            //         });
                            //     }
                            //     a
                            // })
                            // .create_action_row(|a| {
                            //     // Start Button
                            //     a.create_button(|b| b.style(ButtonStyle::Success).label("Start").custom_id(Action::Start.to_custom_id("slidingpuzzle")));
                            //     a
                            // })
                        }))
                    })
                    .await?;
            }
            _ => {
                return Err(qg_shared::anyhow::anyhow!("Invalid game state"));
            }
        }
        Ok(())
    }
    fn title_card(&self) -> Result<String> {
        Ok(format!("```{}\nSliding Puzzle\n```", qg_shared::serialize(&self)?.replace('\n', "")))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Space {
    Empty,
    Value(u8),
}

impl Space {
    fn button_text(&self, max_over_nine: bool) -> String {
        match self {
            Space::Empty => {
                if max_over_nine {
                    String::from("⬛⬛")
                } else {
                    String::from("⬛")
                }
            }
            Space::Value(v) => match v {
                0..=9 => {
                    if max_over_nine {
                        format!("⬛{}", to_emoji(*v))
                    } else {
                        to_emoji(*v).to_string()
                    }
                }
                _ => to_emoji(*v).to_string(),
            },
        }
    }
}

fn to_emoji(n: u8) -> &'static str {
    match n {
        0 => "0️⃣",
        1 => "1️⃣",
        2 => "2️⃣",
        3 => "3️⃣",
        4 => "4️⃣",
        5 => "5️⃣",
        6 => "6️⃣",
        7 => "7️⃣",
        8 => "8️⃣",
        9 => "9️⃣",
        10 => "1️⃣0️⃣",
        11 => "1️⃣1️⃣",
        12 => "1️⃣2️⃣",
        13 => "1️⃣3️⃣",
        14 => "1️⃣4️⃣",
        15 => "1️⃣5️⃣",
        16 => "1️⃣6️⃣",
        17 => "1️⃣7️⃣",
        18 => "1️⃣8️⃣",
        19 => "1️⃣9️⃣",
        20 => "2️⃣0️⃣",
        21 => "2️⃣1️⃣",
        22 => "2️⃣2️⃣",
        23 => "2️⃣3️⃣",
        24 => "2️⃣4️⃣",
        25 => "2️⃣5️⃣",
        _ => unreachable!(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum State {
    AwaitingApproval(Awaiting),
    InProgress(InProgress),
    Finished(WonGame),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Awaiting {
    inviter: UserId,
}

impl Awaiting {
    fn challenge_message(&self) -> String {
        format!("{}, Choose your size and difficulty!", self.inviter.mention())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InProgress {
    board: Board,
}

impl InProgress {
    // fn make_move(&mut self, x: usize, y: usize, piece: Space) -> Result<()> {
    //     if x > 2 || y > 2 {
    //         return Err(qg_shared::anyhow::anyhow!("Invalid move, out of bounds"));
    //     }
    //     if self.board.spaces[x][y] != Space::Empty {
    //         return Err(qg_shared::anyhow::anyhow!("Invalid move, space already occupied"));
    //     }
    //     self.board.spaces[x][y] = piece;
    //     Ok(())
    // }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WonGame {
    winner: Outcome,
    board: Board,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Board {
    spaces: Vec<Space>,
    size: Size,
}

impl Board {
    pub fn new(size: Size, difficulty: Difficulty) -> Self {
        let mut board = match size {
            Size::Three => Self { spaces: vec![Space::Empty; 9], size },
            Size::Four => Self { spaces: vec![Space::Empty; 16], size },
            Size::Five => Self { spaces: vec![Space::Empty; 25], size },
        };

        // random number between 0 and size * size - 1
        let mut rng = qg_shared::rand::thread_rng();
        let emptyspace = rng.gen_range(0..((size.numeral() * size.numeral()) - 1));

        for (i, s) in board.spaces.iter_mut().enumerate() {
            if i == emptyspace {
                *s = Space::Empty;
            } else {
                *s = Space::Value((i + 1) as u8);
            }
        }

        let multiplier = match size {
            Size::Three => 9,
            Size::Four => 16,
            Size::Five => 25,
        };
        // we need to shuffle the board in a way where we know it's solvable. so depending on the selected difficulty, we'll swap pieces a certain number of times
        let number_of_moves = match difficulty {
            Difficulty::Easy => 10 * multiplier,
            Difficulty::Medium => 50 * multiplier,
            Difficulty::Hard => 100 * multiplier,
        };

        for _ in 0..number_of_moves {
            let thismove = match rng.gen_range(0..4) {
                0 => Direction::Up,
                1 => Direction::Down,
                2 => Direction::Left,
                3 => Direction::Right,
                _ => unreachable!(),
            };
            board.move_empty_tile_raw(thismove);
        }

        board
    }

    fn move_empty_tile_raw(&mut self, mut thismove: Direction) {
        // find empty tile
        let empty_tile = self.spaces.iter().enumerate().find_map(|(i, s)| if *s == Space::Empty { Some(i) } else { None }).unwrap();

        let size = self.size.numeral();

        // ensure we're not moving out of bounds, if we are, cycle the move clockwise until we're not
        let (x, y) = loop {
            let (x, y) = match thismove {
                Direction::Up => (empty_tile % size, empty_tile / size),
                Direction::Down => (empty_tile % size, empty_tile / size),
                Direction::Left => (empty_tile % size, empty_tile / size),
                Direction::Right => (empty_tile % size, empty_tile / size),
            };
            if x == 0 && thismove == Direction::Left {
                thismove = Direction::Up;
            } else if x == { size - 1 } && thismove == Direction::Right {
                thismove = Direction::Down;
            } else if y == 0 && thismove == Direction::Up {
                thismove = Direction::Right;
            } else if y == { size - 1 } && thismove == Direction::Down {
                thismove = Direction::Left;
            } else {
                break (x, y);
            }
        };

        // swap the empty tile with the tile we're moving to
        let (x, y) = match thismove {
            Direction::Up => (x, y - 1),
            Direction::Down => (x, y + 1),
            Direction::Left => (x - 1, y),
            Direction::Right => (x + 1, y),
        };

        let i = x + y * size;

        self.spaces.swap(empty_tile, i);
    }

    fn button_for(&self, x: usize, y: usize) -> CreateButton {
        // convert x and y to a single index
        let i = x + (y * self.size.numeral());
        let p = self.spaces[i];

        let direction = self.direction_towards_empty_tile(x as isize, y as isize);
        let empty_tile_index = self.spaces.iter().enumerate().find_map(|(i, s)| if *s == Space::Empty { Some(i) } else { None }).unwrap();

        let over_nine = match self.size {
            Size::Three => false,
            Size::Four => true,
            Size::Five => true,
        };

        match (p, direction) {
            // Empty space, disabled, Secondary style
            (Space::Empty, _) => {
                let b = CreateButton::new(Action::InvalidMove(i).to_custom_id("slidingpuzzle"));
                b.disabled(true).style(ButtonStyle::Secondary).label(p.button_text(over_nine))
            }
            // Value space, disabled, Secondary style unless in correct position, then Success style
            (Space::Value(v), None) => {
                let b = CreateButton::new(Action::InvalidMove(i).to_custom_id("slidingpuzzle"));
                b.style(if v == (i + 1) as u8 { ButtonStyle::Success } else { ButtonStyle::Secondary }).label(p.button_text(over_nine))
                // .disabled(true)
            }
            // Value space, enabled if adjacent to empty space in a cardinal direction, Primary style unless in correct position, then Success style
            (Space::Value(v), Some(_)) => {
                let b = CreateButton::new(Action::MoveTile(i, empty_tile_index).to_custom_id("slidingpuzzle"));
                b.style(if v == (i + 1) as u8 { ButtonStyle::Success } else { ButtonStyle::Primary }).label(p.button_text(over_nine))
            }
        }
    }

    fn direction_towards_empty_tile(&self, x: isize, y: isize) -> Option<Direction> {
        let size = self.size.numeral() as isize;
        let empty_tile = self.spaces.iter().enumerate().find_map(|(i, s)| if *s == Space::Empty { Some(i) } else { None }).unwrap() as isize;
        let (empty_x, empty_y) = (empty_tile % size, empty_tile / size);
        if x == empty_x {
            if y == empty_y - 1 {
                if empty_y == 0 {
                    None
                } else {
                    Some(Direction::Up)
                }
            } else if y == empty_y + 1 {
                if empty_y == size - 1 {
                    None
                } else {
                    Some(Direction::Down)
                }
            } else {
                None
            }
        } else if y == empty_y {
            if x == empty_x - 1 {
                if empty_x == 0 {
                    None
                } else {
                    Some(Direction::Left)
                }
            } else if x == empty_x + 1 {
                if empty_x == size - 1 {
                    None
                } else {
                    Some(Direction::Right)
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn swap_checked(&mut self, s: usize, f: usize) -> Result<()> {
        let size = self.size.numeral();
        if s >= size * size || f >= size * size {
            return Err(anyhow!("Invalid move, out of bounds"));
        }
        // we need either s or f to be the empty tile
        if self.spaces[s] != Space::Empty && self.spaces[f] != Space::Empty {
            return Err(anyhow!("Invalid move, neither tile is the empty tile"));
        }
        // we need s and f to be adjacent
        let (sx, sy) = (s % size, s / size);
        let (fx, fy) = (f % size, f / size);
        if sx != fx && sy != fy {
            return Err(anyhow!("Invalid move, tiles are not adjacent"));
        }
        // we need s and f to be in the same row or column
        if sx != fx && sy != fy {
            return Err(anyhow!("Invalid move, tiles are not in the same row or column"));
        }
        // everything checks out, swap the tiles
        self.spaces.swap(s, f);
        Ok(())
    }

    fn check_winner(&self) -> bool {
        // check that every space except Empty is in the correct position
        for (i, s) in self.spaces.iter().enumerate() {
            if *s != Space::Empty && *s != Space::Value((i + 1) as u8) {
                return false;
            }
        }
        // if we got here, every space is in the correct position
        true
    }

    // pub fn button_for(&self, x: usize, y: usize, button: &mut qg_shared::serenity::builder::CreateButton) {
    //     let p = self.spaces[x][y];
    //     button.label(format!("{}", p)).custom_id(Action::Place(x, y).to_custom_id("slidingpuzzle"));
    //     if p != Space::Empty {
    //         button.disabled(true);
    //     }
    //     button.style(p.button_style());
    // }

    // fn check_winner(&self, players: &CycleVec<Player>) -> Option<Outcome> {
    //     // check rows
    //     for row in self.spaces.iter() {
    //         if row.iter().all(|s| *s == Space::X) {
    //             return Some(Outcome::Win(players.all().find(|p| p.piece == Space::X).copied()?));
    //         }
    //         if row.iter().all(|s| *s == Space::O) {
    //             return Some(Outcome::Win(players.all().find(|p| p.piece == Space::O).copied()?));
    //         }
    //     }
    //     // check columns
    //     for x in 0..3 {
    //         if self.spaces.iter().all(|row| row[x] == Space::X) {
    //             return Some(Outcome::Win(players.all().find(|p| p.piece == Space::X).copied()?));
    //         }
    //         if self.spaces.iter().all(|row| row[x] == Space::O) {
    //             return Some(Outcome::Win(players.all().find(|p| p.piece == Space::O).copied()?));
    //         }
    //     }
    //     // check diagonals
    //     if self.spaces[0][0] == Space::X && self.spaces[1][1] == Space::X && self.spaces[2][2] == Space::X {
    //         return Some(Outcome::Win(players.all().find(|p| p.piece == Space::X).copied()?));
    //     }
    //     if self.spaces[0][0] == Space::O && self.spaces[1][1] == Space::O && self.spaces[2][2] == Space::O {
    //         return Some(Outcome::Win(players.all().find(|p| p.piece == Space::O).copied()?));
    //     }
    //     if self.spaces[0][2] == Space::X && self.spaces[1][1] == Space::X && self.spaces[2][0] == Space::X {
    //         return Some(Outcome::Win(players.all().find(|p| p.piece == Space::X).copied()?));
    //     }
    //     if self.spaces[0][2] == Space::O && self.spaces[1][1] == Space::O && self.spaces[2][0] == Space::O {
    //         return Some(Outcome::Win(players.all().find(|p| p.piece == Space::O).copied()?));
    //     }
    //     // check tie
    //     if self.spaces.iter().flatten().all(|s| *s != Space::Empty) {
    //         return Some(Outcome::Tie);
    //     }
    //     None
    // }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Outcome {
    elapsed: u64,
    moves: u64,
    player: Player,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Player {
    id: UserId,
}
