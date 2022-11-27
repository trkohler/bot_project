use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use models::Category;
use models::Check;
use notion::NotionApi;
use notion::make_new_notion_entry_for_check;
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::KeyboardButton;
use teloxide::types::KeyboardMarkup;
use teloxide::types::Message;
use teloxide::types::MessageId;
use teloxide::Bot;
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::clean_out_messages_in_chat;
use crate::HandlerResult;
use crate::MyDialogue;
use crate::State;

pub async fn start(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    message_ids: Arc<Mutex<[Option<MessageId>; 10]>>,
) -> HandlerResult {
    message_ids.lock().await[0].insert(msg.id);
    let categories_buttons = [
        Category::FoodAndDrinks.to_string(),
        Category::OperationalSpends.to_string(),
        Category::Rent.to_string(),
        Category::UkrReponsibilities.to_string(),
    ]
    .map(|category| KeyboardButton::new(category));

    let sent_id = bot
        .send_message(msg.chat.id, "Спочатку виберіть категорію:")
        .reply_markup(KeyboardMarkup::new([categories_buttons]).one_time_keyboard(true))
        .await?
        .id;

    message_ids.lock().await[1].insert(sent_id);
    dialogue.update(State::ReceiveCategory).await?;

    Ok(())
}

pub async fn receive_check(
    bot: Bot,
    dialogue: MyDialogue,
    category: Category,
    client: NotionApi,
    msg: Message,
    message_ids: Arc<Mutex<[Option<MessageId>; 10]>>,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(raw_check) => {
            message_ids.lock().await[4].insert(msg.id);
            let unchecked_check = Check::from_str(&raw_check);
            if let Ok(check) = unchecked_check {
                make_new_notion_entry_for_check(client, &category, &check).await;
                let sent_id = bot
                    .send_message(msg.chat.id, "Чек успешно заведено.Історію буде видалено.")
                    .await?
                    .id;
                message_ids.lock().await[5].insert(sent_id);
                sleep(Duration::from_millis(1000)).await;
                clean_out_messages_in_chat(&bot, msg.chat.id, message_ids).await;
                dialogue.exit().await?;
            }
        }
        None => {
            bot.send_message(msg.chat.id, "Не можу без чека. Чекаю на чек.")
                .await?;
        }
    }

    Ok(())
}

pub async fn receive_category(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    message_ids: Arc<Mutex<[Option<MessageId>; 10]>>,
) -> HandlerResult {
    if let Some(category) = msg.text() {
        message_ids.lock().await[2].insert(msg.id);
        let sent_id = bot
            .send_message(dialogue.chat_id(), "А зараз мені потрібен твій чек...")
            .await?
            .id;
        message_ids.lock().await[3].insert(sent_id);
        let category_variant = Category::from_str(category);
        if let Ok(variant) = category_variant {
            dialogue
                .update(State::ReceiveCheckWithItems { category: variant })
                .await?;
        }
    }

    Ok(())
}
