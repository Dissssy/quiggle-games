use qg_shared::{serenity::all::*, UnorderedVec};

pub fn command() -> LeaderboardCommand {
    LeaderboardCommand
}

pub struct LeaderboardCommand;

#[qg_shared::async_trait]
impl qg_shared::Command for LeaderboardCommand {
    // fn register<'a>(&self, builder: &'a mut CreateApplicationCommand) -> &'a mut CreateApplicationCommand {
    //     let info = self.get_command_info();
    //     builder.name(info.name).description(info.description);
    //     builder
    // }

    fn register(&self) -> CreateCommand {
        let info = self.get_command_info();
        CreateCommand::new(info.name).description(info.description).set_options(vec![
            CreateCommandOption::new(CommandOptionType::SubCommandGroup, "slidingpuzzle", "Sliding Puzzle Leaderboards")
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::SubCommand, "3x3", "3x3 Sliding Puzzle Leaderboards")
                        .add_sub_option(
                            CreateCommandOption::new(CommandOptionType::String, "sort", "value to sort by when getting the leaderboard (defaults to score)")
                                .add_string_choice("score", "score")
                                .add_string_choice("time", "time"),
                        )
                        .add_sub_option(
                            CreateCommandOption::new(CommandOptionType::String, "difficulty", "difficulty to filter by when getting the leaderboard (defaults to easy)")
                                .add_string_choice("easy", "easy")
                                .add_string_choice("medium", "medium")
                                .add_string_choice("hard", "hard"),
                        ),
                )
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::SubCommand, "4x4", "4x4 Sliding Puzzle Leaderboards")
                        .add_sub_option(
                            CreateCommandOption::new(CommandOptionType::String, "sort", "value to sort by when getting the leaderboard (defaults to score)")
                                .add_string_choice("score", "score")
                                .add_string_choice("time", "time"),
                        )
                        .add_sub_option(
                            CreateCommandOption::new(CommandOptionType::String, "difficulty", "difficulty to filter by when getting the leaderboard (defaults to easy)")
                                .add_string_choice("easy", "easy")
                                .add_string_choice("medium", "medium")
                                .add_string_choice("hard", "hard"),
                        ),
                )
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::SubCommand, "5x5", "5x5 Sliding Puzzle Leaderboards")
                        .add_sub_option(
                            CreateCommandOption::new(CommandOptionType::String, "sort", "value to sort by when getting the leaderboard (defaults to score)")
                                .add_string_choice("score", "score")
                                .add_string_choice("time", "time"),
                        )
                        .add_sub_option(
                            CreateCommandOption::new(CommandOptionType::String, "difficulty", "difficulty to filter by when getting the leaderboard (defaults to easy)")
                                .add_string_choice("easy", "easy")
                                .add_string_choice("medium", "medium")
                                .add_string_choice("hard", "hard"),
                        ),
                ),
            CreateCommandOption::new(CommandOptionType::SubCommand, "tictactoe", "Tic Tac Toe Leaderboards"),
            CreateCommandOption::new(CommandOptionType::SubCommand, "ultimate_tictactoe", "Ultimate Tic Tac Toe Leaderboards"),
        ])
    }

    fn get_command_info(&self) -> qg_shared::CommandInfo {
        qg_shared::CommandInfo {
            name: String::from("leaderboard"),
            description: String::from("Check the leaderboard!"),
            options: qg_shared::UnorderedVec::from(vec![
                qg_shared::CommandOption {
                    name: String::from("slidingpuzzle"),
                    description: String::from("Sliding Puzzle Leaderboards"),
                    choices: qg_shared::UnorderedVec::from(vec![]),
                    option_type: qg_shared::CommandOptionType::SubCommandGroup(UnorderedVec::from(vec![
                        qg_shared::CommandOption {
                            name: String::from("3x3"),
                            description: String::from("3x3 Sliding Puzzle Leaderboards"),
                            option_type: qg_shared::CommandOptionType::SubCommand(UnorderedVec::from(vec![
                                qg_shared::CommandOption {
                                    name: String::from("sort"),
                                    description: String::from("value to sort by when getting the leaderboard (defaults to score)"),
                                    option_type: qg_shared::CommandOptionType::String,
                                    choices: UnorderedVec::from(vec![
                                        qg_shared::CommandOptionChoice {
                                            name: String::from("score"),
                                            value: String::from("score"),
                                        },
                                        qg_shared::CommandOptionChoice {
                                            name: String::from("time"),
                                            value: String::from("time"),
                                        },
                                    ]),
                                    required: false,
                                },
                                qg_shared::CommandOption {
                                    name: String::from("difficulty"),
                                    description: String::from("difficulty to filter by when getting the leaderboard (defaults to easy)"),
                                    option_type: qg_shared::CommandOptionType::String,
                                    choices: UnorderedVec::from(vec![
                                        qg_shared::CommandOptionChoice {
                                            name: String::from("easy"),
                                            value: String::from("easy"),
                                        },
                                        qg_shared::CommandOptionChoice {
                                            name: String::from("medium"),
                                            value: String::from("medium"),
                                        },
                                        qg_shared::CommandOptionChoice {
                                            name: String::from("hard"),
                                            value: String::from("hard"),
                                        },
                                    ]),
                                    required: false,
                                },
                            ])),
                            choices: qg_shared::UnorderedVec::from(vec![]),
                            required: false,
                        },
                        qg_shared::CommandOption {
                            name: String::from("4x4"),
                            description: String::from("4x4 Sliding Puzzle Leaderboards"),
                            option_type: qg_shared::CommandOptionType::SubCommand(UnorderedVec::from(vec![
                                qg_shared::CommandOption {
                                    name: String::from("sort"),
                                    description: String::from("value to sort by when getting the leaderboard (defaults to score)"),
                                    option_type: qg_shared::CommandOptionType::String,
                                    choices: UnorderedVec::from(vec![
                                        qg_shared::CommandOptionChoice {
                                            name: String::from("score"),
                                            value: String::from("score"),
                                        },
                                        qg_shared::CommandOptionChoice {
                                            name: String::from("time"),
                                            value: String::from("time"),
                                        },
                                    ]),
                                    required: false,
                                },
                                qg_shared::CommandOption {
                                    name: String::from("difficulty"),
                                    description: String::from("difficulty to filter by when getting the leaderboard (defaults to easy)"),
                                    option_type: qg_shared::CommandOptionType::String,
                                    choices: UnorderedVec::from(vec![
                                        qg_shared::CommandOptionChoice {
                                            name: String::from("easy"),
                                            value: String::from("easy"),
                                        },
                                        qg_shared::CommandOptionChoice {
                                            name: String::from("medium"),
                                            value: String::from("medium"),
                                        },
                                        qg_shared::CommandOptionChoice {
                                            name: String::from("hard"),
                                            value: String::from("hard"),
                                        },
                                    ]),
                                    required: false,
                                },
                            ])),
                            choices: qg_shared::UnorderedVec::from(vec![]),
                            required: false,
                        },
                        qg_shared::CommandOption {
                            name: String::from("5x5"),
                            description: String::from("5x5 Sliding Puzzle Leaderboards"),
                            option_type: qg_shared::CommandOptionType::SubCommand(UnorderedVec::from(vec![
                                qg_shared::CommandOption {
                                    name: String::from("sort"),
                                    description: String::from("value to sort by when getting the leaderboard (defaults to score)"),
                                    option_type: qg_shared::CommandOptionType::String,
                                    choices: UnorderedVec::from(vec![
                                        qg_shared::CommandOptionChoice {
                                            name: String::from("score"),
                                            value: String::from("score"),
                                        },
                                        qg_shared::CommandOptionChoice {
                                            name: String::from("time"),
                                            value: String::from("time"),
                                        },
                                    ]),
                                    required: false,
                                },
                                qg_shared::CommandOption {
                                    name: String::from("difficulty"),
                                    description: String::from("difficulty to filter by when getting the leaderboard (defaults to easy)"),
                                    option_type: qg_shared::CommandOptionType::String,
                                    choices: UnorderedVec::from(vec![
                                        qg_shared::CommandOptionChoice {
                                            name: String::from("easy"),
                                            value: String::from("easy"),
                                        },
                                        qg_shared::CommandOptionChoice {
                                            name: String::from("medium"),
                                            value: String::from("medium"),
                                        },
                                        qg_shared::CommandOptionChoice {
                                            name: String::from("hard"),
                                            value: String::from("hard"),
                                        },
                                    ]),
                                    required: false,
                                },
                            ])),
                            choices: qg_shared::UnorderedVec::from(vec![]),
                            required: false,
                        },
                    ])),
                    required: false,
                },
                qg_shared::CommandOption {
                    name: String::from("tictactoe"),
                    description: String::from("Tic Tac Toe Leaderboards"),
                    option_type: qg_shared::CommandOptionType::SubCommand(UnorderedVec::from(vec![])),
                    choices: qg_shared::UnorderedVec::from(vec![]),
                    required: false,
                },
                qg_shared::CommandOption {
                    name: String::from("ultimate_tictactoe"),
                    description: String::from("Ultimate Tic Tac Toe Leaderboards"),
                    option_type: qg_shared::CommandOptionType::SubCommand(UnorderedVec::from(vec![])),
                    choices: qg_shared::UnorderedVec::from(vec![]),
                    required: false,
                },
            ]),
        }
    }

    async fn application_command(&mut self, ctx: &Context, interaction: &mut CommandInteraction, db: &mut qg_shared::OptTrans<'_>) -> qg_shared::anyhow::Result<()> {
        // interaction.data.options =
        //  [
        //      CommandDataOption {
        //          name: "slidingpuzzle",
        //          value: SubCommandGroup([
        //              CommandDataOption {
        //                  name: "3x3",
        //                  value: SubCommand([
        //                      CommandDataOption {
        //                          name: "sort",
        //                          value: String("score")
        //                      },
        //                      CommandDataOption {
        //                          name: "difficulty",
        //                          value: String("easy")
        //                      }
        //                    ])
        //              }
        //          ])
        //    }
        // ]

        Self::unwrap_layers(ctx, &interaction.data.options.clone(), interaction, db).await?;

        Ok(())
    }
}

