import { defineConfig } from "vitepress";

export default defineConfig({
  title: "Arqen — Backend infrastructure for agent-ready applications",
  description:
    "A developer-focused backend toolkit for agent-ready applications, with typed tools, durable jobs, discoverable APIs, and thingd integration.",
  base: "/arqen/",
  lang: "en-US",
  cleanUrls: true,
  head: [
    ["link", { rel: "icon", type: "image/svg+xml", href: "/arqen/logo.svg" }],
    ["meta", { name: "theme-color", content: "#080a0d" }],
    ["meta", { property: "og:title", content: "Arqen — Backend infrastructure for agent-ready applications" }],
    [
      "meta",
      {
        property: "og:description",
        content:
          "Typed tools, durable jobs, discoverable APIs, and thingd integration.",
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
          { text: "Feature status", link: "/feature-status" },
          { text: "Storage modes", link: "/in-memory-mode" },
          { text: "thingd integration", link: "/thingd-integration" },
        ],
      },
      {
        text: "Guides",
        items: [
          { text: "Getting started", link: "/getting-started" },
          { text: "Commands", link: "/commands" },
          { text: "Configuration", link: "/configuration" },
          { text: "Typed tools", link: "/typed-tools" },
          { text: "Durable jobs", link: "/durable-jobs" },
        ],
      },
      {
        text: "Operations",
        items: [
          { text: "Deployment", link: "/deployment" },
          { text: "Docker", link: "/docker" },
          { text: "Logging", link: "/logging" },
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
          { text: "Repository structure", link: "/repository-structure" },
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
