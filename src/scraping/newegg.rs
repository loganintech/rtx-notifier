use lazy_static::lazy_static;
use regex::Regex;

use crate::error::NotifyError;
use crate::product::{Product, ProductPage};
use crate::ProductDetails;

lazy_static! {
    // Look for the javascript tag that loads the raw product data from their webservers
    static ref DETAIL_REGEX: Regex =
        Regex::new(r#"<script type="text/javascript" src="(.+ItemInfo4.+)">"#).unwrap();
}

pub async fn newegg_availability(provider: &ProductPage) -> Result<Product, NotifyError> {
    // Open a product page
    let resp = reqwest::get(&provider.page)
        .await
        .map_err(|e| NotifyError::WebRequestFailed(e))?
        .text()
        .await
        .map_err(|_| NotifyError::HTMLParseFailed)?;

    // If we found the js tag with the detail URL, act on it
    if let Some(capture) = DETAIL_REGEX.captures_iter(&resp).next() {
        // Extract the URL knowing capture[0] is the entire match, not just the capturing group
        let product_url = &capture[1];

        // And load the product url
        let resp = reqwest::get(product_url)
            .await
            .map_err(|e| NotifyError::WebRequestFailed(e))?
            .text()
            .await
            .map_err(|_| NotifyError::HTMLParseFailed)?;

        // Then look for the JSON property that shows it's in stock. Yes, we could serialize this but why bother right now
        if resp.contains(r#""instock":true"#) {
            return Ok(Product::NewEgg(Some(ProductDetails {
                product: provider.product.clone(),
                page: provider.page.clone(),
                ..ProductDetails::default()
            })));
        }
    }

    Err(NotifyError::NoProductFound)
}
