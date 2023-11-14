use qg_shared::{
    anyhow::{anyhow, Result},
    colored::Colorize,
    serenity::{
        builder::CreateApplicationCommand,
        client::Context,
        model::{
            application::{
                component::ButtonStyle,
                interaction::{
                    application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
                    message_component::MessageComponentInteraction,
                },
            },
            id::UserId,
            mention::Mentionable,
        },
    },
    CycleVec,
};

use serde::{Deserialize, Serialize};

pub fn command() -> UltimateTicTacToe {
    UltimateTicTacToe
}

pub struct UltimateTicTacToe;

#[qg_shared::async_trait]
impl qg_shared::Command for UltimateTicTacToe {
    fn register<'a>(&self, builder: &'a mut CreateApplicationCommand) -> &'a mut CreateApplicationCommand {
        let info = self.get_command_info();
        builder.name(info.name).description(info.description).create_option(|o| {
            for option in info.options {
                o.name(option.name).description(option.description).kind(option.option_type.into()).required(option.required);
            }
            o
        })
    }

    fn get_command_info(&self) -> qg_shared::CommandInfo {
        qg_shared::CommandInfo {
            name: String::from("ultimatetictactoe"),
            description: String::from("Play a game of Ultimate Tic Tac Toe"),
            options: vec![qg_shared::CommandOption {
                name: String::from("opponent"),
                description: String::from("The opponent to play against"),
                option_type: qg_shared::CommandOptionType::User,
                required: true,
            }]
            .into(),
        }
    }

    async fn application_command(&mut self, ctx: &Context, interaction: &mut ApplicationCommandInteraction) -> Result<()> {
        let mut players = vec![Player {
            id: interaction.user.id,
            piece: Space::X,
        }];
        let other;
        players.push({
            match interaction.data.options.first().ok_or(qg_shared::anyhow::anyhow!("No opponent specified"))?.resolved.as_ref() {
                Some(CommandDataOptionValue::User(user, _m)) => {
                    other = user.clone();
                    Player { id: other.id, piece: Space::O }
                }
                _ => return Err(qg_shared::anyhow::anyhow!("No opponent specified")),
            }
        });
        #[cfg(not(feature = "allow-self-play"))]
        {
            let individuals = {
                let mut individuals = players.iter().map(|player| player.id).collect::<Vec<UserId>>();
                individuals.sort();
                individuals.dedup();
                individuals
            };

            if individuals.len() != players.len() {
                return Err(qg_shared::anyhow::anyhow!("Playing with yourself is not pemitted"));
            }
        }

        let game = Game {
            players: CycleVec::new(players),
            gamestate: State::AwaitingApproval(Awaiting {
                inviter: interaction.user.id,
                invitee: other.id,
            }),
            last_time: qg_shared::current_time()?,
        };

        game.send(ctx, interaction).await?;
        Ok(())
    }

    async fn message_component(&mut self, ctx: &Context, interaction: &mut MessageComponentInteraction) -> Result<()> {
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

        game.do_action(ctx, interaction, action).await?;

        Ok(())
    }
}

pub enum Action {
    Accept,
    Decline,
    Place(usize, usize),
}

