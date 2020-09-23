use scraper::{Html, Selector};

use crate::{error::NotifyError, product::{Product, ProductPage}};

pub async fn evga_availability(provider: &ProductPage) -> Result<Product, NotifyError> {
    let resp = reqwest::get(&provider.page)
        .await
        .map_err(|e| NotifyError::WebRequestFailed(e))?
        .text()
        .await
        .map_err(|_| NotifyError::HTMLParseFailed)?;

    let document = Html::parse_document(&resp);

    let selector = Selector::parse(&provider.css_selector.clone().unwrap_or("".to_string()))
        .map_err(|_| NotifyError::HTMLParseFailed)?;
    let mut selected = document.select(&selector);
    let found = selected.next();

    if found.is_none()
        || (found.is_some()
        && !found
        .unwrap()
        .inner_html()
        .to_ascii_lowercase()
        .contains("out of stock"))
    {
        if let Some(provider) = Product::from_product(
            &provider.product_key,
            provider.product.clone(),
            provider.page.clone(),
        ) {
            return Ok(provider);
        }
    }

    Err(NotifyError::NoProductFound)
}