use crate::configuration::config_model::JSONConfiguration;
use crate::consumer::handle_message_result::action_result;
use crate::consumer::handle_message_result::HandleMessageResult;
use crate::consumer::setup::setup_consumer;
use lapin::message::Delivery;
use lapin::options::BasicConsumeOptions;
use lapin::types::FieldTable;
use lapin::Channel;
use lapin::Consumer;

pub type DeliveredMessage = Delivery;

/// # Create a consumer
/// Returns consumer
pub async fn create_consumer(queue_name: &str, channel: &Channel) -> Consumer {
    let consumer = channel
        .basic_consume(
            queue_name,
            "",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("basic_consume");
    return consumer;
}

/// This is the function that the consuming is happening
/// There are two (infinite) loops here, a. one around the consumer, which
/// while the connection is healthy keeps dequeuing messages
/// b. one that once the connection dies, restarts the whole process of getting a channel and
/// setting up the consumer.
///
/// this function also allows you to pass an argument of type T into the handler
pub async fn consume_with_option<T>(
    config: &JSONConfiguration,
    handler: &dyn Fn(&Delivery, Option<&T>) -> HandleMessageResult,
    args: Option<&T>,
) -> HandleMessageResult {
    loop {
        let model = setup_consumer(
            config.connection.clone(),
            config.binding.clone(),
            config.declare.clone(),
        )
        .await;

        println!("[{}] - {:?}", line!(), model.channel.status().state());

        let consumer = create_consumer(&config.binding.queue, &model.channel).await;

        println!("[{}] - {:?}", line!(), consumer.tag());

        for message in consumer {
            match message {
                Ok((channel, delivery)) => {
                    action_result(handler(&delivery, args), &channel, delivery.delivery_tag).await;
                }
                Err(why) => {
                    println!("[{}] channel status: {:?}", line!(), why);
                }
            };
        }
    }
}

/// This is the function that the consuming is happening
/// There are two (infinite) loops here, a. one around the consumer, which
/// while the connection is healthy keeps dequeuing messages
/// b. one that once the connection dies, restarts the whole process of getting a channel and
/// setting up the consumer
pub async fn consume(
    config: &JSONConfiguration,
    handler: &dyn Fn(&Delivery) -> HandleMessageResult,
) -> HandleMessageResult {
    loop {
        let model = setup_consumer(
            config.connection.clone(),
            config.binding.clone(),
            config.declare.clone(),
        )
        .await;

        println!("[{}] - {:?}", line!(), model.channel.status().state());

        let consumer = create_consumer(&config.binding.queue, &model.channel).await;

        println!("[{}] - {:?}", line!(), consumer.tag());

        for message in consumer {
            match message {
                Ok((channel, delivery)) => {
                    action_result(handler(&delivery), &channel, delivery.delivery_tag).await;
                }
                Err(why) => {
                    println!("[{}] channel status: {:?}", line!(), why);
                }
            };
        }
    }
}
