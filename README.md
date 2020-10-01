# RTX-Notifier

If you've stumbled here you're as frustrated as I am that the RTX cards are pretty impossible to buy. This program will monitor some retail sites and if one is found to have an RTX available, will open the page in your web browser.

## Supported Products and Stores

|Type|Products|NewEgg|Amazon|Evga|Nvidia|B&H|BestBuy
|---|---|---|---|---|---|---|---|
| 3080 | Founders Edition | | | | `*` | | | 
| 3080 | EVGA FTW3 Ultra | `*` | `*` | `*` | | | |
| 3080 | EVGA FTW3 Gaming | `*` | `*` | `*` | | | |
| 3080 | EVGA XC3 Gaming | `*` | `*` | `*` | | | `*` |
| 3080 | EVGA XC3 Black | `*` | `*` | `*` | | | |
| 3080 | EVGA XC3 Ultra | `*` |  | `*` | | | |
| 3080 | ASUS TUF Gaming | `*` | `*` | | | | |
| 3080 | ASUS TUF OC |  | `*` | | | | |
| 3080 | MSI Gaming X TRIO | `*` | | | | `*` | |
| 3080 | MSI Ventus 3X | `*` | | | | `*` | |
| 3080 | MSI Torx | | `*` | | | |
| 3080 | MSI Tri-Frozr | | `*` | | | |
| 3080 | GIGABYTE Eagle OC | `*` | `*` | | | `*` | |
| 3080 | GIGABYTE Gaming | `*` | | | | | |
| 3080 | GIGABYTE Gaming OC | `*` | `*` | | | `*` | |
| 3080 | PNY Gaming Epic-X | `*` | `*` | | | | `*` |
| 3080 | PNY Gaming Epic-X Tri-Fan | | `*` | | | | |
| 3080 | ZOTAC Trinity | `*` | `*` | | | `*` | |
| 3080 | ZOTAC Trinity OC | `*` | `*` | | | |
| 3090 | Founders Edition | | | | `*` | | | 
| 3090 | EVGA FTW3 Ultra | `*` | | `*` | | | | 
| 3090 | EVGA FTW3 Gaming | `*` | | `*` | | | | 
| 3090 | EVGA XC3 Ultra | `*` | | `*` | | | | 
| 3090 | EVGA XC3 Gaming | `*` | | `*` | | | | 
| 3090 | EVGA XC3 Black | `*` | | `*` | | | |
| 3090 | ASUS STRIX | `*` | | | | | |  
| 3090 | ASUS TUF Gaming | `*` | `*` | | | | |
| 3090 | ASUS TUF OC | `*` | `*` | | | `*` | |
| 3090 | MSI Ventus 3X OC | `*` | | | | | |
| 3090 | MSI Ventus 3X | `*` | | | | `*` | |
| 3090 | MSI Gaming X TRIO | `*` | | | | `*` | |
| 3090 | MSI Torx | | `*` | | | | |
| 3090 | MSI Twin-Frozr | | `*` | | | | |
| 3090 | GIGABYTE Eagle | `*` | | | | | |
| 3090 | GIGABYTE Eagle OC | `*` | `*` | | | `*` | |
| 3090 | GIGABYTE Windforce 3X | `*` | `*` | | | `*` | |
| 3090 | PNY Epic-X | | `*` | | | | | 
| 3090 | ZOTAC Trinity | `*` | | | | `*` | |

## Setup

Make sure to rename `example_config.json` to `config.json` otherwise the script will exit. There are comments within it describing the basic options, as well as a snippet below with the same descriptions.

Most config items are optional and won't be used if omitted. For example, without imap or twilio config, mail and text integrations are disabled automatically. If the discord url is missing, no attempt will be made to post to a channel.



### Minmal Config Example

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

    // This delays certain providers. These are added automatically when a ratelimit is hit for a service
    "ratelimit_keys": {
      "bnh": "2020-09-28T00:49:28.888712-07:00",
      "amazon": "2020-09-28T00:52:28.888712-07:00"
    },

    // Optional SOCKS5 Proxy URL
    "proxy_url": "socks5://127.0.0.1:9050"
  },
  "subscribers": [
    {
      // List of services that the recipient wants a text about
      "service": [
        "newegg",
        "bestbuy",
        "nvidia",
        "bnh",
        "amazon"
      ],
      // The phone number to send a text to
      "to_phone_number": "+15556667777",
      // Whether or not the bot should send a text to that person
      "active": true
    }
  ],
  // I recommend copying the providers from the `example_config.json`, Otherwise you have a lot of writing to do
  "products": [
    {
      // Website to scrape product for, must be one of [evga, newegg, amazon, bnh, bestbuy, nvidia]
      "evga": {
        // The name of the product for display purposes
        "product": "EVGA 3090 FTW3 GAMING",
        // The URL of the product to scrape
        "page": "https://www.evga.com/products/product.aspx?pn=24G-P5-3985-KR",
        // Active and Active Chance work together. Active overrides active_chance. If active is false, it will not scrape
        // If active is null, it WILL scrape. This is to be considered "no input"
        // active_chance is a value between 0 and 10. 0 being never scrape, 10 being always scrape
        // A number outside of the range will constrain to the outer edges of the range
        "active": true,
        "active_chance": 7
      }
    }
  ]
}
```