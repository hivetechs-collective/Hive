// Test file for verifying file clicking functionality
// This file should open in the editor when clicked in:
// 1. The File Explorer panel
// 2. The Git Source Control panel

console.log('If you can see this in the editor, file clicking is working!');

// Test data
const testData = {
    message: 'File clicking is working correctly',
    timestamp: new Date().toISOString(),
    features: [
        'Explorer file clicking',
        'Git panel file clicking', 
        'Editor tabs opening'
    ]
};

console.log('Test data:', testData);