-- The first migration file for Rust version of the app.
-- Legacy schema is available in the Git history.

-- Detect whether legacy tables exist (for conditional copy)
SET @has_users    = (SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = DATABASE() AND table_name = 'users');
SET @has_posts    = (SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = DATABASE() AND table_name = 'posts');
SET @has_settings = (SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = DATABASE() AND table_name = 'user_settings');
SET @has_tokens   = (SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = DATABASE() AND table_name = 'user_tokens');

-- Create new-schema tables (temporary names)
CREATE TABLE `posts_new` (
  `id` INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `at_proto_uri` VARCHAR(8192) NOT NULL,
  `traq_message_id` BINARY(16) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `users_new` (
  `id` BINARY(16) NOT NULL PRIMARY KEY
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `user_settings_new` (
  `id` INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `did` VARCHAR(2048) NOT NULL,
  `target_channel_id` BINARY(16) NOT NULL,
  `user_id` BINARY(16) NOT NULL UNIQUE,
  FOREIGN KEY (`user_id`) REFERENCES `users_new`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `user_tokens_new` (
  `id` INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `access_token` VARCHAR(36) NOT NULL,
  `user_id` BINARY(16) NOT NULL UNIQUE,
  FOREIGN KEY (`user_id`) REFERENCES `users_new`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Migrate existing rows (hex-normalised UUID -> BINARY(16))
SET @sql_posts = IF(@has_posts > 0,
  'INSERT INTO `posts_new` (`at_proto_uri`, `traq_message_id`) SELECT `at_proto_uri`, UNHEX(REPLACE(`traq_message_id`, "-", "")) FROM `posts`',
  'SELECT 1');
PREPARE stmt_posts FROM @sql_posts;
EXECUTE stmt_posts;
DEALLOCATE PREPARE stmt_posts;

SET @sql_users = IF(@has_users > 0,
  'INSERT INTO `users_new` (`id`) SELECT UNHEX(REPLACE(`id`, "-", "")) FROM `users`',
  'SELECT 1');
PREPARE stmt_users FROM @sql_users;
EXECUTE stmt_users;
DEALLOCATE PREPARE stmt_users;

SET @sql_settings = IF(@has_settings > 0,
  'INSERT INTO `user_settings_new` (`did`, `target_channel_id`, `user_id`) SELECT `did`, UNHEX(REPLACE(`target_channel_id`, "-", "")), UNHEX(REPLACE(`user_id`, "-", "")) FROM `user_settings`',
  'SELECT 1');
PREPARE stmt_settings FROM @sql_settings;
EXECUTE stmt_settings;
DEALLOCATE PREPARE stmt_settings;

SET @sql_tokens = IF(@has_tokens > 0,
  'INSERT INTO `user_tokens_new` (`access_token`, `user_id`) SELECT `access_token`, UNHEX(REPLACE(`user_id`, "-", "")) FROM `user_tokens`',
  'SELECT 1');
PREPARE stmt_tokens FROM @sql_tokens;
EXECUTE stmt_tokens;
DEALLOCATE PREPARE stmt_tokens;

-- Drop legacy tables (disable FK checks briefly)
SET FOREIGN_KEY_CHECKS = 0;

DROP TABLE IF EXISTS `user_settings`;
DROP TABLE IF EXISTS `user_tokens`;
DROP TABLE IF EXISTS `posts`;
DROP TABLE IF EXISTS `users`;

SET FOREIGN_KEY_CHECKS = 1;

-- Promote new tables to final names
RENAME TABLE `posts_new` TO `posts`;
RENAME TABLE `users_new` TO `users`;
RENAME TABLE `user_settings_new` TO `user_settings`;
RENAME TABLE `user_tokens_new` TO `user_tokens`;

-- Add unique index on posts.at_proto_uri
CREATE UNIQUE INDEX `idx_at_proto_uri` ON `posts` (`at_proto_uri`);

-- Create tables not containing UUIDs (no migration needed)
CREATE TABLE IF NOT EXISTS `system_states` (
  `key` VARCHAR(16) NOT NULL PRIMARY KEY,
  `value` BIGINT NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Old queued_events rows use a legacy Jetstream envelope incompatible with the
-- current worker, and compatibility is explicitly not required. Drop them.
DROP TABLE IF EXISTS `queued_events`;
CREATE TABLE `queued_events` (
  `id` INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `event_json` JSON NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Remove Kysely bookkeeping tables
DROP TABLE IF EXISTS `kysely_migration`;
DROP TABLE IF EXISTS `kysely_migration_lock`;
