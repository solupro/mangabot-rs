use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "可用命令:"
)]
pub enum Command {
    #[command(description = "开始对话")]
    Start,

    #[command(description = "复制消息")]
    Copy(String),

}
pub mod start;
pub mod copy;

