-- CREATE TABLE brief
-- (
--     id         bigserial PRIMARY KEY,
--     slot       BIGINT    UNIQUE NOT NULL,
--     tx_info_hash  BYTEA     NOT NULL,
--     hash_account       VARCHAR(256)  DEFAULT '',
--     transaction_number INT           DEFAULT 0,
--     updated_on TIMESTAMP default current_timestamp
-- );
-- CREATE INDEX index_brief_root_hash ON brief (root_hash);
-- CREATE INDEX index_brief_hash_account ON brief (hash_account);

CREATE TABLE bridge_transaction
(
    id         bigserial PRIMARY KEY,
    slot       BIGINT    NOT NULL,
    signature  VARCHAR(256) UNIQUE DEFAULT '',
    tx_info_hash     BYTEA     NOT NULL,
    proof  VARCHAR(256) DEFAULT '',
    is_generated_proof BOOLEAN NOT NULL, 
    current_mt_root BYTEA,
    updated_on TIMESTAMP default current_timestamp
)