CREATE OR REPLACE FUNCTION public.notify_changes()
    RETURNS TRIGGER
    LANGUAGE plpgsql
AS $$
    BEGIN
        PERFORM pg_notify(TG_ARGV[0], 'changes detected');
        RETURN NULL;
    END;
$$

CREATE OR REPLACE FUNCTION public.create_trigger (id TEXT, rel_name TEXT)
	RETURNS void
	AS $$
BEGIN
	EXECUTE format('CREATE TRIGGER %I
 	AFTER INSERT
 	OR UPDATE
 	OR DELETE
 	OR TRUNCATE ON %I
 	FOR EACH STATEMENT
 	EXECUTE FUNCTION public.notify_changes (%I);', 'ws_trigger_for_' || id, rel_name, id);
END;
$$
LANGUAGE plpgsql;