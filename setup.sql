CREATE DATABASE projectnotifier;

CREATE TABLE config
(
    id                SERIAL PRIMARY KEY NOT NULL UNIQUE,
    last_seen_evga    TIMESTAMPTZ        NOT NULL,
    last_seen_newegg  TIMESTAMPTZ        NOT NULL,
    last_seen_asus    TIMESTAMPTZ        NOT NULL,
    twilio_auth_token TEXT               NOT NULL,
    twilio_account_id TEXT               NOT NULL,
    imap_username     TEXT               NOT NULL,
    imap_password     TEXT               NOT NULL,
    imap_host         TEXT               NOT NULL,
    from_phone_number TEXT               NOT NULL
);

CREATE TABLE subscriber
(
    id              SERIAL PRIMARY KEY NOT NULL UNIQUE,
    service         TEXT               NOT NULL,
    to_phone_number TEXT               NOT NULL,
    UNIQUE (service, to_phone_number)
);

SELECT *
FROM config
ORDER BY id DESC
LIMIT 1;

INSERT INTO config (id, last_seen_evga, last_seen_newegg, last_seen_asus, twilio_auth_token, twilio_account_id,
                    imap_username, imap_password, imap_host, from_phone_number)
values (1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP,
        '44b4d93e7a75c8cf7f523d0bb6b845da', 'AC5061e0a7990f09c8d6a6bf644efcc7c7', 'hookintopython', 'xf2JYTt6OIMm',
        'imap.gmail.com', '+17792446755');

INSERT INTO subscriber (id, service, to_phone_number)
VALUES (2, 'newegg', '+14088333405'),
       (1, 'evga', '+14088333405'),
       (3, 'evgaa', '+18054594801'),
       (4, 'newegga', '+18054594801');
