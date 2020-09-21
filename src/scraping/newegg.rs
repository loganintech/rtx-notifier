use crate::error::NotifyError;
use crate::provider::{NeweggProduct, ProviderType};
use crate::scraping::ProductPage;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref DETAIL_REGEX: Regex =
        Regex::new(r#"<script type="text/javascript" src="(.+ItemInfo4.+)">"#).unwrap();
}

pub async fn newegg_availability(provider: &ProductPage) -> Result<ProviderType, NotifyError> {
    let resp = reqwest::get(&provider.page)
        .await
        .map_err(|_| NotifyError::HTMLParseFailed)?
        .text()
        .await
        .map_err(|_| NotifyError::HTMLParseFailed)?;

    if let Some(capture) = DETAIL_REGEX.captures_iter(&resp).next() {
        let product_url = &capture[1];

        let resp = reqwest::get(product_url)
            .await
            .map_err(|_| NotifyError::HTMLParseFailed)?
            .text()
            .await
            .map_err(|_| NotifyError::HTMLParseFailed)?;

        if resp.contains(r#""instock":true"#) {
            return Ok(ProviderType::NewEgg(NeweggProduct::Known(
                provider.product.clone(),
                provider.page.clone(),
            )));
        }
    }

    Err(NotifyError::NoProductFound)
}
