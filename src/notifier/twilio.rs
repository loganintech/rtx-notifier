use twilio::OutboundMessage;

use crate::{NotifyError, product::Product, Subscriber};

pub async fn send_twilio_message(
    product: &Product,
    client: &twilio::Client,
    subscriber: &Subscriber,
    from_phone: &str,
) -> Result<(), NotifyError> {
    let message = &product.new_stock_message();
    // And send our text message
    client
        .send_message(OutboundMessage::new(
            from_phone, // If this unwrap panics someone (probably Logan), has severely broken the twilio integration
            &subscriber.to_phone_number,
            message,
        ))
        .await
        .map_err(NotifyError::TwilioSend)?;

    println!(
        "Sent [{}] message to {}",
        message, subscriber.to_phone_number
    );

    Ok(())
}
