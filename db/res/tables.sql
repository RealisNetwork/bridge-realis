-- name: 1-extrinsics-realis
CREATE TABLE extrinsics_realis (
     hash TEXT PRIMARY KEY,
     block OID,
     from_account TEXT,
     to_account  TEXT,
     value  jsonb,
     type OID,
     status  OID NOT NULL
);

-- name: 1.1-extrinsics-bsc
CREATE TABLE extrinsics_bsc (
     hash TEXT PRIMARY KEY,
     from_account TEXT,
     block OID,
     to_account  TEXT,
     value  jsonb,
     type OID,
     status  OID NOT NULL
);

-- name: 1.3-bsc-realis
CREATE TABLE blocks_realis (
    block OID
);

-- name: 1.4-blocks-bsc
CREATE TABLE blocks_bsc (
    block OID
);

-- name: 2-message-status
CREATE TABLE request_status
(
    id   oid PRIMARY KEY,
    name text
);

-- name: 2.0-in-progress
INSERT INTO request_status (id, name)
VALUES ('0', 'Got');

-- name: 2.1-in-progress
INSERT INTO request_status (id, name)
VALUES ('1', 'InProgress');

-- name: 2.2-in-progress
INSERT INTO request_status (id, name)
VALUES ('2', 'Success');

-- name: 2.3-in-progress
INSERT INTO request_status (id, name)
VALUES ('3', 'Error');

-- name: 2.4-in-progress
INSERT INTO request_status (id, name)
VALUES ('4', 'Rollbacked');

-- name: 3-extrinsic-type
CREATE TABLE types
(
    id   oid PRIMARY KEY,
    name text
);

-- name: 3.1-in-progress
INSERT INTO types (id, name)
VALUES ('1', 'Tokens');

-- name: 2.4-in-progress
INSERT INTO types (id, name)
VALUES ('2', 'Nft');