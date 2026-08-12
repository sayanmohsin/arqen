import { defineConfig } from "vitepress";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const repoRoot = resolve(__dirname, "../..");
const crateManifest = readFileSync(resolve(repoRoot, "crates/arqen/Cargo.toml"), "utf8");
const workspaceManifest = readFileSync(resolve(repoRoot, "Cargo.toml"), "utf8");
const changelog = readFileSync(resolve(repoRoot, "crates/arqen/CHANGELOG.md"), "utf8");
const arqenVersion = crateManifest.match(/^version\s*=\s*"([^"]+)"/m)?.[1] ?? "unknown";
const thingdVersion =
  workspaceManifest.match(/thingd\s*=.*?version\s*=\s*"=([^\"]+)"/)?.[1] ?? "unknown";
const releaseDate = changelog.match(/## \[[^\]]+\][^\n]*\((\d{4}-\d{2}-\d{2})\)/)?.[1] ?? "unknown";
let docsRevision = "local";
try {
  docsRevision = execFileSync("git", ["rev-parse", "--short", "HEAD"], { cwd: repoRoot })
    .toString()
    .trim();
} catch {
  // Git is not required when the documentation is built from an archive.
}

export default defineConfig({
  vite: {
    define: {
      __ARQEN_DOCS_META__: JSON.stringify({
        arqenVersion,
        thingdVersion,
        releaseDate,
        docsRevision,
      }),
    },
  },
  title: "Arqen — Backend infrastructure for agent-ready applications",
  description:
    "A developer-focused backend toolkit for agent-ready applications, with typed tools, durable jobs, discoverable APIs, and thingd integration.",
  base: "/arqen/",
  lang: "en-US",
  cleanUrls: true,
  head: [
    ["link", { rel: "icon", type: "image/svg+xml", href: "/arqen/logo.svg" }],
    ["meta", { name: "theme-color", content: "#080a0d" }],
    [
      "meta",
      {
        property: "og:title",
        content: "Arqen — Backend infrastructure for agent-ready applications",
      },
    ],
    [
      "meta",
      {
        property: "og:description",
        content: "Typed tools, durable jobs, discoverable APIs, and thingd integration.",
      },
    ],
  ],
  themeConfig: {
    logo: "/logo.svg",
    siteTitle: "",
    search: { provider: "local" },
    nav: [
      { text: "Start", link: "/" },
      { text: "Concepts", link: "/about" },
      { text: "Guides", link: "/getting-started" },
      { text: "Operations", link: "/deployment" },
      { text: "Agent Integration", link: "/agent-guide" },
      { text: "Reference", link: "/architecture" },
      { text: "Project", link: "/contributing" },
      { text: "thingd.cloud", link: "https://thingd.cloud" },
    ],
    sidebar: [
      {
        text: "Start",
        items: [
          { text: "Overview", link: "/" },
          { text: "About Arqen", link: "/about" },
          { text: "Why Arqen?", link: "/why-arqen" },
          { text: "Use cases", link: "/use-cases" },
          { text: "FAQ", link: "/faq" },
        ],
      },
      {
        text: "Concepts",
        items: [
          { text: "Architecture", link: "/architecture" },
          { text: "Modules", link: "/modules" },
          { text: "Feature status", link: "/feature-status" },
          { text: "API stability", link: "/api-stability" },
          { text: "Storage modes", link: "/in-memory-mode" },
          { text: "thingd integration", link: "/thingd-integration" },
          {
            text: "Storage and migration",
            link: "/migration",
          },
          { text: "Application hardening", link: "/application-hardening" },
        ],
      },
      {
        text: "Guides",
        items: [
          { text: "Build a backend", link: "/build-a-backend" },
          { text: "Getting started", link: "/getting-started" },
          { text: "Thingd schema", link: "/schema" },
          { text: "Commands", link: "/commands" },
          { text: "Configuration", link: "/configuration" },
          { text: "Health", link: "/health" },
          { text: "Examples", link: "/examples" },
          { text: "Authentication", link: "/authentication" },
          { text: "Validation", link: "/validation" },
          { text: "Typed tools", link: "/typed-tools" },
          { text: "Durable jobs", link: "/durable-jobs" },
          { text: "Testing", link: "/testing" },
          { text: "Tooling", link: "/tooling" },
        ],
      },
      {
        text: "Operations",
        items: [
          { text: "Deployment", link: "/deployment" },
          { text: "Production runbook", link: "/production-runbook" },
          { text: "Docker", link: "/docker" },
          { text: "Logging", link: "/logging" },
          { text: "Observability", link: "/observability" },
          { text: "OpenAPI", link: "/openapi" },
          { text: "Performance", link: "/performance" },
          { text: "Security", link: "/security" },
          { text: "Release", link: "/release" },
        ],
      },
      {
        text: "Agent Integration",
        items: [
          { text: "Agent guide", link: "/agent-guide" },
          { text: "Agent discovery", link: "/agent-discovery" },
          { text: "Manifest contract", link: "/manifest" },
        ],
      },
      {
        text: "Reference",
        items: [
          { text: "Adapter contract", link: "/adapter-contract" },
          { text: "Standards", link: "/standards" },
          { text: "Repository structure", link: "/repository-structure" },
          { text: "Troubleshooting", link: "/troubleshooting" },
          { text: "Migration", link: "/migration" },
          { text: "Roadmap", link: "/roadmap" },
        ],
      },
      {
        text: "Project",
        items: [
          { text: "Contributing", link: "/contributing" },
          { text: "GitHub", link: "https://github.com/sayanmohsin/arqen" },
          { text: "thingd.cloud", link: "https://thingd.cloud" },
        ],
      },
    ],
    socialLinks: [{ icon: "github", link: "https://github.com/sayanmohsin/arqen" }],
    footer: {
      message: "Rust-first implementation · language-agnostic application positioning",
      copyright: "© 2026 Arqen contributors · MIT License",
    },
  },
});
