use std::fs::File;
use std::sync::Arc;

use log::{error, info};
use teloxide::adaptors::AutoSend;
use teloxide::dispatching2::{Dispatcher, UpdateFilterExt};
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::requests::{Requester, RequesterExt};
use teloxide::types::{
    InlineQuery, InlineQueryResult, InlineQueryResultArticle, InputMessageContent,
    InputMessageContentText, Update,
};
use teloxide::{dptree, Bot};
use teloxide_listener::Listener;

use nmsl_core::SunBible;

async fn handler(query: InlineQuery, bot: AutoSend<Bot>, bible: Arc<SunBible>) -> Result<(), ()> {
    let resp = if query.query.is_empty() {
        String::from("NMSL")
    } else {
        bible.convert(query.query.as_str())
    };
    let hash = format!("{:x}", md5::compute(&resp));
    let result = [InlineQueryResult::Article(InlineQueryResultArticle::new(
        hash,
        resp.clone(),
        InputMessageContent::Text(InputMessageContentText::new(resp)),
    ))];
    let resp = bot.answer_inline_query(&query.id, result).await;
    if let Err(e) = resp {
        error!("Unable to send answer: {}", e);
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let bible_file = std::env::var("APP_BIBLE").expect("APP_BIBLE");
    pretty_env_logger::init();
    let bot = Bot::from_env().auto_send();
    let bible = Arc::new(
        SunBible::new_from_reader(File::open(bible_file).expect("bible not exist"))
            .expect("load bible"),
    );

    info!("Starting bot...");
    let listener = Listener::from_env().build(bot.clone()).await;
    Dispatcher::builder(
        bot,
        dptree::entry().branch(Update::filter_inline_query().endpoint(handler)),
    )
    .dependencies(dptree::deps![bible])
    .build()
    .setup_ctrlc_handler()
    .dispatch_with_listener(
        listener,
        LoggingErrorHandler::with_custom_text("An error from the update listener"),
    )
    .await;
}
