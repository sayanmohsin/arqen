import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { releaseRules } from "../release.config.mjs";

const root = process.cwd();
const args = new Set(process.argv.slice(2));

function argumentValue(name) {
  const index = process.argv.indexOf(name);
  return index === -1 ? undefined : process.argv[index + 1];
}

function git(...gitArgs) {
  return execFileSync("git", gitArgs, { cwd: root, encoding: "utf8" }).trim();
}

function readVersion() {
  const cargo = fs.readFileSync(path.join(root, "Cargo.toml"), "utf8");
  return cargo.match(/^version = "([0-9]+\.[0-9]+\.[0-9]+)"/m)?.[1];
}

function parseVersion(version) {
  const match = version.match(/^(\d+)\.(\d+)\.(\d+)$/);
  if (!match) throw new Error(`Invalid version: ${version}`);
  return match.slice(1).map(Number);
}

function bumpVersion(version, releaseType) {
  const [major, minor, patch] = parseVersion(version);
  if (releaseType === "major") return `${major + 1}.0.0`;
  if (releaseType === "minor") return `${major}.${minor + 1}.0`;
  return `${major}.${minor}.${patch + 1}`;
}

function latestReleaseTag() {
  try {
    return execFileSync(
      "git",
      ["describe", "--tags", "--match", "arqen-v[0-9]*", "--abbrev=0"],
      { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] },
    ).trim();
  } catch {
    return undefined;
  }
}

function commitsSince(tag) {
  const range = tag ? `${tag}..HEAD` : "HEAD";
  const raw = git("log", range, "--format=%H%x1f%s%x1f%b%x1e");
  return raw
    .split("\x1e")
    .filter(Boolean)
    .map((entry) => {
      const [hash, subject, body = ""] = entry.split("\x1f");
      return { hash: hash.trim(), message: `${subject.trim()}\n${body.trim()}`.trim() };
    });
}

function releaseTypeFor(commits) {
  let releaseType;
  for (const { message } of commits) {
    const subject = message.split("\n", 1)[0];
    const match = subject.match(/^(\w+)(?:\(([^)]+)\))?(!)?:\s+.+/);
    const breaking = Boolean(match?.[3]) || /BREAKING CHANGE:/m.test(message);
    if (breaking) return "major";
    const type = match?.[1];
    const rule = releaseRules.find((candidate) => candidate.type === type);
    if (rule && (!releaseType || rule.release === "minor")) releaseType = rule.release;
  }
  return releaseType;
}

function releaseNotes(commits, lastTag, version) {
  const groups = new Map([
    ["breaking", []],
    ["feat", []],
    ["fix", []],
    ["perf", []],
  ]);
  for (const { hash, message } of commits) {
    const subject = message.split("\n", 1)[0];
    const match = subject.match(/^(\w+)(?:\(([^)]+)\))?(!)?:\s+(.+)/);
    if (!match) continue;
    const [, type, scope, bang, description] = match;
    const breaking = bang || /BREAKING CHANGE:/m.test(message);
    const group = breaking ? "breaking" : type;
    if (groups.has(group)) groups.get(group).push(`${scope ? `${scope}: ` : ""}${description} (${hash.slice(0, 7)})`);
  }
  const sections = [];
  for (const [group, entries] of groups) {
    if (!entries.length) continue;
    const title = group === "breaking" ? "BREAKING CHANGES" : group === "feat" ? "Features" : group === "fix" ? "Bug Fixes" : "Performance";
    sections.push(`### ${title}\n\n${entries.map((entry) => `- ${entry}`).join("\n")}`);
  }
  const repository = process.env.GITHUB_REPOSITORY || "sayanmohsin/arqen";
  const compare = lastTag ? `https://github.com/${repository}/compare/${lastTag}...arqen-v${version}` : `https://github.com/${repository}/releases/tag/arqen-v${version}`;
  return `## [${version}](${compare}) (${new Date().toISOString().slice(0, 10)})\n\n${sections.join("\n\n") || "Release preparation and package consolidation."}`;
}

function createPlan() {
  const currentVersion = readVersion();
  if (!currentVersion) throw new Error("Could not read the workspace version from Cargo.toml");
  const lastTag = latestReleaseTag();
  const commits = commitsSince(lastTag);
  const releaseType = releaseTypeFor(commits);
  if (!releaseType) return { needed: false, reason: "no releasable conventional commits" };
  const version = bumpVersion(currentVersion, releaseType);
  return { needed: true, version, releaseType, lastTag: lastTag || null, notes: releaseNotes(commits, lastTag, version) };
}

function replaceVersion(version) {
  const file = path.join(root, "Cargo.toml");
  const source = fs.readFileSync(file, "utf8");
  const updated = source.replace(/^version = "[^"]+"/m, `version = "${version}"`);
  if (source === updated) throw new Error("Workspace version was not found in Cargo.toml");
  fs.writeFileSync(file, updated);
}

function applyPlan(plan) {
  replaceVersion(plan.version);
  execFileSync("cargo", ["check", "--workspace"], { cwd: root, stdio: "inherit" });
  const changelog = path.join(root, "CHANGELOG.md");
  const existing = fs.readFileSync(changelog, "utf8").trimStart();
  fs.writeFileSync(changelog, `${plan.notes.trim()}\n\n${existing}`);
}

const planFile = argumentValue("--plan-file");
const outputFile = argumentValue("--json");
const plan = planFile ? JSON.parse(fs.readFileSync(path.resolve(planFile), "utf8")) : createPlan();
if (args.has("--apply") && plan.needed) applyPlan(plan);
const serialized = `${JSON.stringify(plan, null, 2)}\n`;
if (outputFile) fs.writeFileSync(path.resolve(outputFile), serialized);
process.stdout.write(serialized);
