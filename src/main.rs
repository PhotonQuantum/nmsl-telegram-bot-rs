use std::fs::File;
use std::sync::Arc;

use log::{error, info};
use teloxide::adaptors::AutoSend;
use teloxide::dispatching::{Dispatcher, DispatcherHandlerRx};
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::prelude::StreamExt;
use teloxide::requests::{Requester, RequesterExt};
use teloxide::types::{
    InlineQuery, InlineQueryResult, InlineQueryResultArticle, InputMessageContent,
    InputMessageContentText,
};
use teloxide::Bot;
use teloxide_listener::Listener;
use tokio_stream::wrappers::UnboundedReceiverStream;

use nmsl_core::SunBible;

#[tokio::main]
async fn main() {
    let bible_file = std::env::var("APP_BIBLE").expect("APP_BIBLE");
    pretty_env_logger::init();
    let bot = Bot::from_env().auto_send();
    let bible = Arc::new(
        SunBible::new_from_reader(File::open(bible_file).expect("bible not exist"))
            .expect("load bible"),
    );

    let dispatcher = Dispatcher::new(bot.clone()).inline_queries_handler(
        move |rx: DispatcherHandlerRx<AutoSend<Bot>, InlineQuery>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, move |query| {
                let resp = if query.update.query.is_empty() {
                    String::from("NMSL")
                } else {
                    bible.convert(query.update.query.as_str())
                };
                let hash = format!("{:x}", md5::compute(&resp));
                let result = [InlineQueryResult::Article(InlineQueryResultArticle::new(
                    hash,
                    resp.clone(),
                    InputMessageContent::Text(InputMessageContentText::new(resp)),
                ))];
                async move {
                    let resp = query
                        .requester
                        .answer_inline_query(&query.update.id, result)
                        .await;
                    if let Err(e) = resp {
                        error!("Unable to send answer: {}", e);
                    }
                }
            })
        },
    );

    info!("Starting bot...");
    dispatcher
        .setup_ctrlc_handler()
        .dispatch_with_listener(
            Listener::from_env().build(bot).await,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
}
