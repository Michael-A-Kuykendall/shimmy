#!/usr/bin/env node
/**
 * Autonomous screenshot analysis tool
 * Reads PNG screenshot and returns base64 for analysis
 * Usage: node analyze-screenshot.js <image-path> [mode]
 */

const fs = require('fs');
const path = require('path');
const Tesseract = require('tesseract.js');

const imagePath = process.argv[2];
const mode = process.argv[3] || 'base64';
const outputFile = process.argv[4];

if (!imagePath) {
    console.error('❌ Usage: node analyze-screenshot.js <image-path> [mode] [output-file]');
    process.exit(1);
}

const fullPath = path.resolve(imagePath);

if (!fs.existsSync(fullPath)) {
    console.error(`❌ File not found: ${fullPath}`);
    process.exit(1);
}

(async () => {
    try {
        const imageBytes = fs.readFileSync(fullPath);
        const b64Data = imageBytes.toString('base64');
        
        const result = {
            success: true,
            file: fullPath,
            mode: mode,
            size_bytes: imageBytes.length,
            base64_length: b64Data.length,
        };
        
        if (mode === 'base64' || mode === 'full') {
            result.image_url = `data:image/png;base64,${b64Data}`;
            result.json_envelope = {
                type: "input_image",
                image_url: `data:image/png;base64,${b64Data}`
            };
        } else if (mode === 'meta') {
            result.base64_length = b64Data.length;
        } else if (mode === 'ocr' || mode === 'text') {
            // console.error('🔍 Performing OCR...'); // Keep stderr clean for JSON output
            const { data: { text } } = await Tesseract.recognize(fullPath, 'eng', {
                logger: m => {} // Silence logger
            });
            result.text = text.trim();
        }
        
        if (outputFile) {
            fs.writeFileSync(outputFile, JSON.stringify(result, null, 2));
            console.log(`✅ Output written to ${outputFile}`);
        } else {
            console.log(JSON.stringify(result, null, 2));
        }
        
    } catch (err) {
        console.error(`❌ Error: ${err.message}`);
        process.exit(1);
    }
})();
