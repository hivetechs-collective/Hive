#!/bin/bash

# Fix corrupted Rust files by removing })  pattern from line endings

FILES=(
    "./src/cache/mod.rs"
    "./src/migration/rollback.rs"
    "./src/migration/ui.rs"
    "./src/migration/analyzer.rs"
    "./src/migration/database.rs"
    "./src/migration/config.rs"
    "./src/migration/validation_suite.rs"
    "./src/migration/guide.rs"
    "./src/migration/database_impl.rs"
    "./src/migration/validator.rs"
    "./src/migration/live_test.rs"
    "./src/migration/performance.rs"
)

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "Fixing $file..."
        # Remove }) from end of each line
        sed -i '' 's/})$//' "$file"
    fi
done

echo "Files fixed!"