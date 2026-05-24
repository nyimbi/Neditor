import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const args = parseArgs(process.argv.slice(2));
const requiredMarker = "NEDITOR_PROVIDER_EVIDENCE_OK";
const providerProfile = String(args.profile || process.env.NEDITOR_AI_PROVIDER_PROFILE || "openai-compatible");
const endpoint = String(args.endpoint || process.env.NEDITOR_AI_PROVIDER_ENDPOINT || "");
const model = String(args.model || process.env.NEDITOR_AI_PROVIDER_MODEL || "");
const apiKeyEnv = String(args["api-key-env"] || process.env.NEDITOR_AI_PROVIDER_API_KEY_ENV || "");
const outputPath = resolve(args.output || process.env.NEDITOR_AI_PROVIDER_EVIDENCE_OUTPUT || join(root, ".tmp", "ai-provider-evidence", "external", "provider-evidence.json"));
const prompt = promptText();
const sourceCommit = gitCommit();
const sourceTreeClean = gitTreeClean();

const issues = [];
if (!["openai-compatible", "anthropic-compatible", "gemini-compatible", "local-http"].includes(providerProfile)) {
  issues.push(`Unsupported provider profile: ${providerProfile}`);
}
if (!endpoint) issues.push("Missing endpoint. Pass --endpoint or set NEDITOR_AI_PROVIDER_ENDPOINT.");
if (!model) issues.push("Missing model. Pass --model or set NEDITOR_AI_PROVIDER_MODEL.");
if (providerProfile !== "local-http" && !apiKeyEnv) issues.push("Missing API key environment variable name. Pass --api-key-env or set NEDITOR_AI_PROVIDER_API_KEY_ENV.");
if (apiKeyEnv && !process.env[apiKeyEnv]) issues.push(`API key environment variable is not set: ${apiKeyEnv}`);
if (!sourceTreeClean) issues.push("AI provider evidence must be collected from a clean Git tree.");

if (issues.length > 0) {
  console.error("AI provider evidence collection failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  process.exit(1);
}

const startedAt = new Date().toISOString();
const request = buildRequest();
const response = await fetch(endpoint, {
  method: "POST",
  headers: request.headers,
  body: JSON.stringify(request.body),
});
const raw = await response.text();
const finishedAt = new Date().toISOString();
const extractedText = extractProviderText(raw, providerProfile);
const markers = [requiredMarker].filter((marker) => extractedText.includes(marker));
const endpointUrl = new URL(endpoint);
const evidence = {
  schema: "neditor.ai-provider-evidence.v1",
  generatedAt: finishedAt,
  status: response.ok && markers.includes(requiredMarker) ? "passed" : "failed",
  appVersion: packageJson.version,
  sourceCommit,
  sourceTreeClean,
  providerProfile,
  endpointHost: endpointUrl.host,
  endpointPath: endpointUrl.pathname,
  model,
  secretMaterialStored: false,
  request: {
    startedAt,
    apiKeyEnv: providerProfile === "local-http" ? null : apiKeyEnv,
    promptSha256: sha256(prompt),
    bodyShape: request.bodyShape,
  },
  response: {
    finishedAt,
    httpStatus: response.status,
    rawSha256: sha256(raw),
    extractedTextSha256: sha256(extractedText),
    markers,
    preview: extractedText.slice(0, 800),
  },
};

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, `${JSON.stringify(evidence, null, 2)}\n`);

if (evidence.status !== "passed") {
  console.error(`AI provider evidence was collected but did not pass validation markers: ${relative(outputPath)}`);
  process.exit(1);
}

console.log(`Collected AI provider evidence: ${relative(outputPath)}`);

function buildRequest() {
  const baseHeaders = {
    "content-type": "application/json",
  };
  if (providerProfile === "local-http") {
    return {
      headers: baseHeaders,
      bodyShape: "openai-compatible-chat-completions",
      body: openAiCompatibleBody(),
    };
  }
  if (providerProfile === "anthropic-compatible") {
    return {
      headers: {
        ...baseHeaders,
        "x-api-key": process.env[apiKeyEnv],
        "anthropic-version": "2023-06-01",
      },
      bodyShape: "anthropic-messages",
      body: {
        model,
        max_tokens: 600,
        messages: [{ role: "user", content: prompt }],
      },
    };
  }
  if (providerProfile === "gemini-compatible") {
    return {
      headers: {
        ...baseHeaders,
        "x-goog-api-key": process.env[apiKeyEnv],
      },
      bodyShape: "gemini-generate-content",
      body: {
        contents: [{ role: "user", parts: [{ text: prompt }] }],
        generationConfig: { temperature: 0 },
      },
    };
  }
  return {
    headers: {
      ...baseHeaders,
      authorization: `Bearer ${process.env[apiKeyEnv]}`,
    },
    bodyShape: "openai-compatible-chat-completions",
    body: openAiCompatibleBody(),
  };
}

function openAiCompatibleBody() {
  return {
    model,
    temperature: 0,
    messages: [
      {
        role: "system",
        content: "You are validating NEditor provider connectivity. Return concise Markdown only.",
      },
      {
        role: "user",
        content: prompt,
      },
    ],
  };
}

function promptText() {
  const promptFile = args["prompt-file"] || process.env.NEDITOR_AI_PROVIDER_PROMPT_FILE;
  if (promptFile) {
    const path = resolve(promptFile);
    if (!existsSync(path)) {
      console.error(`Prompt file does not exist: ${path}`);
      process.exit(1);
    }
    return readFileSync(path, "utf8");
  }
  return [
    "Return a short Markdown readiness note for NEditor.",
    `Include the exact marker ${requiredMarker}.`,
    "Do not include secrets, keys, personal data, or external links.",
  ].join("\n");
}

function extractProviderText(raw, profile) {
  try {
    const json = JSON.parse(raw);
    if (profile === "anthropic-compatible") {
      return (json.content || []).map((part) => part.text || "").join("\n").trim();
    }
    if (profile === "gemini-compatible") {
      return (json.candidates || [])
        .flatMap((candidate) => candidate.content?.parts || [])
        .map((part) => part.text || "")
        .join("\n")
        .trim();
    }
    return String(json.choices?.[0]?.message?.content || json.choices?.[0]?.text || json.content || json.output || raw).trim();
  } catch {
    return raw.trim();
  }
}

function sha256(value) {
  return createHash("sha256").update(String(value)).digest("hex");
}

function gitCommit() {
  const result = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8",
  });
  if (result.status !== 0) return "";
  return result.stdout.trim();
}

function gitTreeClean() {
  const result = spawnSync("git", ["status", "--porcelain"], {
    cwd: root,
    encoding: "utf8",
  });
  return result.status === 0 && result.stdout.trim() === "";
}

function parseArgs(values) {
  const parsed = {};
  for (let index = 0; index < values.length; index += 1) {
    const value = values[index];
    if (!value.startsWith("--")) continue;
    const key = value.slice(2);
    const next = values[index + 1];
    if (!next || next.startsWith("--")) {
      parsed[key] = true;
    } else {
      parsed[key] = next;
      index += 1;
    }
  }
  return parsed;
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
