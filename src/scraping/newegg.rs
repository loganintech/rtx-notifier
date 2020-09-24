use lazy_static::lazy_static;
use regex::Regex;
use reqwest::StatusCode;

use crate::error::NotifyError;
use crate::product::{Product, ProductDetails};

lazy_static! {
    // Look for the javascript tag that loads the raw product data from their webservers
    static ref DETAIL_REGEX: Regex =
        Regex::new(r#"<script type="text/javascript" src="(.+ItemInfo4.+)">"#).unwrap();
}

pub async fn newegg_availability(provider: &ProductDetails) -> Result<Product, NotifyError> {
    // Open a product page
    let raw_resp = reqwest::get(&provider.page)
        .await
        .map_err(NotifyError::WebRequestFailed)?;
    let status = raw_resp.status();
    let resp = raw_resp
        .text()
        .await
        .map_err(|_| NotifyError::HTMLParseFailed)?;

    let capture = DETAIL_REGEX.captures_iter(&resp).next();
    if (!resp.contains("id=LFrame_tblMainA") && capture.is_none()) || status != StatusCode::from_u16(200).unwrap() {
        return Err(NotifyError::NoPage);
    }
    // If we found the js tag with the detail URL, act on it
    if let Some(capture) = capture {
        // Extract the URL knowing capture[0] is the entire match, not just the capturing group
        let product_url = &capture[1];

        // And load the product url
        let product_resp = reqwest::get(product_url)
            .await
            .map_err(NotifyError::WebRequestFailed)?
            .text()
            .await
            .map_err(|_| NotifyError::HTMLParseFailed)?;

        // Then look for the JSON property that shows it's in stock. Yes, we could serialize this but why bother right now
        if product_resp.contains(r#""instock":true"#) {
            return Ok(Product::NewEgg(Some(ProductDetails {
                product: provider.product.clone(),
                page: provider.page.clone(),
                ..ProductDetails::default()
            })));
        }
    }

    Err(NotifyError::NoProductFound)
}
