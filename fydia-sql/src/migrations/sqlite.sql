CREATE TABLE IF NOT EXISTS "Channels" (
    id TEXT(15) NOT NULL PRIMARY KEY,
    "parent_id" TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    "channel_type" TEXT(15) NOT NULL
);
CREATE TABLE IF NOT EXISTS "User" (
    id INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    "instance" TEXT,
    token TEXT(30) NOT NULL UNIQUE,
    email TEXT NOT NULL,
    password TEXT NOT NULL,
    description TEXT
);
CREATE TABLE IF NOT EXISTS "Server" (
    id TEXT(30) NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    owner INTEGER NOT NULL,
    icon TEXT,
    CONSTRAINT Server_FK FOREIGN KEY (owner) REFERENCES "User"(id)
);
CREATE TABLE IF NOT EXISTS `Messages` (
    id TEXT(32) NOT NULL PRIMARY KEY,
    content TEXT,
    message_type TEXT(15) NOT NULL,
    edited INTEGER(1) NOT NULL,
    `timestamp` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    channel_id TEXT(15) NOT NULL,
    author_id INTEGER NOT NULL,
    CONSTRAINT `Messages_FK` FOREIGN KEY (channel_id) REFERENCES "Channels"(id),
    CONSTRAINT `Messages_FK_1` FOREIGN KEY (author_id) REFERENCES "User"(id)
);
CREATE TABLE IF NOT EXISTS "Roles" (
    id INTEGER NOT NULL,
    serverid TEXT(10) NOT NULL,
    name TEXT(255) NOT NULL,
    color TEXT(25) NOT NULL,
    channel_access TEXT,
    permission TEXT,
    CONSTRAINT Roles_PK PRIMARY KEY (id),
    CONSTRAINT Server_FK FOREIGN KEY (serverid) REFERENCES "Server"(id)
);

CREATE TABLE "Members" (
  "serverid" varchar(30) NOT NULL,
  "userid" int(10) NOT NULL,
  UNIQUE(serverid,userid),
  CONSTRAINT Members_FK FOREIGN KEY (userid) REFERENCES "User" (id),
  CONSTRAINT Members_FK_1 FOREIGN KEY (serverid) REFERENCES "Server" (id)
);