impl LeaderboardCommand {
    #[qg_shared::async_recursion]
    async fn unwrap_layers(ctx: &Context, options: &[CommandDataOption], interaction: &mut CommandInteraction, db: &mut qg_shared::OptTrans<'_>) -> qg_shared::anyhow::Result<()> {
        let tx = match db {
            Some(db) => db,
            None => return Err(qg_shared::anyhow::anyhow!("No database connection")),
        };

        let option = match options.first() {
            Some(option) => option,
            None => return Err(qg_shared::anyhow::anyhow!("No options found")),
        };

        match option.name.as_str() {
            "slidingpuzzle" => match option.value {
                CommandDataOptionValue::SubCommandGroup(ref options) => {
                    Self::unwrap_layers(ctx, options, interaction, db).await?;
                }
                _ => {
                    return Err(qg_shared::anyhow::anyhow!("Expected SubCommandGroup, got {:?}", option.value));
                }
            },
            t if t == "3x3" || t == "4x4" || t == "5x5" => match option.value {
                CommandDataOptionValue::SubCommand(ref options) => {
                    let mut filters = qg_shared::db::SlidingPuzzleFilters::default();

                    // filters.threebythree();
                    match t {
                        "3x3" => {
                            filters.threebythree();
                        }
                        "4x4" => {
                            filters.fourbyfour();
                        }
                        "5x5" => {
                            filters.fivebyfive();
                        }
                        _ => unreachable!(),
                    }

                    for option in options {
                        match option.name.as_str() {
                            "sort" => match option.value {
                                CommandDataOptionValue::String(ref value) => match value.as_str() {
                                    "score" => {
                                        filters.sort_by_score();
                                    }
                                    "time" => {
                                        filters.sort_by_time();
                                    }
                                    _ => {
                                        return Err(qg_shared::anyhow::anyhow!("Unhandled value `{}`", value));
                                    }
                                },
                                _ => {
                                    return Err(qg_shared::anyhow::anyhow!("Expected String, got {:?}", option.value));
                                }
                            },
                            "difficulty" => match option.value {
                                CommandDataOptionValue::String(ref value) => match value.as_str() {
                                    "easy" => {
                                        filters.easy();
                                    }
                                    "medium" => {
                                        filters.medium();
                                    }
                                    "hard" => {
                                        filters.hard();
                                    }
                                    _ => {
                                        return Err(qg_shared::anyhow::anyhow!("Unhandled value `{}`", value));
                                    }
                                },
                                _ => {
                                    return Err(qg_shared::anyhow::anyhow!("Expected String, got {:?}", option.value));
                                }
                            },
                            v => {
                                return Err(qg_shared::anyhow::anyhow!("Unhandled option `{}`", v));
                            }
                        }
                    }

                    let (leaderboard, more_available) = qg_shared::db::SlidingPuzzle::get_standings(filters, tx).await?;

                    Self::send_slidingpuzzle_leaderboard(ctx, &leaderboard, interaction, t, more_available).await?;
                }
                _ => {
                    return Err(qg_shared::anyhow::anyhow!("Expected SubCommand, got {:?}", option.value));
                }
            },
            "tictactoe" => match option.value {
                CommandDataOptionValue::SubCommand(ref _options) => {
                    let (mut standings, more) = qg_shared::db::TicTacToe::get_standings(tx).await?;
                    Self::send_tictactoe_leaderboard(ctx, &mut standings, interaction, more).await?;
                }
                _ => {
                    return Err(qg_shared::anyhow::anyhow!("Expected SubCommand, got {:?}", option.value));
                }
            },
            "ultimate_tictactoe" => match option.value {
                CommandDataOptionValue::SubCommand(ref _options) => {
                    let (mut standings, more) = qg_shared::db::UltimateTicTacToe::get_standings(tx).await?;
                    Self::send_ultimate_tictactoe_leaderboard(ctx, &mut standings, interaction, more).await?;
                }
                _ => {
                    return Err(qg_shared::anyhow::anyhow!("Expected SubCommand, got {:?}", option.value));
                }
            },
            v => {
                return Err(qg_shared::anyhow::anyhow!("Unhandled option `{}`", v));
            }
        }

        Ok(())
    }

