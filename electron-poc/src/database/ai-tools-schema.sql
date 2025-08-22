-- AI Tools Launch Tracking Table
-- Tracks each unique launch of AI tools per repository/folder
-- Used to determine when to use --resume flag for tools like Claude Code

CREATE TABLE IF NOT EXISTS ai_tool_launches (
    -- Unique identifier for each launch record
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    
    -- Tool identifier (e.g., 'claude-code', 'gemini', 'qwen', 'aider', 'cline')
    tool_id TEXT NOT NULL,
    
    -- Absolute path to the repository/folder where tool was launched
    -- This is the key field for determining resume behavior
    repository_path TEXT NOT NULL,
    
    -- Number of times the tool has been launched in this repository
    launch_count INTEGER DEFAULT 1,
    
    -- Timestamp of the first launch in this repository
    first_launched_at TEXT DEFAULT CURRENT_TIMESTAMP,
    
    -- Timestamp of the most recent launch
    last_launched_at TEXT DEFAULT CURRENT_TIMESTAMP,
    
    -- Current status of the tool in this repository
    -- 'active', 'closed', 'crashed'
    status TEXT DEFAULT 'active',
    
    -- Session metadata (JSON)
    -- Can store things like last command, last file edited, etc.
    session_metadata TEXT,
    
    -- User who launched the tool (for multi-user support)
    user_id TEXT DEFAULT 'default',
    
    -- Version of the tool that was launched
    tool_version TEXT,
    
    -- Additional context (JSON)
    -- Can store environment variables, launch options, etc.
    launch_context TEXT,
    
    -- Ensure unique tool launches per repository
    UNIQUE(tool_id, repository_path, user_id),
    
    -- Foreign key to users table
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Index for fast lookups by tool and repository
CREATE INDEX IF NOT EXISTS idx_ai_tool_launches_lookup 
ON ai_tool_launches(tool_id, repository_path);

-- Index for finding recent launches
CREATE INDEX IF NOT EXISTS idx_ai_tool_launches_recent 
ON ai_tool_launches(last_launched_at DESC);

-- Index for active sessions
CREATE INDEX IF NOT EXISTS idx_ai_tool_launches_active 
ON ai_tool_launches(status) WHERE status = 'active';