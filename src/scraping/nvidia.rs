use async_trait::async_trait;

use crate::{error::NotifyError, product::Product, scraping::ScrapingProvider};

static OUT_OF_STOCK_HTML: &str = r#"<div class="cta-button btn show-out-of-stock stock-grey-out" data-nvnotify-form-path="null" data-theme-override="null">Out Of Stock</div>"#;

pub struct NvidiaScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for NvidiaScraper {
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        product: &'a Product,
    ) -> Result<Product, NotifyError> {
        let text = resp
            .text()
            .await
            .map_err(|_| NotifyError::HTMLParseFailed)?;

        println!("Resp: {}", text);

        // If we find the out of stock HTML, we didn't find a product
        if text.contains(OUT_OF_STOCK_HTML) {
            return Err(NotifyError::NoProductFound);
        }

        Ok(product.clone())
    }
}
