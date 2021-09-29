CREATE OR REPLACE FUNCTION public.notify_changes()
    RETURNS TRIGGER
    LANGUAGE plpgsql
AS $function$
    BEGIN
        PERFORM pg_notify(TG_ARGV[0], 'changes detected');
        RETURN NULL;
    END;
$function$