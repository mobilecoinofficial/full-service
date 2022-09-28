-- Old versions of the database used an empty string to indicate no assigned_subaddress_b58 but that violates
-- foreign key constraints. A previous migration changed the assigned_subaddress_b58 field to be NULLable bue
-- forgot to update existing rows.
UPDATE transaction_logs SET assigned_subaddress_b58=NULL where assigned_subaddress_b58='';
