-- name: 1.0-extrinsic-type
CREATE TABLE types
(
    id   OID PRIMARY KEY,
    name TEXT
);

-- name: 1.1-in-progress
INSERT INTO types (id, name)
VALUES ('1', 'Tokens');

-- name: 1.2-in-progress
INSERT INTO types (id, name)
VALUES ('2', 'Nft');

-- name: 2.0-message-status
CREATE TABLE request_status
(
    id   OID PRIMARY KEY,
    name TEXT
);

-- name: 2.1-in-progress
INSERT INTO request_status (id, name)
VALUES ('1', 'Got');

-- name: 2.2-in-progress
INSERT INTO request_status (id, name)
VALUES ('2', 'InProgress');

-- name: 2.3-in-progress
INSERT INTO request_status (id, name)
VALUES ('3', 'Success');

-- name: 2.4-in-progress
INSERT INTO request_status (id, name)
VALUES ('4', 'Error');

-- name: 2.5-in-progress
INSERT INTO request_status (id, name)
VALUES ('5', 'RollbackSuccess');

-- name: 2.6-in-progress
INSERT INTO request_status (id, name)
VALUES ('6', 'RollbackError');


-- name: 3.1-extrinsics-realis
CREATE TABLE extrinsics_realis
(
    hash         TEXT PRIMARY KEY,
    block        OID,
    from_account TEXT,
    to_account   TEXT,
    value        JSONB,
    type         OID,
    status       OID,
    CONSTRAINT fk_extrinsic_status_realis
        FOREIGN KEY (status) REFERENCES request_status (id),
    CONSTRAINT fk_extrinsic_type_realis
            FOREIGN KEY (type) REFERENCES types (id)
);

-- name: 3.2-extrinsics-bsc
CREATE TABLE extrinsics_bsc
(
    hash         TEXT PRIMARY KEY,
    block        OID,
    from_account TEXT,
    to_account   TEXT,
    value        JSONB,
    type         OID,
    status       OID,
    CONSTRAINT fk_extrinsic_status_bsc
        FOREIGN KEY (status) REFERENCES request_status (id),
    CONSTRAINT fk_extrinsic_type_bsc
        FOREIGN KEY (type) REFERENCES types (id)
);

-- name: 4.1-bsc-realis
CREATE TABLE blocks_realis
(
    block OID
);

-- name: 4.2-blocks-bsc
CREATE TABLE blocks_bsc
(
    block OID
);