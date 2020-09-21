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
    active          BOOL               NOT NULL,
    UNIQUE (service, to_phone_number)
);

SELECT *
FROM subscriber
WHERE service = 'evga'
  AND active = true;

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

CREATE TABLE productpage
(
    id           SERIAL PRIMARY KEY NOT NULL UNIQUE,
    product      TEXT               NOT NULL,
    page         TEXT               NOT NULL,
    product_key  TEXT               NOT NULL,
    css_selector TEXT               NOT NULL,
    active       BOOL               NOT NULL
);


INSERT INTO productpage (id, product, page, product_key, css_selector, active)
VALUES (1, 'EVGA RTX 3080 FTW3 Ultra', 'https://www.evga.com/products/product.aspx?pn=10G-P5-3897-KR', 'evgartx',
        '#LFrame_pnlOutOfStock', 'true'),
       (2, 'EVGA RTX 3080 XC3 Black', 'https://www.evga.com/products/product.aspx?pn=10G-P5-3881-KR', 'evgartx',
        '#LFrame_pnlOutOfStock', 'true'),
       (3, 'EVGA RTX 3080 XC3 Gaming', 'https://www.evga.com/products/product.aspx?pn=10G-P5-3883-KR', 'evgartx',
        '#LFrame_pnlOutOfStock', 'true'),
       (4, 'EVRA RTX 3080 XC3 Ultra', 'https://www.evga.com/products/product.aspx?pn=10G-P5-3885-KR', 'evgartx',
        '#LFrame_pnlOutOfStock', 'true'),
       (5, 'MSI RTX 3080 GAMING X TRIO',
        'https://www.newegg.com/msi-geforce-rtx-3080-rtx-3080-gaming-x-trio-10g/p/N82E16814137597', 'neweggrtx', '.',
        'true'),
       (6, 'NVIDIA 3080 FE', 'https://www.nvidia.com/en-es/geforce/graphics-cards/30-series/rtx-3080', 'nvidia', '.',
        'true'),
       (7, 'MSI RTX 3080 Ventus 3X',
        'https://www.newegg.com/msi-geforce-rtx-3080-rtx-3080-ventus-3x-10g-oc/p/N82E16814137598', 'neweggrtx', '.',
        'true'),
       (8, 'MSI RTX 3080 GV-N3080GAMING OC',
        'https://www.newegg.com/gigabyte-geforce-rtx-3080-gv-n3080gaming-oc-10gd/p/N82E16814932329', 'neweggrtx', '.',
        'true'),
       (9, 'EVGA RTX 3080 BLACK GAMING',
        'https://www.newegg.com/gigabyte-geforce-rtx-3080-gv-n3080eagle-oc-10gd/p/N82E16814932330', 'neweggrtx', '.',
        'true'),
       (10, 'EVGA RTX 3080 XC3 Gaming', 'https://www.newegg.com/evga-geforce-rtx-3080-10g-p5-3883-kr/p/N82E16814487521',
        'neweggrtx', '.', 'true'),
       (11, 'ZOTAC GAMING RTX Trinity',
        'https://www.newegg.com/zotac-geforce-rtx-3080-zt-a30800d-10p/p/N82E16814500502', 'neweggrtx', '.', 'true'),
       (12, 'ASUS TUF Gaming 3080',
        'https://www.newegg.com/asus-geforce-rtx-3080-tuf-rtx3080-10g-gaming/p/N82E16814126453', 'neweggrtx', '.',
        'true'),
       (13, 'ASUS ROG Strix 3080',
        'https://www.newegg.com/asus-geforce-rtx-3080-rog-strix-rtx3080-o10g-gaming/p/N82E16814126457', 'neweggrtx',
        '.', 'true'),
       (14, 'ASUS ROG Strix 3090',
        'https://www.newegg.com/asus-geforce-rtx-3090-rog-strix-rtx3090-o24g-gaming/p/N82E16814126456', 'neweggrtx',
        '.', 'true'),
       (15, 'MSI RTX 3090 VENTUS 3X',
        'https://www.newegg.com/msi-geforce-rtx-3090-rtx-3090-ventus-3x-24g/p/N82E16814137599', 'neweggrtx', '.',
        'true'),
       (16, 'MSI RTX 3090 GAMING X TRIO',
        'https://www.newegg.com/msi-geforce-rtx-3090-rtx-3090-gaming-x-trio-24g/p/N82E16814137595', 'neweggrtx', '.',
        'true'),
       (17, 'GIGABYTE RTX 3090',
        'https://www.newegg.com/gigabyte-geforce-rtx-3090-gv-n3090gaming-oc-24gd/p/N82E16814932327', 'neweggrtx', '.',
        'true'),
       (18, 'GIGABYTE RTX 3090 Eagle',
        'https://www.newegg.com/gigabyte-geforce-rtx-3090-gv-n3090eagle-oc-24gd/p/N82E16814932328', 'neweggrtx', '.',
        'true'),
       (19, 'MSI RTX 3090 VENTUS 3X',
        'https://www.newegg.com/msi-geforce-rtx-3090-rtx-3090-ventus-3x-24g-oc/p/N82E16814137596', 'neweggrtx', '.',
        'true'),
       (20, 'ASUS TUF 3090', 'https://www.newegg.com/asus-geforce-rtx-3090-tuf-rtx3090-24g-gaming/p/N82E16814126455',
        'neweggrtx', '.', 'true'),
       (21, 'ZOTAC RTX 3090 Trinity', 'https://www.newegg.com/zotac-geforce-rtx-3090-zt-a30900d-10p/p/N82E16814500503',
        'neweggrtx', '.', 'true'),
       (22, 'ASUS TUF 3090', 'https://www.newegg.com/asus-geforce-rtx-3090-tuf-rtx3090-o24g-gaming/p/N82E16814126454',
        'neweggrtx', '.', 'true'),
       (23, 'MSI RTX 3090 VENTUS 3X',
        'https://www.newegg.com/msi-geforce-rtx-3080-rtx-3080-ventus-3x-10g/p/N82E16814137600', 'neweggrtx', '.',
        'true'),
       (24, 'ASUS TUF 3080', 'https://www.newegg.com/asus-geforce-rtx-3080-tuf-rtx3080-o10g-gaming/p/N82E16814126452',
        'neweggrtx', '.', 'true'),
       (25, 'ZOTAC RTX Gaming Trinity OC',
        'https://www.newegg.com/zotac-geforce-rtx-3080-zt-t30800j-10p/p/N82E16814500504', 'neweggrtx', '.', 'true');
