CREATE TABLE IF NOT EXISTS `Channels` (
    `id` varchar(15) PRIMARY KEY NOT NULL,
    `parent_id` TEXT NOT NULL,
    `name` text NOT NULL,
    `description` text DEFAULT NULL,
    `channel_type` varchar(100) DEFAULT NULL
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4;
CREATE TABLE IF NOT EXISTS `User` (
    `id` int(10) NOT NULL AUTO_INCREMENT,
    `name` text NOT NULL,
    `instance` text DEFAULT NULL,
    `token` varchar(30) NOT NULL,
    `email` text NOT NULL,
    `password` text NOT NULL,
    `description` text DEFAULT NULL,
    `server` text DEFAULT NULL,
    PRIMARY KEY (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4;
CREATE TABLE IF NOT EXISTS `Server` (
    `id` varchar(30) PRIMARY KEY NOT NULL,
    `name` text NOT NULL,
    `owner` int(10) NOT NULL,
    `icon` text DEFAULT NULL,
    `members` text NOT NULL,
    KEY `server_FK` (`owner`),
    CONSTRAINT `server_FK` FOREIGN KEY (`owner`) REFERENCES `User` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4;
CREATE TABLE IF NOT EXISTS `Messages` (
    `id` varchar(32) NOT NULL,
    `content` text DEFAULT NULL,
    `message_type` varchar(32) NOT NULL,
    `edited` tinyint(1) NOT NULL,
    `timestamp` datetime NOT NULL,
    `channel_id` varchar(15) NOT NULL,
    `author_id` int(10) NOT NULL,
    PRIMARY KEY (`id`),
    KEY `Messages_FK` (`channel_id`),
    KEY `Messages_FK_1` (`author_id`),
    CONSTRAINT `Messages_FK` FOREIGN KEY (`channel_id`) REFERENCES `Channels` (`id`),
    CONSTRAINT `Messages_FK_1` FOREIGN KEY (`author_id`) REFERENCES `User` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4;
CREATE TABLE IF NOT EXISTS `Roles` (
    `id` int(11) NOT NULL AUTO_INCREMENT,
    `serverid` varchar(10) NOT NULL,
    `name` varchar(255) NOT NULL,
    `color` varchar(25) NOT NULL,
    `channel_access` text DEFAULT NULL,
    `permission` text DEFAULT NULL,
    PRIMARY KEY (`id`),
    KEY `Roles_FK` (`serverid`),
    CONSTRAINT `Roles_FK` FOREIGN KEY (`serverid`) REFERENCES `Server` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4;