    async fn send_tictactoe_leaderboard(ctx: &Context, leaderboard: &mut [qg_shared::db::TTTLeaderboardEntry], interaction: &mut CommandInteraction, _more_available: bool) -> Result<()> {
        let mut message = CreateInteractionResponseMessage::new();

        message = message.content("`Tic Tac Toe Leaderboard`");

        leaderboard.sort_by(|a, b| {
            b.rating.partial_cmp(&a.rating).unwrap_or({
                // if the rating fails to compare, compare by wins
                b.wins.cmp(&a.wins)
            })
        });

        for (i, entry) in leaderboard.iter().enumerate() {
            let mut embed = CreateEmbed::default();

            embed = embed.fields(vec![
                ("Wins", format!("{}", entry.wins), true),
                // ("Losses", format!("{}", entry.losses), true),
                ("Rating", format!("{}", entry.rating), true),
            ]);

            let fancyuser = Self::get_author(&entry.user, ctx, interaction).await;

            let mut author = CreateEmbedAuthor::new(format!("#{}: {} ({})", i + 1, fancyuser.name, fancyuser.discord_id));

            if let Some(img) = fancyuser.avatar {
                author = author.icon_url(img);
            }

            embed = embed.author(author);

            message = message.add_embed(embed);
        }

        interaction.create_response(&ctx.http, CreateInteractionResponse::Message(message)).await?;

        Ok(())
    }

