const Ajv = require('ajv');
const addFormats = require('ajv-formats');
const path = require('path');
const fs = require('fs');

function loadFixture(name) {
  const p = path.resolve(__dirname, 'fixtures', name);
  const raw = fs.readFileSync(p, 'utf8');
  return JSON.parse(raw);
}

function getDiscoverySchema() {
  return {
    type: "object",
    required: ["discovery_port","last_updated","epoch","backends"],
    properties: {
      discovery_port: { type: "integer", minimum: 1, maximum: 65535 },
      last_updated: { type: "string", format: "date-time" },
      epoch: { type: "integer", minimum: 0 },
      backends: {
        type: "array",
        items: {
          type: "object",
          required: ["id","url","port","models","capabilities","health","started_at","pid"],
          properties: {
            id: { type: "string" },
            url: { type: "string", format: "uri" },
            port: { type: "integer", minimum: 1, maximum: 65535 },
            models: {
              type: "array",
              items: {
                type: "object",
                required: ["name","display_name","source","backend","health_status"],
                properties: {
                  name: { type: "string" },
                  display_name: { type: "string" },
                  size_bytes: { type: ["integer","null"], minimum: 0 },
                  parameter_count: { type: ["string","null"] },
                  quantization: { type: ["string","null"] },
                  context_length: { type: ["integer","null"], minimum: 1 },
                  model_type: { type: ["string","null"] },
                  source: { type: "string" },
                  backend: { type: "string" },
                  health_status: { type: "string", enum: ["ready","checking","failed","unknown"] },
                  health_error: { type: ["string","null"] }
                },
                additionalProperties: false
              }
            },
            capabilities: { type: "array", items: { type: "string" } },
            health: {
              type: "object",
              required: ["healthy","last_check"],
              properties: {
                healthy: { type: "boolean" },
                last_check: { type: "string", format: "date-time" }
              },
              additionalProperties: false
            },
            started_at: { type: "string", format: "date-time" },
            pid: { type: "integer", minimum: 1 }
          },
          additionalProperties: false
        }
      }
    },
    additionalProperties: false
  };
}

async function run() {
  const discovery = loadFixture('discovery-snapshot.json');

  const ajv = new Ajv({ allErrors: true, strict: false });
  addFormats(ajv);
  const validate = ajv.compile(getDiscoverySchema());

  const valid = validate(discovery);
  if (!valid) {
    console.error('Discovery fixture did not validate:', validate.errors);
    process.exit(2);
  }

  console.log('✔ discovery-snapshot.json validates against discovery schema');
  process.exit(0);
}

run().catch(err => { console.error('Test runner crashed:', err); process.exit(3); });
