use futures::future::join_all;
use models::Category;
use notion::NotionApi;
use notion::get_how_much_money_remain;
use teloxide::requests::Requester;
use teloxide::types::Message;
use teloxide::Bot;


use crate::HandlerResult;


pub async fn remain(bot: Bot, msg: Message, api: NotionApi) -> HandlerResult {
    let categories = [
        Category::FoodAndDrinks,
        Category::Rent,
        Category::UkrReponsibilities,
        Category::OperationalSpends,
    ];

    let cat_futures = categories
        .iter()
        .map(|category| get_how_much_money_remain(&api, category.clone()));

    let message = join_all(cat_futures)
        .await
        .iter()
        .filter_map(|res| res.as_ref().ok())
        .map(|remain_amount| {
            format!(
                "По категорії '{}' залишилося {} євро.",
                remain_amount.category.to_string(),
                remain_amount.amount
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    bot.send_message(msg.chat.id, message).await?;

    Ok(())
}
