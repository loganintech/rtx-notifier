use async_trait::async_trait;

use crate::{
    error::NotifyError,
    product::Product,
    scraping::ScrapingProvider,
};

pub struct BnHScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for BnHScraper {
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        product: &'a Product,
    ) -> Result<Product, NotifyError> {
        let resp = resp
            .text()
            .await
            .map_err(|_| NotifyError::HTMLParseFailed)?;

        println!("BNH: {}", resp);
        Err(NotifyError::NoProductFound)
    }
}
