use std::sync::Arc;

use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::resource::advancement::AdvancementArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::tree::builder::{argument, literal};
use crate::command::tree::CommandTree;
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::entity::player::Player;

const NAMES: [&str; 1] = ["advancement"];
const DESCRIPTION: &str = "Gives, removes, or checks player advancements.";

const ARG_TARGETS: &str = "targets";
const ARG_ADVANCEMENT: &str = "advancement";
#[allow(dead_code)]
const ARG_CRITERION: &str = "criterion";

struct GrantEverythingExecutor;

impl CommandExecutor for GrantEverythingExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;

            for target in targets {
                grant_all_advancements(target).await;
            }

            sender
                .send_message(TextComponent::translate::<_, Vec<TextComponent>>(
                    "commands.advancement.grant.many.to.many.success",
                    vec![],
                ))
                .await;
            Ok(())
        })
    }
}

struct GrantOnlyExecutor;

impl CommandExecutor for GrantOnlyExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;
            let advancement_id = AdvancementArgumentConsumer::find_arg(args, ARG_ADVANCEMENT)?;

            let registry = server.advancement_registry.read().await;
            let advancement = registry.get(&advancement_id).cloned();
            drop(registry);

            if let Some(adv) = advancement {
                for target in targets {
                    let mut tracker = target.advancement_tracker.lock().await;
                    tracker.grant_advancement(&advancement_id, &adv.advancement.requirements);
                    drop(tracker);
                    crate::advancement::AdvancementTriggers::send_update(target, server).await;
                }
                sender
                    .send_message(TextComponent::text(format!(
                        "Granted advancement {} to {} player(s)",
                        advancement_id, 1
                    )))
                    .await;
            } else {
                sender
                    .send_message(
                        TextComponent::text(format!("Unknown advancement: {}", advancement_id))
                            .color_named(NamedColor::Red),
                    )
                    .await;
            }
            Ok(())
        })
    }
}

struct RevokeEverythingExecutor;

impl CommandExecutor for RevokeEverythingExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;

            let registry = server.advancement_registry.read().await;
            let all_ids: Vec<ResourceLocation> = registry.all_ids();
            drop(registry);

            for target in targets {
                let mut tracker = target.advancement_tracker.lock().await;
                for id in &all_ids {
                    tracker.revoke_advancement(id);
                }
                tracker.mark_needs_reset();
                drop(tracker);
                crate::advancement::AdvancementTriggers::send_update(target, server).await;
            }

            sender
                .send_message(TextComponent::translate::<_, Vec<TextComponent>>(
                    "commands.advancement.revoke.many.to.many.success",
                    vec![],
                ))
                .await;
            Ok(())
        })
    }
}

struct RevokeOnlyExecutor;

impl CommandExecutor for RevokeOnlyExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;
            let advancement_id = AdvancementArgumentConsumer::find_arg(args, ARG_ADVANCEMENT)?;

            for target in targets {
                let mut tracker = target.advancement_tracker.lock().await;
                tracker.revoke_advancement(&advancement_id);
                drop(tracker);
                crate::advancement::AdvancementTriggers::send_update(target, server).await;
            }

            sender
                .send_message(TextComponent::text(format!(
                    "Revoked advancement {} from {} player(s)",
                    advancement_id, 1
                )))
                .await;
            Ok(())
        })
    }
}

async fn grant_all_advancements(player: &Arc<Player>) {
    if let Some(server) = player.world().server.upgrade() {
        let registry = server.advancement_registry.read().await;
        let mut tracker = player.advancement_tracker.lock().await;

        for (id, entry) in registry.iter() {
            tracker.grant_advancement(id, &entry.advancement.requirements);
        }
        tracker.mark_needs_reset();
        drop(tracker);
        drop(registry);
        crate::advancement::AdvancementTriggers::send_update(player, &server).await;
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("grant").then(
                argument(ARG_TARGETS, PlayersArgumentConsumer)
                    .then(literal("everything").execute(GrantEverythingExecutor))
                    .then(
                        literal("only").then(
                            argument(ARG_ADVANCEMENT, AdvancementArgumentConsumer)
                                .execute(GrantOnlyExecutor),
                        ),
                    ),
            ),
        )
        .then(
            literal("revoke").then(
                argument(ARG_TARGETS, PlayersArgumentConsumer)
                    .then(literal("everything").execute(RevokeEverythingExecutor))
                    .then(
                        literal("only").then(
                            argument(ARG_ADVANCEMENT, AdvancementArgumentConsumer)
                                .execute(RevokeOnlyExecutor),
                        ),
                    ),
            ),
        )
}
