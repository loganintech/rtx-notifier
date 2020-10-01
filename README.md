# RTX-Notifier

If you've stumbled here you're as frustrated as I am that the RTX cards are pretty impossible to buy. This program will monitor some retail sites and if one is found to have an RTX available, will open the page in your web browser.

The config format is self explanitory, but make sure to rename it to `config.json` from `example_config.json`.

If you leave out any twilio or imap credentials the bot will only perform scraping and opening the web pages on Mac or Windows.

If you want to contribute I welcome merge requests. 

## Supported Stores


## Minmal Config Example

```json5
{
  "application_config": {
    // Some default values here, these are really only used for email. You don't need to change them
    "last_seen_evga": "2020-09-20T18:52:49.444913-07:00",
    "last_seen_newegg": "2020-09-20T18:51:06.486222-07:00",
    "last_seen_asus": "2020-09-19T18:43:58.644-07:00",

    // No notifications are sent within 30 minutes of this timestamp, to prevent spam. Only applies to Twilio
    "last_notification_sent": "2020-09-20T14:47:24.571591-07:00",

    // If any of these properties are null, the twilio integration is skipped
    // The auth token from your twilio account
    "twilio_auth_token": null,
    // The account id from your twilio account (as a string) ""
    "twilio_account_id": null,
    // The from phone number in your twilio account (as a string "+15556667777")
    "from_phone_number": null,

    // If any of these properties are null, no attempt is made to read emails from the IMAP integration
    "imap_username": null,
    "imap_password": null,
    "imap_host": null,
    "imap_port": 993,

    // These are personal choices. I recommend daemon mode if you're just running locally (it will keep running, and check for new products at the specified timeout)
    "should_open_browser": true,
    "daemon_mode": true,
    "daemon_timeout": 30,

    // Webhook URL to send to discord
    "discord_url": null,

    // This delays ALL scraping. It must be set manually
    "scraping_timeout": "2020-09-28T00:49:28.888712-07:00",

    // This delays certain providers. These are added automatically when a ratelimit is hit for a product
    "ratelimit_keys": {
      "bnh": "2020-09-28T00:49:28.888712-07:00",
      "amazon": "2020-09-28T00:52:28.888712-07:00"
    },

    // Optional SOCKS5 Proxy URL
    "proxy_url": "socks5://127.0.0.1:9050"
  },
  // I recommend copying the providers from the `example_config.json`, Otherwise you have a lot of writing to do
  "subscribers": [
    {
      // Website to scrape product for, must be one of [evga, newegg, amazon, bnh, bestbuy, nvidia]
      "evga": {
        // The name of the product for display purposes
        "product": "EVGA 3090 FTW3 GAMING",
        // The URL of the product to scrape
        "page": "https://www.evga.com/products/product.aspx?pn=24G-P5-3985-KR",
        // Used by the EVGA scraper to identify which CSS selector shows an out of stock message
        "css_selector": "#LFrame_pnlOutOfStock",
        // Active and Active Chance work together. Active overrides active_chance. If active is false, it will not scrape
        // If active is null, it WILL scrape. This is to be considered "no input"
        // active_chance is a value between 0 and 10. 0 being won't ever scrape, 10 being always scrape
        // A number outside of the range will constrain to the outer edges of the range
        "active": true,
        "active_chance": 7
      }
    }
  ]
}
```