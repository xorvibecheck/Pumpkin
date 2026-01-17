use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};
use pumpkin_util::resource_location::ResourceLocation;

use crate::command::{
    CommandSender,
    args::{
        Arg, ArgumentConsumer, ConsumeResult, ConsumedArgs, DefaultNameArgConsumer, FindArg,
        GetClientSideArgParser,
    },
    dispatcher::CommandError,
    tree::RawArgs,
};
use crate::server::Server;

pub struct AdvancementArgumentConsumer;

impl GetClientSideArgParser for AdvancementArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::ResourceLocation
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AllRecipes)
    }
}

impl ArgumentConsumer for AdvancementArgumentConsumer {
    fn consume<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let advancement = args.pop();
        match advancement {
            Some(s) => Box::pin(async move { Some(Arg::ResourceLocation(s)) }),
            None => Box::pin(async move { None }),
        }
    }
}

impl DefaultNameArgConsumer for AdvancementArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "advancement"
    }
}

impl<'a> FindArg<'a> for AdvancementArgumentConsumer {
    type Data = ResourceLocation;

    fn find_arg(args: &'a ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::ResourceLocation(s)) => {
                let resource = if s.contains(':') {
                    ResourceLocation::from(*s)
                } else {
                    ResourceLocation::vanilla(*s)
                };
                Ok(resource)
            }
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
