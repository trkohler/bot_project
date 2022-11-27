pub mod add_check;
pub mod remain;

use std::sync::Arc;

use models::Category;
use notion::NotionApi;
use remain::remain as remain_handler;
use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::MessageId,
    utils::command::BotCommands,
};
use tokio::sync::Mutex;

use crate::{
    add_check::{receive_category, receive_check, start},
};

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveCategory,
    ReceiveCheckWithItems {
        category: Category,
    },
}

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "Додати витрати в певну категорію.")]
    AddCheck,
    #[command(description = "cancel the purchase procedure.")]
    Cancel,
    #[command(description = "Показати скільки грошей залишилося")]
    Remain,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();
    let notion_api = NotionApi::from_env();
    let message_ids: Arc<Mutex<[Option<MessageId>; 10]>> = Arc::new(Mutex::new([None; 10]));

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![
            InMemStorage::<State>::new(),
            notion_api,
            message_ids
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![State::Start]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::AddCheck].endpoint(start))
                .branch(case![Command::Remain].endpoint(remain_handler)),
        )
        .branch(case![Command::Cancel].endpoint(cancel));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::ReceiveCategory].endpoint(receive_category))
        .branch(case![State::ReceiveCheckWithItems { category }].endpoint(receive_check))
        .branch(dptree::endpoint(invalid_state));

    dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(message_handler)
}

async fn help(
    bot: Bot,
    msg: Message,
    message_ids: Arc<Mutex<[Option<MessageId>; 10]>>,
) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    clean_out_messages_in_chat(&bot, msg.chat.id, message_ids).await;
    Ok(())
}

async fn cancel(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    message_ids: Arc<Mutex<[Option<MessageId>; 10]>>,
) -> HandlerResult {
    bot.send_message(msg.chat.id, "Cancelling the dialogue.")
        .await?;
    clean_out_messages_in_chat(&bot, msg.chat.id, message_ids).await;
    dialogue.exit().await?;
    Ok(())
}

async fn invalid_state(
    bot: Bot,
    msg: Message,
    message_ids: Arc<Mutex<[Option<MessageId>; 10]>>,
) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Unable to handle the message. Type /help to see the usage.",
    )
    .await?;
    clean_out_messages_in_chat(&bot, msg.chat.id, message_ids).await;
    Ok(())
}

pub async fn clean_out_messages_in_chat(
    bot: &Bot,
    chat_id: ChatId,
    message_ids: Arc<Mutex<[Option<MessageId>; 10]>>,
) {
    for idx in 0..10 {
        let message_id = message_ids.lock().await[idx];
        message_ids.lock().await[idx] = delete_message(message_id, bot, chat_id.clone()).await;
    }
}

async fn delete_message(
    message_id: Option<MessageId>,
    bot: &Bot,
    chat_id: ChatId,
) -> Option<MessageId> {
    match message_id {
        Some(message_id) => {
            let _ = bot.delete_message(chat_id, message_id).await;
        }
        None => (),
    };
    None::<MessageId>
}