impl Action {
    pub fn from_custom_id(custom_id: &str) -> Option<Self> {
        let mut split = custom_id.split(':').skip(1);
        let action = split.next()?;
        match action {
            "Accept" => {
                if split.next().is_some() {
                    return None;
                }
                Some(Self::Accept)
            }
            "Decline" => {
                if split.next().is_some() {
                    return None;
                }
                Some(Self::Decline)
            }
            "Place" => {
                let x = split.next()?.parse().ok()?;
                let y = split.next()?.parse().ok()?;
                if split.next().is_some() {
                    return None;
                }
                Some(Self::Place(x, y))
            }
            _ => None,
        }
    }
    pub fn to_custom_id(&self, command: &str) -> String {
        format!("{}:{}", command, self.name())
    }
    pub fn name(&self) -> String {
        match self {
            Self::Accept => String::from("Accept"),
            Self::Decline => String::from("Decline"),
            Self::Place(x, y) => format!("Place:{}:{}", x, y),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    players: CycleVec<Player>,
    gamestate: State,
    last_time: u64,
}

impl Game {
    pub async fn do_action(&mut self, ctx: &Context, interaction: &mut MessageComponentInteraction, action: Action) -> Result<()> {
        match self.gamestate {
            State::AwaitingApproval(ref u) => {
                if interaction.user.id != u.invitee {
                    interaction
                        .create_interaction_response(&ctx.http, |f| f.interaction_response_data(|d| d.content("You are not the invitee").ephemeral(true)))
                        .await?;
                    return Ok(());
                }
                match action {
                    Action::Accept => {
                        self.gamestate = State::InProgress(InProgress::new());
                        interaction
                            .create_interaction_response(&ctx.http, |f| {
                                f.kind(qg_shared::serenity::model::application::interaction::InteractionResponseType::DeferredUpdateMessage)
                            })
                            .await?;
                        let pid = self.players.current().ok_or(anyhow!("Player not found"))?.id.0;
                        if pid != interaction.user.id.0 {
                            let now = qg_shared::current_time()?;
                            if now.saturating_sub(self.last_time) > 0 {
                                ctx.http
                                    .get_user(pid)
                                    .await?
                                    .create_dm_channel(&ctx.http)
                                    .await?
                                    .send_message(&ctx.http, |m| m.content(format!("It is your turn in {}", interaction.message.link())))
                                    .await?;
                            }
                            self.last_time = now;
                        }
                    }
                    Action::Decline => {
                        self.gamestate = State::Cancelled("Declined".into());
                        interaction
                            .create_interaction_response(&ctx.http, |f| {
                                f.kind(qg_shared::serenity::model::application::interaction::InteractionResponseType::DeferredUpdateMessage)
                            })
                            .await?;
                    }
                    _ => {
                        interaction
                            .create_interaction_response(&ctx.http, |f| f.interaction_response_data(|d| d.content(format!("Invalid action: {}", action.name())).ephemeral(true)))
                            .await?;
                    }
                }
            }
            State::InProgress(ref mut game) => {
                if self.players.current().map(|s| s.id) != Some(interaction.user.id) {
                    interaction
                        .create_interaction_response(&ctx.http, |f| f.interaction_response_data(|d| d.content("It is not your turn").ephemeral(true)))
                        .await?;
                    return Ok(());
                }
                match action {
                    Action::Place(x, y) => {
                        let next_player = match game.make_move(x, y, self.players.current().ok_or(anyhow!("Player not found"))?.piece, &self.players) {
                            Ok(b) => b,
                            Err(e) => {
                                interaction
                                    .create_interaction_response(&ctx.http, |f| f.interaction_response_data(|d| d.content(format!("Invalid move: {}", e)).ephemeral(true)))
                                    .await?;
                                return Ok(());
                            }
                        };
                        if let Some(winner) = game.board.check_winner(&self.players) {
                            for player in self.players.all() {
                                ctx.http
                                    .get_user(player.id.0)
                                    .await?
                                    .create_dm_channel(&ctx.http)
                                    .await?
                                    .send_message(&ctx.http, |m| {
                                        m.content({
                                            if let Outcome::Win(p) = winner {
                                                format!("You {} in {}", if *player == p { "won" } else { "got your ass handed to you" }, interaction.message.link())
                                            } else {
                                                format!("You tied in {}", interaction.message.link())
                                            }
                                        })
                                    })
                                    .await?;
                            }

                            self.gamestate = State::Finished(WonGame { winner, board: game.board.clone() });
                        } else if next_player {
                            self.players.next_player();

                            let now = qg_shared::current_time()?;
                            if now.saturating_sub(self.last_time) > 60 {
                                ctx.http
                                    .get_user(self.players.current().ok_or(anyhow!("Player not found"))?.id.0)
                                    .await?
                                    .create_dm_channel(&ctx.http)
                                    .await?
                                    .send_message(&ctx.http, |m| m.content(format!("It is your turn in {}", interaction.message.link())))
                                    .await?;
                            }
                            self.last_time = now;
                        }

                        interaction
                            .create_interaction_response(&ctx, |f| f.kind(qg_shared::serenity::model::application::interaction::InteractionResponseType::DeferredUpdateMessage))
                            .await?
                    }
                    _ => {
                        interaction
                            .create_interaction_response(&ctx.http, |f| f.interaction_response_data(|d| d.content(format!("Invalid action: {}", action.name())).ephemeral(true)))
                            .await?;
                    }
                }
            }
            _ => {
                interaction
                    .create_interaction_response(&ctx.http, |f| f.interaction_response_data(|d| d.content(format!("Invalid action: {}", action.name())).ephemeral(true)))
                    .await?;
            }
        }

        self.render(ctx, interaction).await?;

        Ok(())
    }

    async fn render(&self, ctx: &Context, interaction: &mut MessageComponentInteraction) -> Result<()> {
        match &self.gamestate {
            State::Cancelled(reason) => {
                interaction
                    .edit_original_interaction_response(&ctx.http, |m| m.content(format!("Game cancelled: {}", reason)).components(|f| f))
                    .await?;
            }
            State::AwaitingApproval(ref u) => {
                let mut content = self.title_card()?;
                content.push_str(u.challenge_message().as_str());
                interaction
                    .edit_original_interaction_response(&ctx.http, |d| {
                        d.content(content).components(|c| {
                            c.create_action_row(|a| {
                                a.create_button(|b| b.style(ButtonStyle::Success).label("Accept").custom_id(Action::Accept.to_custom_id("ultimatetictactoe")))
                                    .create_button(|b| b.style(ButtonStyle::Danger).label("Decline").custom_id(Action::Decline.to_custom_id("ultimatetictactoe")))
                            })
                        })
                    })
                    .await?;
            }
            State::InProgress(game) => {
                let mut content = self.title_card()?;
                let current_player = self.players.current().ok_or(anyhow!("Player not found"))?;
                content.push_str(&format!("It is {}'s turn [{}]", current_player.id.mention(), current_player.piece));

                content.push_str(&game.board.string_map());

                interaction
                    .edit_original_interaction_response(&ctx.http, |d| {
                        d.content(content).components(|c| {
                            for x in 0..=2 {
                                c.create_action_row(|a| {
                                    for y in 0..=2 {
                                        a.create_button(|b| {
                                            game.board.button_for(x, y, b);
                                            b
                                        });
                                    }
                                    a
                                });
                            }
                            c
                        })
                    })
                    .await?;
            }
            State::Finished(won_game) => {
                let mut content = self.title_card()?;
                content.push_str(won_game.win_message().as_str());

                content.push_str(&won_game.board.raw_string_map());

                interaction
                    .edit_original_interaction_response(&ctx.http, |d| {
                        d.content(content).components(|c| {
                            // for x in 0..=2 {
                            //     c.create_action_row(|a| {
                            //         for y in 0..=2 {
                            //             a.create_button(|b| {
                            //                 won_game.board.button_for(x, y, b);
                            //                 b.disabled(true);
                            //                 b
                            //             });
                            //         }
                            //         a
                            //     });
                            // }
                            c
                        })
                    })
                    .await?;
            }
        }
        Ok(())
    }
    async fn send(&self, ctx: &Context, interaction: &mut ApplicationCommandInteraction) -> Result<()> {
        match self.gamestate {
            State::AwaitingApproval(ref u) => {
                let mut content = self.title_card()?;
                content.push_str(u.challenge_message().as_str());
                interaction
                    .create_interaction_response(&ctx.http, |f| {
                        f.interaction_response_data(|d| {
                            d.content(content).components(|c| {
                                c.create_action_row(|a| {
                                    a.create_button(|b| b.style(ButtonStyle::Success).label("Accept").custom_id(Action::Accept.to_custom_id("ultimatetictactoe")))
                                        .create_button(|b| b.style(ButtonStyle::Danger).label("Decline").custom_id(Action::Decline.to_custom_id("ultimatetictactoe")))
                                })
                            })
                        })
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
        Ok(format!("```{}\nUltimate Tic Tac Toe\n```", qg_shared::serialize(&self)?.replace('\n', "")))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MetaSpace {
    X(Board),
    O(Board),
    Tie(Board),
    Empty(Board),
}

impl Eq for MetaSpace {
    // Empty != anything else
    // Tie == Tie || X || O
    // X == X
    // O == O
}

impl PartialEq for MetaSpace {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MetaSpace::Empty(_), MetaSpace::Empty(_)) => true,
            (MetaSpace::Empty(_), _) => false,
            (_, MetaSpace::Empty(_)) => false,
            (MetaSpace::Tie(_), MetaSpace::Tie(_)) => true,
            (MetaSpace::Tie(_), MetaSpace::X(_)) => true,
            (MetaSpace::Tie(_), MetaSpace::O(_)) => true,
            (MetaSpace::X(_), MetaSpace::Tie(_)) => true,
            (MetaSpace::O(_), MetaSpace::Tie(_)) => true,
            (MetaSpace::X(_), MetaSpace::X(_)) => true,
            (MetaSpace::O(_), MetaSpace::O(_)) => true,
            _ => false,
        }
    }
}

impl MetaSpace {
    pub fn ignore_board(&self) -> Space {
        match self {
            MetaSpace::X(_) => Space::X,
            MetaSpace::O(_) => Space::O,
            MetaSpace::Tie(_) => Space::Empty,
            MetaSpace::Empty(_) => Space::Empty,
        }
    }
    pub fn ignore_space(&self) -> Board {
        match self {
            MetaSpace::X(b) => b.clone(),
            MetaSpace::O(b) => b.clone(),
            MetaSpace::Tie(b) => b.clone(),
            MetaSpace::Empty(b) => b.clone(),
        }
    }
    pub fn board_mut(&mut self) -> &mut Board {
        match self {
            MetaSpace::X(b) => b,
            MetaSpace::O(b) => b,
            MetaSpace::Tie(b) => b,
            MetaSpace::Empty(b) => b,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Space {
    X,
    O,
    Empty,
}
impl Space {
    fn button_style(&self) -> ButtonStyle {
        match self {
            Space::X => ButtonStyle::Danger,
            Space::O => ButtonStyle::Primary,
            Space::Empty => ButtonStyle::Secondary,
        }
    }
    fn string_with_ansi(&self) -> String {
        match self {
            Space::X => format!("{}", "X".red()),
            Space::O => format!("{}", "O".blue()),
            Space::Empty => format!("{}", "_".black()),
        }
    }
}

impl std::fmt::Display for Space {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Space::X => write!(f, "🇽"),
            Space::O => write!(f, "🇴"),
            Space::Empty => write!(f, "."),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum State {
    AwaitingApproval(Awaiting),
    InProgress(InProgress),
    Finished(WonGame),
    Cancelled(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Awaiting {
    inviter: UserId,
    invitee: UserId,
}

impl Awaiting {
    fn challenge_message(&self) -> String {
        format!("{} has challenged {} to a game of Ultimate Tic Tac Toe", self.inviter.mention(), self.invitee.mention())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InProgress {
    board: MetaBoard,
}

impl InProgress {
    fn make_move(&mut self, x: usize, y: usize, piece: Space, players: &CycleVec<Player>) -> Result<bool> {
        if x > 2 || y > 2 {
            return Err(qg_shared::anyhow::anyhow!("Invalid move, out of bounds"));
        }

        Ok(match self.board.selected {
            Some((bx, by)) => {
                // do move
                let board = self.board.spaces[bx][by].board_mut();
                if board.spaces[x][y] != Space::Empty {
                    return Err(qg_shared::anyhow::anyhow!("Invalid move, space already taken"));
                }
                board.spaces[x][y] = piece;
                // check if board has been won
                match board.check_winner(players) {
                    None => {
                        // something??
                    }
                    Some(outcome) => {
                        self.board.handle_outcome(bx, by, outcome);
                    }
                }
                if self.board.spaces[x][y].ignore_board() == Space::Empty {
                    self.board.selected = Some((x, y));
                } else {
                    self.board.selected = None;
                }
                true
            }
            None => {
                // select board
                // if board has already been won, error
                if self.board.spaces[x][y].ignore_board() != Space::Empty {
                    return Err(qg_shared::anyhow::anyhow!("Invalid move, board already won"));
                }
                // else select board
                self.board.selected = Some((x, y));
                false
            }
        })
    }

    fn new() -> InProgress {
        InProgress { board: MetaBoard::new() }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WonGame {
    winner: Outcome,
    board: MetaBoard,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaBoard {
    selected: Option<(usize, usize)>,
    spaces: Vec<Vec<MetaSpace>>,
}

impl MetaBoard {
    pub fn handle_outcome(&mut self, x: usize, y: usize, outcome: Outcome) {
        let oldboard = self.spaces[x][y].ignore_space();
        match outcome {
            Outcome::Win(p) => match p.piece {
                Space::X => {
                    self.spaces[x][y] = MetaSpace::X(oldboard);
                }
                Space::O => {
                    self.spaces[x][y] = MetaSpace::O(oldboard);
                }
                Space::Empty => unreachable!(),
            },
            Outcome::Tie => {
                self.spaces[x][y] = MetaSpace::Tie(oldboard);
            }
        }
    }

    // pub fn into_board(&self) -> Board {
    //     Board {
    //         spaces: self.spaces.iter().map(|row| row.iter().map(|s| s.ignore_board()).collect()).collect(),
    //     }
    // }

    fn new() -> MetaBoard {
        MetaBoard {
            selected: None,
            spaces: vec![
                vec![
                    MetaSpace::Empty(Board {
                        spaces: vec![vec![Space::Empty; 3]; 3]
                    });
                    3
                ];
                3
            ],
        }
    }

    fn raw_string_map(&self) -> String {
        // discord supports ANSI escape codes, so we can use those to color the board for readability
        format!("```ansi\n{}\n```", {
            // formatted as
            // 00 01 02 | 00 01 02 | 00 01 02
            // 10 11 12 | 10 11 12 | 10 11 12
            // 20 21 22 | 20 21 22 | 20 21 22
            // ---------+----------+---------
            // 00 01 02 | 00 01 02 | 00 01 02
            // 10 11 12 | 10 11 12 | 10 11 12
            // 20 21 22 | 20 21 22 | 20 21 22
            // ---------+----------+---------
            // 00 01 02 | 00 01 02 | 00 01 02
            // 10 11 12 | 10 11 12 | 10 11 12
            // 20 21 22 | 20 21 22 | 20 21 22

            // with each compartment being one of the 3x3 boards
            let mut strings = vec![String::new(); 9];
            // we will insert the `---------+----------+---------` lines after processing, its easier
            for (x, row) in self.spaces.iter().enumerate() {
                for (y, space) in row.iter().enumerate() {
                    let board = space.ignore_space();
                    for (bx, brow) in board.spaces.iter().enumerate() {
                        for (by, bspace) in brow.iter().enumerate() {
                            strings[(x * 3) + bx].push_str(bspace.string_with_ansi().as_str());
                            if by != 2 {
                                strings[(x * 3) + bx].push(' ');
                            } else if y != 2 {
                                strings[(x * 3) + bx].push_str(" | ");
                            }
                        }
                    }
                    // if y != 2 {
                    //     strings[x * 3].push_str("   ");
                    //     strings[(x * 3) + 1].push_str("   ");
                    //     strings[(x * 3) + 2].push_str("   ");
                    // }
                }
            }
            for string in strings.iter_mut() {
                let mut ts = format!(" {}", string);
                std::mem::swap(string, &mut ts);
            }
            strings.insert(3, String::from("-------+-------+-------"));
            strings.insert(7, String::from("-------+-------+-------"));
            strings.join("\n")
        })
    }

    fn button_for(&self, x: usize, y: usize, button: &mut qg_shared::serenity::builder::CreateButton) {
        // this depends on whether or not a board is selected
        match self.selected {
            None => {
                // the button is the space of the board at x, y for metaboard
                let p = self.spaces[x][y].ignore_board();
                button.label(format!("{}", p)).custom_id(Action::Place(x, y).to_custom_id("ultimatetictactoe"));
                if p != Space::Empty {
                    button.disabled(true);
                }
                button.style(p.button_style());
            }
            Some((bx, by)) => {
                self.spaces[bx][by].ignore_space().button_for(x, y, button, bx, by);
                // println!("bx: {} by:{} x:{} y:{}", bx, by, x, y);
            }
        }
    }

    fn string_map(&self) -> String {
        // same as raw_string_map, but if a metaspace is won, the entire 3x3 grid for it is the winner's piece
        // if a metaspace is empty, show the individual spaces

        // discord supports ANSI escape codes, so we can use those to color the board for readability
        format!("```ansi\n{}\n```", {
            // formatted as
            // 00 01 02 | 00 01 02 | 00 01 02
            // 10 11 12 | 10 11 12 | 10 11 12
            // 20 21 22 | 20 21 22 | 20 21 22
            // ---------+----------+---------
            // 00 01 02 | 00 01 02 | 00 01 02
            // 10 11 12 | 10 11 12 | 10 11 12
            // 20 21 22 | 20 21 22 | 20 21 22
            // ---------+----------+---------
            // 00 01 02 | 00 01 02 | 00 01 02
            // 10 11 12 | 10 11 12 | 10 11 12
            // 20 21 22 | 20 21 22 | 20 21 22

            // with each compartment being one of the 3x3 boards
            let mut strings = vec![String::new(); 9];
            // we will insert the `---------+----------+---------` lines after processing, its easier
            for (x, row) in self.spaces.iter().enumerate() {
                for (y, space) in row.iter().enumerate() {
                    let board = space.ignore_space();
                    if let MetaSpace::Empty(_) = space {
                        for (bx, brow) in board.spaces.iter().enumerate() {
                            for (by, bspace) in brow.iter().enumerate() {
                                strings[(x * 3) + bx].push_str(bspace.string_with_ansi().as_str());
                                if by != 2 {
                                    strings[(x * 3) + bx].push(' ');
                                } else if y != 2 {
                                    strings[(x * 3) + bx].push_str(" | ");
                                }
                            }
                        }
                    } else {
                        let space_string = if let MetaSpace::Tie(_) = space {
                            "?".green().to_string()
                        } else {
                            space.ignore_board().string_with_ansi()
                        };
                        for (bx, brow) in board.spaces.iter().enumerate() {
                            for (by, _bspace) in brow.iter().enumerate() {
                                strings[(x * 3) + bx].push_str(&space_string);
                                if by != 2 {
                                    strings[(x * 3) + bx].push(' ');
                                } else if y != 2 {
                                    strings[(x * 3) + bx].push_str(" | ");
                                }
                            }
                        }
                    }
                    // if y != 2 {
                    //     strings[x * 3].push_str("   ");
                    //     strings[(x * 3) + 1].push_str("   ");
                    //     strings[(x * 3) + 2].push_str("   ");
                    // }
                }

                // if x != 2 {
                //     strings[x * 3].push_str("   ");
                //     strings[(x * 3) + 1].push_str("   ");
                //     strings[(x * 3) + 2].push_str("   ");
                // }
            }
            for string in strings.iter_mut() {
                let mut ts = format!(" {}", string);
                std::mem::swap(string, &mut ts);
            }
            strings.insert(3, String::from("-------+-------+-------"));
            strings.insert(7, String::from("-------+-------+-------"));
            strings.join("\n")
        })
    }

    fn check_winner(&self, players: &CycleVec<Player>) -> Option<Outcome> {
        // just like check_winner for board, except we need to be able to handle a tie differently since that's also a piece on the board
        // a tie can win the game for EITHER PLAYER

        let mut winners = vec![];

        for i in 0..=2 {
            // check rows
            if let Some(w) = all_equal(&self.spaces[i][0], &self.spaces[i][1], &self.spaces[i][2]) {
                // return Some(Outcome::Win(players.all().find(|p| p.piece == w).copied()?));
                winners.push(w);
            }
        }
        for i in 0..=2 {
            // check columns
            if let Some(w) = all_equal(&self.spaces[0][i], &self.spaces[1][i], &self.spaces[2][i]) {
                // return Some(Outcome::Win(players.all().find(|p| p.piece == w).copied()?));
                winners.push(w);
            }
        }
        // check diagonals
        if let Some(w) = all_equal(&self.spaces[0][0], &self.spaces[1][1], &self.spaces[2][2]) {
            // return Some(Outcome::Win(players.all().find(|p| p.piece == w).copied()?));
            winners.push(w);
        }
        if let Some(w) = all_equal(&self.spaces[2][0], &self.spaces[1][1], &self.spaces[0][2]) {
            // return Some(Outcome::Win(players.all().find(|p| p.piece == w).copied()?));
            winners.push(w);
        }
        // check tie
        if self.spaces.iter().flatten().all(|s| {
            *s != MetaSpace::Empty(Board {
                spaces: vec![vec![Space::Empty; 3]; 3],
            })
        }) {
            return Some(Outcome::Tie);
        }
        winners.sort();
        winners.dedup();
        match winners.len() {
            0 => None,
            1 => Some(Outcome::Win(players.all().find(|p| p.piece == winners[0]).copied()?)),
            _ => Some(Outcome::Tie),
        }
    }
}

fn all_equal(x: &MetaSpace, y: &MetaSpace, z: &MetaSpace) -> Option<Space> {
    let won = match (x, y, z) {
        // if any of them are empty, None
        (MetaSpace::Empty(_), _, _) => None,
        (_, MetaSpace::Empty(_), _) => None,
        (_, _, MetaSpace::Empty(_)) => None,
        // if all of them are a tie, None
        (MetaSpace::Tie(_), MetaSpace::Tie(_), MetaSpace::Tie(_)) => None,
        // if all of them are the same, Some
        (MetaSpace::X(_), MetaSpace::X(_), MetaSpace::X(_)) => Some(Space::X),
        (MetaSpace::O(_), MetaSpace::O(_), MetaSpace::O(_)) => Some(Space::O),
        // if all are the same except one is a tie, Some
        (MetaSpace::Tie(_), MetaSpace::X(_), MetaSpace::X(_)) => Some(Space::X),
        (MetaSpace::X(_), MetaSpace::Tie(_), MetaSpace::X(_)) => Some(Space::X),
        (MetaSpace::X(_), MetaSpace::X(_), MetaSpace::Tie(_)) => Some(Space::X),
        (MetaSpace::Tie(_), MetaSpace::O(_), MetaSpace::O(_)) => Some(Space::O),
        (MetaSpace::O(_), MetaSpace::Tie(_), MetaSpace::O(_)) => Some(Space::O),
        (MetaSpace::O(_), MetaSpace::O(_), MetaSpace::Tie(_)) => Some(Space::O),
        // if all are the same except two are ties, Some
        (MetaSpace::Tie(_), MetaSpace::Tie(_), MetaSpace::X(_)) => Some(Space::X),
        (MetaSpace::Tie(_), MetaSpace::X(_), MetaSpace::Tie(_)) => Some(Space::X),
        (MetaSpace::X(_), MetaSpace::Tie(_), MetaSpace::Tie(_)) => Some(Space::X),
        (MetaSpace::Tie(_), MetaSpace::Tie(_), MetaSpace::O(_)) => Some(Space::O),
        (MetaSpace::Tie(_), MetaSpace::O(_), MetaSpace::Tie(_)) => Some(Space::O),
        (MetaSpace::O(_), MetaSpace::Tie(_), MetaSpace::Tie(_)) => Some(Space::O),
        // else None
        _ => None,
    };
    qg_shared::log::trace!(
        "x: {}, y: {}, z: {}, won: {}",
        x.ignore_board().string_with_ansi(),
        y.ignore_board().string_with_ansi(),
        z.ignore_board().string_with_ansi(),
        won.unwrap_or(Space::Empty).string_with_ansi()
    );
    won
}

impl WonGame {
    fn win_message(&self) -> String {
        match self.winner {
            Outcome::Win(player) => format!("{} [{}] has won!", player.id.mention(), player.piece),
            Outcome::Tie => String::from("It's a tie!"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Board {
    spaces: Vec<Vec<Space>>,
}

impl Board {
    pub fn button_for(&self, x: usize, y: usize, button: &mut qg_shared::serenity::builder::CreateButton, thisboardx: usize, thisboardy: usize) {
        let p = self.spaces[x][y];
        button.label(format!("{}", p)).custom_id(Action::Place(x, y).to_custom_id("ultimatetictactoe"));
        if p != Space::Empty {
            button.disabled(true);
        }
        if x == thisboardx && y == thisboardy {
            button.style(ButtonStyle::Success);
        } else {
            button.style(p.button_style());
        }
    }

    fn check_winner(&self, players: &CycleVec<Player>) -> Option<Outcome> {
        // check rows
        for row in self.spaces.iter() {
            if row.iter().all(|s| *s == Space::X) {
                return Some(Outcome::Win(players.all().find(|p| p.piece == Space::X).copied()?));
            }
            if row.iter().all(|s| *s == Space::O) {
                return Some(Outcome::Win(players.all().find(|p| p.piece == Space::O).copied()?));
            }
        }
        // check columns
        for x in 0..3 {
            if self.spaces.iter().all(|row| row[x] == Space::X) {
                return Some(Outcome::Win(players.all().find(|p| p.piece == Space::X).copied()?));
            }
            if self.spaces.iter().all(|row| row[x] == Space::O) {
                return Some(Outcome::Win(players.all().find(|p| p.piece == Space::O).copied()?));
            }
        }
        // check diagonals
        if self.spaces[0][0] == Space::X && self.spaces[1][1] == Space::X && self.spaces[2][2] == Space::X {
            return Some(Outcome::Win(players.all().find(|p| p.piece == Space::X).copied()?));
        }
        if self.spaces[0][0] == Space::O && self.spaces[1][1] == Space::O && self.spaces[2][2] == Space::O {
            return Some(Outcome::Win(players.all().find(|p| p.piece == Space::O).copied()?));
        }
        if self.spaces[0][2] == Space::X && self.spaces[1][1] == Space::X && self.spaces[2][0] == Space::X {
            return Some(Outcome::Win(players.all().find(|p| p.piece == Space::X).copied()?));
        }
        if self.spaces[0][2] == Space::O && self.spaces[1][1] == Space::O && self.spaces[2][0] == Space::O {
            return Some(Outcome::Win(players.all().find(|p| p.piece == Space::O).copied()?));
        }
        // check tie
        if self.spaces.iter().flatten().all(|s| *s != Space::Empty) {
            return Some(Outcome::Tie);
        }
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Outcome {
    Win(Player),
    Tie,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Player {
    id: UserId,
    piece: Space,
}
