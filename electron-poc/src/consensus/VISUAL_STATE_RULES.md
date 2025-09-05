# Visual State Rules for Iterative Consensus

## Simple, Clear Rules

### 1. Initial State (When Consensus Starts)
```
Generator:  ready
Refiner:    ready  
Validator:  ready
Curator:    ready
```

### 2. Round N Execution
```
Step 1: Generator:  running → completed
Step 2: Refiner:    running → completed  
Step 3: Validator:  running → completed
Step 4: Check consensus (no visual change)
```

### 3. If Consensus NOT Achieved (Continue to Round N+1)
```
Reset for new round:
Generator:  completed → ready
Refiner:    completed → ready
Validator:  completed → ready
Curator:    ready (no change)

Then repeat Round execution...
```

### 4. If Consensus Achieved
```
Curator: ready → running → completed
```

## What We're Seeing Wrong
```
Generator:  ready (stuck)
Refiner:    completed (wrong)
Validator:  completed (wrong)
Curator:    completed (wrong)
```

## Root Causes
1. Stages aren't resetting properly at start
2. Old state persists from previous runs
3. Some initialization code is marking stages as completed

## Fix Required
1. ALWAYS reset ALL stages to 'ready' when starting consensus
2. ONLY update stages when they actually run
3. NEVER let Curator update until consensus achieved