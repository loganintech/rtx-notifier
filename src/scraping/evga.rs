use async_trait::async_trait;
use scraper::{Html, Selector};

use crate::{
    error::NotifyError,
    product::{Product, ProductDetails},
    scraping::ScrapingProvider,
};

pub struct EvgaScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for EvgaScraper {
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        details: &'a ProductDetails,
    ) -> Result<Product, NotifyError> {
        let resp = resp
            .text()
            .await
            .map_err(|_| NotifyError::HTMLParseFailed)?;
        if resp.contains("There has been an error while requesting your page") {
            return Err(NotifyError::NoProductFound);
        }

        let document = Html::parse_document(&resp);

        let selector = Selector::parse(
            &details
                .css_selector
                .clone()
                .unwrap_or_else(|| "".to_string()),
        )
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
            if let Some(product) = Product::from_product(
                &details.product_key,
                details.product.clone(),
                details.page.clone(),
            ) {
                return Ok(product);
            }
        }

        Err(NotifyError::NoProductFound)
    }
}
