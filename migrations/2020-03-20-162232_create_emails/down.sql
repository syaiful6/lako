-- This file should undo anything in `up.sql`
DROP TRIGGER trigger_lako_reconfirm ON emails;
DROP FUNCTION IF EXISTS lako_reconfirm_email_on_email_change();
DROP INDEX emails_user_id_fk;
DROP table emails;
DROP FUNCTION IF EXISTS lako_random_string();