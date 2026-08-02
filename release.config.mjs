export const releaseRules = [
  { breaking: true, release: "major" },
  { type: "feat", release: "minor" },
  { type: "fix", release: "patch" },
  { type: "perf", release: "patch" },
];

export const releaseTagFormat = "arqen-v${version}";
