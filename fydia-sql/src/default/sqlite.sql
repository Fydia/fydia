CREATE TABLE IF NOT EXISTS "Channels" (
  id TEXT(15) NOT NULL PRIMARY KEY,
  serverid TEXT(10) NOT NULL,
  name TEXT NOT NULL,
  description TEXT,
  "type" TEXT(15) NOT NULL
);
CREATE TABLE IF NOT EXISTS "User" (
  id INTEGER NOT NULL PRIMARY KEY,
  name TEXT NOT NULL,
  "instance" TEXT,
  token TEXT(30) NOT NULL,
  email TEXT NOT NULL,
  password TEXT NOT NULL,
  description TEXT,
  server TEXT
);
CREATE TABLE IF NOT EXISTS "Server" (
  id TEXT(30) NOT NULL,
  shortid TEXT(10),
  name TEXT NOT NULL,
  owner INTEGER NOT NULL,
  icon TEXT,
  members TEXT NOT NULL,
  CONSTRAINT Server_FK FOREIGN KEY (owner) REFERENCES "User"(id)
);
CREATE TABLE IF NOT EXISTS `Messages` (
  id TEXT(32) NOT NULL PRIMARY KEY,
  content TEXT,
  message_type TEXT(15) NOT NULL,
  edited INTEGER(1) NOT NULL,
  `timestamp` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  channel_id TEXT(15) NOT NULL,
  author_id INTEGER NOT NULL,
  CONSTRAINT `Messages_FK` FOREIGN KEY (channel_id) REFERENCES "Channels"(id),
  CONSTRAINT `Messages_FK_1` FOREIGN KEY (author_id) REFERENCES "User"(id)
);