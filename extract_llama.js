const fs = require('fs');
const content = fs.readFileSync('patch_line.txt', 'utf8');

// The pattern in the file is literal backslash-n, not escaped
const llamaStart = content.indexOf('llama.rs\\n+#!');
console.log('llamaStart:', llamaStart);
if (llamaStart < 0) { process.exit(1); }

const afterLlama = content.substring(llamaStart + 12);
const templatesIdx = afterLlama.indexOf('\\n+\\n*** Add File:');
console.log('templatesIdx:', templatesIdx);

let code = afterLlama.substring(0, templatesIdx);
code = code.replace(/\\n\+/g, '\n');
code = code.replace(/\\n/g, '\n');
code = code.replace(/\\"/g, '"');
code = '#!' + code;  // add back the #! we skipped

fs.writeFileSync('llama_extracted.rs', code);
console.log('Wrote', code.split('\n').length, 'lines');
