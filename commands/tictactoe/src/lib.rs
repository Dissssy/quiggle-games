use qg_shared::{
    anyhow::{anyhow, Result},
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

pub fn command() -> TicTacToe {
    TicTacToe
}

pub struct TicTacToe;

#[qg_shared::async_trait]
impl qg_shared::Command for TicTacToe {
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
            name: String::from("tictactoe"),
            description: String::from("Play a game of Tic Tac Toe"),
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
                    if user.bot {
                        return Err(qg_shared::anyhow::anyhow!("You cannot play against a bot"));
                    }

                    other = user.clone();
                    Player { id: other.id, piece: Space::O }
                }
                _ => return Err(qg_shared::anyhow::anyhow!("No opponent specified")),
            }
        });
        if !std::env::var("ALLOW_SELF_PLAY").ok().and_then(|s| s.parse::<bool>().ok()).unwrap_or(false) {
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
                    return Err(anyhow!("You are not the invitee"));
                }
                match action {
                    Action::Accept => {
                        self.gamestate = State::InProgress(InProgress {
                            board: Board {
                                spaces: vec![vec![Space::Empty; 3]; 3],
                            },
                        });
                        // interaction
                        //     .create_interaction_response(&ctx.http, |f| {
                        //         f.kind(qg_shared::serenity::model::application::interaction::InteractionResponseType::DeferredUpdateMessage)
                        //     })
                        //     .await?;
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
                        // interaction
                        //     .create_interaction_response(&ctx.http, |f| {
                        //         f.kind(qg_shared::serenity::model::application::interaction::InteractionResponseType::DeferredUpdateMessage)
                        //     })
                        //     .await?;
                    }
                    _ => {
                        return Err(anyhow!("Invalid action"));
                    }
                }
            }
            State::InProgress(ref mut game) => {
                if self.players.current().map(|s| s.id) != Some(interaction.user.id) {
                    return Err(anyhow!("It is not your turn"));
                }
                match action {
                    Action::Place(x, y) => {
                        if let Err(e) = game.make_move(x, y, self.players.current().ok_or(anyhow!("Player not found"))?.piece) {
                            return Err(anyhow!("Invalid move: {}", e));
                        }
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
                        } else {
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

                        // interaction
                        //     .create_interaction_response(&ctx, |f| f.kind(qg_shared::serenity::model::application::interaction::InteractionResponseType::DeferredUpdateMessage))
                        //     .await?
                    }
                    _ => {
                        return Err(anyhow!("Invalid action: {}", action.name()));
                    }
                }
            }
            _ => {
                return Err(anyhow!("Invalid action: {}", action.name()));
            }
        }

        self.render(ctx, interaction).await?;

        Ok(())
    }

    async fn render(&self, ctx: &Context, interaction: &mut MessageComponentInteraction) -> Result<()> {
        match &self.gamestate {
            State::Cancelled(reason) => {
                interaction
                    .create_interaction_response(&ctx.http, |f| {
                        f.kind(qg_shared::serenity::model::application::interaction::InteractionResponseType::DeferredUpdateMessage)
                    })
                    .await?;
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
                                a.create_button(|b| b.style(ButtonStyle::Success).label("Accept").custom_id(Action::Accept.to_custom_id("tictactoe")))
                                    .create_button(|b| b.style(ButtonStyle::Danger).label("Decline").custom_id(Action::Decline.to_custom_id("tictactoe")))
                            })
                        })
                    })
                    .await?;
            }
            State::InProgress(game) => {
                let mut content = self.title_card()?;
                let current_player = self.players.current().ok_or(anyhow!("Player not found"))?;
                content.push_str(&format!("It is {}'s turn [{}]", current_player.id.mention(), current_player.piece));
                interaction
                    .create_interaction_response(&ctx.http, |f| {
                        f.kind(qg_shared::serenity::model::application::interaction::InteractionResponseType::DeferredUpdateMessage)
                    })
                    .await?;
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
                interaction
                    .create_interaction_response(&ctx.http, |f| {
                        f.kind(qg_shared::serenity::model::application::interaction::InteractionResponseType::DeferredUpdateMessage)
                    })
                    .await?;
                interaction
                    .edit_original_interaction_response(&ctx.http, |d| {
                        d.content(content).components(|c| {
                            for x in 0..=2 {
                                c.create_action_row(|a| {
                                    for y in 0..=2 {
                                        a.create_button(|b| {
                                            won_game.board.button_for(x, y, b);
                                            b.disabled(true);
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
                                    a.create_button(|b| b.style(ButtonStyle::Success).label("Accept").custom_id(Action::Accept.to_custom_id("tictactoe")))
                                        .create_button(|b| b.style(ButtonStyle::Danger).label("Decline").custom_id(Action::Decline.to_custom_id("tictactoe")))
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
        Ok(format!("```{}\nTic Tac Toe\n```", qg_shared::serialize(&self)?.replace('\n', "")))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Space {
    X,
    O,
    Empty,
}
impl Space {
    fn button_style(&self) -> ButtonStyle {
        match self {
            Space::X => ButtonStyle::Primary,
            Space::O => ButtonStyle::Success,
            Space::Empty => ButtonStyle::Secondary,
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
        format!("{} has challenged {} to a game of Tic Tac Toe", self.inviter.mention(), self.invitee.mention())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InProgress {
    board: Board,
}

impl InProgress {
    fn make_move(&mut self, x: usize, y: usize, piece: Space) -> Result<()> {
        if x > 2 || y > 2 {
            return Err(qg_shared::anyhow::anyhow!("Invalid move, out of bounds"));
        }
        if self.board.spaces[x][y] != Space::Empty {
            return Err(qg_shared::anyhow::anyhow!("Invalid move, space already occupied"));
        }
        self.board.spaces[x][y] = piece;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WonGame {
    winner: Outcome,
    board: Board,
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
    pub fn button_for(&self, x: usize, y: usize, button: &mut qg_shared::serenity::builder::CreateButton) {
        let p = self.spaces[x][y];
        button.label(format!("{}", p)).custom_id(Action::Place(x, y).to_custom_id("tictactoe"));
        if p != Space::Empty {
            button.disabled(true);
        }
        button.style(p.button_style());
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
