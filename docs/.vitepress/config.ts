import { defineConfig } from "vitepress";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const repoRoot = resolve(__dirname, "../..");
const crateManifest = readFileSync(resolve(repoRoot, "crates/arqen/Cargo.toml"), "utf8");
const changelog = readFileSync(resolve(repoRoot, "crates/arqen/CHANGELOG.md"), "utf8");
const arqenVersion = crateManifest.match(/^version\s*=\s*"([^"]+)"/m)?.[1] ?? "unknown";
const nativeThingdVersion =
  crateManifest.match(/thingd\s*=.*?version\s*=\s*"([^\"]+)"/)?.[1] ?? "unknown";
const httpApiVersion = "v1";
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
        nativeThingdVersion,
        httpApiVersion,
        releaseDate,
        docsRevision,
      }),
    },
  },
  title: "Arqen — Build and operate a backend",
  description:
    "A Rust backend toolkit for HTTP services, modules, jobs, health checks, and Thingd storage.",
  base: "/arqen/",
  lang: "en-US",
  cleanUrls: true,
  head: [
    ["link", { rel: "icon", type: "image/svg+xml", href: "/arqen/favicon.svg" }],
    ["meta", { name: "theme-color", content: "#080a0d" }],
    [
      "meta",
      {
        property: "og:title",
        content: "Arqen — Build and operate a backend",
      },
    ],
    [
      "meta",
      {
        property: "og:description",
        content: "HTTP services, modules, durable jobs, health checks, and Thingd storage.",
      },
    ],
  ],
  themeConfig: {
    logo: "/logo.svg",
    siteTitle: "",
    search: { provider: "local" },
    nav: [
      { text: "Start here", link: "/build-a-backend" },
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
          { text: "Build a backend", link: "/build-a-backend" },
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
          { text: "Getting started", link: "/getting-started" },
          { text: "CLI project generator", link: "/cli-generator" },
          { text: "Thingd schema", link: "/schema" },
          { text: "Thingd bootstrap", link: "/bootstrap" },
          { text: "Commands", link: "/commands" },
          { text: "Configuration", link: "/configuration" },
          { text: "HTTP caching", link: "/http-caching" },
          { text: "Streaming", link: "/streaming" },
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
      message: "Rust-first backend toolkit · explicit integrations for HTTP, jobs, and Thingd",
      copyright: "© 2026 Arqen contributors · MIT License",
    },
  },
});
