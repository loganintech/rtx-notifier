use lazy_static::lazy_static;
use regex::Regex;
use reqwest::header::HeaderMap;

use crate::error::NotifyError;
use crate::Product;
use crate::provider::ProviderType;
use crate::scraping::ProductPage;

lazy_static! {
    static ref BUTTON_REGEX: Regex =
        Regex::new(r#"<div class="fulfillment.+([Ss][Oo][Ll][Dd] [Oo][Uu][Tt])</button></div></div>"#).unwrap();
}


pub async fn bestbuy_availability(provider: &ProductPage) -> Result<ProviderType, NotifyError> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("Accept", "*/*".parse().unwrap());
    headers.insert("Host", "www.bestbuy.com".parse().unwrap());
    headers.insert("User-Agent", "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:80.0) Gecko/20100101 Firefox/80.0".parse().unwrap());

    let resp = client.get(&provider.page).headers(headers)
        .send()
        .await
        .map_err(|e| NotifyError::WebRequestFailed(e))?
        .text()
        .await
        .map_err(|_| NotifyError::HTMLParseFailed)?;

    // If we can't find the sold out button, we're back in stock
    if let None = BUTTON_REGEX.captures_iter(&resp).next() {
        return Ok(ProviderType::BestBuy(Product {
            product: provider.product.clone(),
            page: provider.page.clone(),
            ..Product::default()
        }));
    }

    Err(NotifyError::NoProductFound)
}
