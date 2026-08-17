/**
 * CodeMirror language resolution from Markdown fence info strings.
 * Maps language identifiers (as used in ```lang fences) to CodeMirror extensions.
 * Used by the editor's markdown() codeLanguages option.
 */
import { LanguageDescription, LanguageSupport, StreamLanguage } from "@codemirror/language";
import { python } from "@codemirror/lang-python";
import { rust } from "@codemirror/lang-rust";
import { cpp } from "@codemirror/lang-cpp";
import { java } from "@codemirror/lang-java";
import { sql } from "@codemirror/lang-sql";
import { xml } from "@codemirror/lang-xml";
import { yaml } from "@codemirror/lang-yaml";
import { php } from "@codemirror/lang-php";

// Lazy-loaded legacy modes — each returns LanguageSupport as required by LanguageDescription.load
async function legacyRuby() { const { ruby } = await import("@codemirror/legacy-modes/mode/ruby"); return new LanguageSupport(StreamLanguage.define(ruby)); }
async function legacyGo() { const { go } = await import("@codemirror/legacy-modes/mode/go"); return new LanguageSupport(StreamLanguage.define(go)); }
async function legacySwift() { const { swift } = await import("@codemirror/legacy-modes/mode/swift"); return new LanguageSupport(StreamLanguage.define(swift)); }
async function legacyKotlin() { const { kotlin } = await import("@codemirror/legacy-modes/mode/clike"); return new LanguageSupport(StreamLanguage.define(kotlin)); }
async function legacyScala() { const { scala } = await import("@codemirror/legacy-modes/mode/clike"); return new LanguageSupport(StreamLanguage.define(scala)); }
async function legacyShell() { const { shell } = await import("@codemirror/legacy-modes/mode/shell"); return new LanguageSupport(StreamLanguage.define(shell)); }
async function legacyPowerShell() { const { powerShell } = await import("@codemirror/legacy-modes/mode/powershell"); return new LanguageSupport(StreamLanguage.define(powerShell)); }
async function legacyDockerfile() { const { dockerFile } = await import("@codemirror/legacy-modes/mode/dockerfile"); return new LanguageSupport(StreamLanguage.define(dockerFile)); }
async function legacyToml() { const { toml } = await import("@codemirror/legacy-modes/mode/toml"); return new LanguageSupport(StreamLanguage.define(toml)); }
async function legacyProperties() { const { properties } = await import("@codemirror/legacy-modes/mode/properties"); return new LanguageSupport(StreamLanguage.define(properties)); }
async function legacyObjC() { const { objectiveC } = await import("@codemirror/legacy-modes/mode/clike"); return new LanguageSupport(StreamLanguage.define(objectiveC)); }
async function legacyHaskell() { const { haskell } = await import("@codemirror/legacy-modes/mode/haskell"); return new LanguageSupport(StreamLanguage.define(haskell)); }
async function legacyErlang() { const { erlang } = await import("@codemirror/legacy-modes/mode/erlang"); return new LanguageSupport(StreamLanguage.define(erlang)); }
async function legacyClojure() { const { clojure } = await import("@codemirror/legacy-modes/mode/clojure"); return new LanguageSupport(StreamLanguage.define(clojure)); }
async function legacyLua() { const { lua } = await import("@codemirror/legacy-modes/mode/lua"); return new LanguageSupport(StreamLanguage.define(lua)); }
async function legacyR() { const { r } = await import("@codemirror/legacy-modes/mode/r"); return new LanguageSupport(StreamLanguage.define(r)); }
async function legacyPerl() { const { perl } = await import("@codemirror/legacy-modes/mode/perl"); return new LanguageSupport(StreamLanguage.define(perl)); }
async function legacyNginx() { const { nginx } = await import("@codemirror/legacy-modes/mode/nginx"); return new LanguageSupport(StreamLanguage.define(nginx)); }

/**
 * Build the LanguageDescription list for CodeMirror markdown codeLanguages.
 * Each entry maps one or more fence info aliases to a language loader.
 */
export function buildCodeLanguages(): LanguageDescription[] {
  return [
    LanguageDescription.of({ name: "Python", alias: ["python", "py"], load: () => Promise.resolve(python()) }),
    LanguageDescription.of({ name: "Rust", alias: ["rust", "rs"], load: () => Promise.resolve(rust()) }),
    LanguageDescription.of({ name: "C++", alias: ["cpp", "c++", "cxx", "cc"], load: () => Promise.resolve(cpp()) }),
    LanguageDescription.of({ name: "C", alias: ["c"], load: () => Promise.resolve(cpp()) }),
    LanguageDescription.of({ name: "Java", alias: ["java"], load: () => Promise.resolve(java()) }),
    LanguageDescription.of({ name: "SQL", alias: ["sql"], load: () => Promise.resolve(sql()) }),
    LanguageDescription.of({ name: "XML", alias: ["xml", "html", "htm", "svg"], load: () => Promise.resolve(xml()) }),
    LanguageDescription.of({ name: "YAML", alias: ["yaml", "yml"], load: () => Promise.resolve(yaml()) }),
    LanguageDescription.of({ name: "PHP", alias: ["php"], load: () => Promise.resolve(php()) }),
    LanguageDescription.of({ name: "Ruby", alias: ["ruby", "rb"], load: legacyRuby }),
    LanguageDescription.of({ name: "Go", alias: ["go", "golang"], load: legacyGo }),
    LanguageDescription.of({ name: "Swift", alias: ["swift"], load: legacySwift }),
    LanguageDescription.of({ name: "Kotlin", alias: ["kotlin", "kt"], load: legacyKotlin }),
    LanguageDescription.of({ name: "Scala", alias: ["scala"], load: legacyScala }),
    LanguageDescription.of({ name: "Shell", alias: ["sh", "bash", "shell", "zsh", "fish"], load: legacyShell }),
    LanguageDescription.of({ name: "PowerShell", alias: ["powershell", "ps1", "pwsh"], load: legacyPowerShell }),
    LanguageDescription.of({ name: "Dockerfile", alias: ["dockerfile", "docker"], load: legacyDockerfile }),
    LanguageDescription.of({ name: "TOML", alias: ["toml"], load: legacyToml }),
    LanguageDescription.of({ name: "INI", alias: ["ini", "properties", "env"], load: legacyProperties }),
    LanguageDescription.of({ name: "Objective-C", alias: ["objc", "objectivec", "objective-c"], load: legacyObjC }),
    LanguageDescription.of({ name: "Haskell", alias: ["haskell", "hs"], load: legacyHaskell }),
    LanguageDescription.of({ name: "Erlang", alias: ["erlang", "erl"], load: legacyErlang }),
    LanguageDescription.of({ name: "Clojure", alias: ["clojure", "clj", "cljs"], load: legacyClojure }),
    LanguageDescription.of({ name: "Lua", alias: ["lua"], load: legacyLua }),
    LanguageDescription.of({ name: "R", alias: ["r"], load: legacyR }),
    LanguageDescription.of({ name: "Perl", alias: ["perl", "pl"], load: legacyPerl }),
    LanguageDescription.of({ name: "Nginx", alias: ["nginx"], load: legacyNginx }),
  ];
}

/**
 * Resolve a fence info string to a LanguageDescription, if known.
 * Case-insensitive. Returns undefined if not found.
 */
export function resolveLanguage(infoString: string): LanguageDescription | undefined {
  const normalized = infoString.toLowerCase().trim();
  return buildCodeLanguages().find(
    (desc) => desc.name.toLowerCase() === normalized || (desc.alias ?? []).includes(normalized),
  );
}