    async fn send_ultimate_tictactoe_leaderboard(ctx: &Context, leaderboard: &mut [qg_shared::db::UTTTLeaderboardEntry], interaction: &mut CommandInteraction, _more_available: bool) -> Result<()> {
        let mut message = CreateInteractionResponseMessage::new();

        message = message.content("`Ultimate Tic Tac Toe Leaderboard`");

        leaderboard.sort_by(|a, b| {
            b.rating.partial_cmp(&a.rating).unwrap_or({
                // if the rating fails to compare, compare by wins
                b.wins.cmp(&a.wins)
            })
        });

        for (i, entry) in leaderboard.iter().enumerate() {
            let mut embed = CreateEmbed::default();

            embed = embed.fields(vec![
                ("Wins", format!("{}", entry.wins), true),
                // ("Losses", format!("{}", entry.losses), true),
                ("Rating", format!("{}", entry.rating), true),
            ]);

            let fancyuser = Self::get_author(&entry.user, ctx, interaction).await;

            let mut author = CreateEmbedAuthor::new(format!("#{}: {} ({})", i + 1, fancyuser.name, fancyuser.discord_id));

            if let Some(img) = fancyuser.avatar {
                author = author.icon_url(img);
            }

            embed = embed.author(author);

            message = message.add_embed(embed);
        }

        interaction.create_response(&ctx.http, CreateInteractionResponse::Message(message)).await?;

        Ok(())
    }

