ALTER TABLE website_traces
DROP CONSTRAINT IF EXISTS website_traces_user_id_fkey;
ALTER TABLE website_traces
DROP COLUMN IF EXISTS user_id CASCADE;
DROP TABLE IF EXISTS users CASCADE;
DROP TYPE user_role;
