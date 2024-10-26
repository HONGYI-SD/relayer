-- CREATE TABLE brief
-- (
--     id         bigserial PRIMARY KEY,
--     slot       BIGINT    UNIQUE NOT NULL,
--     root_hash  VARCHAR(256) DEFAULT '',
--     hash_account       VARCHAR(256)  DEFAULT '',
--     transaction_number INT           DEFAULT 0,
--     updated_on TIMESTAMP default current_timestamp
-- );
-- CREATE INDEX index_brief_root_hash ON brief (root_hash);
-- CREATE INDEX index_brief_hash_account ON brief (hash_account);

CREATE TABLE bridge_transaction
(
    id         bigserial PRIMARY KEY,
    slot       BIGINT    UNIQUE NOT NULL,
    signature  VARCHAR(256) DEFAULT '',
    tx_hash  VARCHAR(256) DEFAULT '',
    proof  VARCHAR(256) DEFAULT '',
    updated_on TIMESTAMP default current_timestamp
)