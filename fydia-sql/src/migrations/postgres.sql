CREATE TABLE IF NOT EXISTS "Channels" (
    "id" varchar(15) PRIMARY KEY NOT NULL,
    "parent_id" TEXT NOT NULL,
    "name" text NOT NULL,
    description text DEFAULT NULL,
    "channel_type" varchar(100) DEFAULT NULL
);
CREATE TABLE IF NOT EXISTS "User" (
    "id" SERIAL PRIMARY KEY NOT NULL,
    "name" text NOT NULL,
    "instance" text DEFAULT NULL,
    "token" varchar(30) NOT NULL UNIQUE,
    "email" text NOT NULL,
    "password" text NOT NULL,
    "description" text DEFAULT NULL
);
CREATE TABLE IF NOT EXISTS "Server" (
    "id" varchar(30) PRIMARY KEY NOT NULL,
    "name" text NOT NULL,
    "owner" int NOT NULL,
    "icon" text DEFAULT NULL,
    CONSTRAINT server_FK FOREIGN KEY ("owner") REFERENCES "User" ("id")
);
CREATE TABLE IF NOT EXISTS "Messages" (
    "id" varchar(32) NOT NULL,
    "content" text DEFAULT NULL,
    "message_type" varchar(32) NOT NULL,
    "edited" BOOLEAN NOT NULL,
    "timestamp" timestamp NOT NULL,
    "channel_id" varchar(15) NOT NULL,
    "author_id" int NOT NULL,
    PRIMARY KEY ("id"),
    CONSTRAINT Messages_FK FOREIGN KEY ("channel_id") REFERENCES "Channels" ("id"),
    CONSTRAINT Messages_FK_1 FOREIGN KEY ("author_id") REFERENCES "User" ("id")
);

CREATE TABLE "Members" (
  "serverid" varchar(30) NOT NULL,
  "userid" int NOT NULL,
  UNIQUE("serverid","userid"),
  CONSTRAINT "Members_FK" FOREIGN KEY ("userid") REFERENCES "User" ("id"),
  CONSTRAINT "Members_FK_1" FOREIGN KEY ("serverid") REFERENCES "Server" ("id")
);