-- Set the first profile as default if no default exists
UPDATE consensus_profiles 
SET is_default = 1 
WHERE id = (
    SELECT id FROM consensus_profiles 
    WHERE is_default = 0 OR is_default IS NULL
    ORDER BY id 
    LIMIT 1
)
AND NOT EXISTS (
    SELECT 1 FROM consensus_profiles WHERE is_default = 1
);