    async fn send_slidingpuzzle_leaderboard(
        ctx: &Context,
        leaderboard: &[qg_shared::db::SlidingPuzzleWithUser],
        interaction: &mut CommandInteraction,
        s: &str,
        _more_available: bool,
    ) -> qg_shared::anyhow::Result<()> {
        let mut message = CreateInteractionResponseMessage::new();

        message = message.content(format!("`Sliding Puzzle {} Leaderboard`", s));

        // {
        //     let mut embed = CreateEmbed::default();

        //     embed = embed.title(format!("{} Leaderboard", s)).fields(leaderboard.iter().enumerate().map(|(i, holder)| {
        //         (
        //             format!("{}. {} ({})", i + 1, holder.user.name, holder.user.discord_id),
        //             format!("Score: {}\nTime: {}", holder.puzzle.score, holder.puzzle.time),
        //             false,
        //         )
        //     }));

        //     message = message.add_embed(embed);
        // }

        for (i, entry) in leaderboard.iter().enumerate() {
            let mut embed = CreateEmbed::default();

            embed = embed.fields(vec![("Score", format!("{}", entry.puzzle.score), true), ("Time", format!("{}", entry.puzzle.time), true)]);

            let fancyuser = Self::get_author(&entry.user, ctx, interaction).await;

            let mut author = CreateEmbedAuthor::new(format!("#{}: {} ({})", i + 1, fancyuser.name, fancyuser.discord_id));

            if let Some(img) = fancyuser.avatar {
                author = author.icon_url(img);
            }

            embed = embed.author(author);

            message = message.add_embed(embed);
        }

        interaction.create_response(&ctx.http, CreateInteractionResponse::Message(message)).await?;

        Ok(())
    }

    async fn get_author(user: &qg_shared::db::User, ctx: &Context, interaction: &mut CommandInteraction) -> FancyUser {
        let mut author = FancyUser {
            name: user.name.clone(),
            discord_id: user.discord_id as u64,
            avatar: None,
        };

        // first, attempt to get the member from the discord api
        if let Some(guild) = interaction.guild_id.as_ref() {
            // then attempt to get the member from the discord api
            if let Ok(member) = guild.member(&ctx.http, UserId::from(author.discord_id)).await {
                // if we got the member, we can get the avatar url, nickname, etc
                if let Some(avatar) = member.avatar_url() {
                    author.avatar = Some(avatar);
                }
                if let Some(nick) = member.nick {
                    author.name = nick;
                }
            }
        }
        // fallback to the regular avatar url
        if author.avatar.is_none() {
            // then attempt to get the user from the discord api
            if let Ok(user) = ctx.http.get_user(UserId::from(author.discord_id)).await {
                // if we got the user, we can get the avatar url
                if let Some(avatar) = user.avatar_url() {
                    author.avatar = Some(avatar);
                }
            }
        }

        author
    }
}

struct FancyUser {
    name: String,
    discord_id: u64,
    avatar: Option<String>,
}
