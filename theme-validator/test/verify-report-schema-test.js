const Ajv = require('ajv');
const addFormats = require('ajv-formats');
const path = require('path');
const fs = require('fs');

const ShimmyValidator = require(path.resolve(__dirname, '..', 'validator.js'));

async function run() {
  // instantiate and create a fake report
  const v = new ShimmyValidator();

  // Populate results as all-success to generate a full report
  v.results.discovery = { status: 'success', port: 51061, error: null };
  v.results.schema_load = { status: 'success', error: null };
  v.results.websocket_connection = { status: 'success', error: null };
  v.results.get_models_test = { status: 'success', error: null, response: { type: 'models_response', success: true, models: [{ name: 'test-model', display_name: 'Test' }], timestamp: new Date().toISOString() } };
  v.results.select_model_test = { status: 'success', error: null, response: { type: 'model_selected', success: true, model_name: 'test-model', timestamp: new Date().toISOString() } };
  v.results.chat_test = { status: 'success', error: null, tokens_received: 10, generation_complete: true };

  const { report, passed } = v.generateReport();

  // Load schema
  const schemaPath = path.resolve(__dirname, '..', '..', 'scripts', 'verify-report.schema.json');
  const schemaContent = fs.readFileSync(schemaPath, 'utf8');
  const schema = JSON.parse(schemaContent);

  const ajv = new Ajv({ allErrors: true, strict: false });
  addFormats(ajv);
  const validate = ajv.compile(schema);

  const valid = validate(report);

  if (!valid) {
    console.error('Schema validation failed:', validate.errors);
    process.exit(2);
  }

  console.log('✔ verify-report.json produced by generateReport() validates against scripts/verify-report.schema.json');
  process.exit(0);
}

run().catch(err => {
  console.error('Test runner crashed:', err);
  process.exit(3);
});
