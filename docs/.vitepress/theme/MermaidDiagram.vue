<script setup lang="ts">
import mermaid from "mermaid";
import { onBeforeUnmount, onMounted, ref } from "vue";

const props = defineProps<{ type: "architecture" | "use-cases" | "schema" }>();
const diagram = ref<HTMLElement | null>(null);
let observer: MutationObserver | undefined;

const definitions = {
  architecture: `flowchart TB
    client["People, programs, and agents"] --> api["Arqen HTTP boundary"]
    api --> core
    subgraph core["Arqen application boundary"]
      direction LR
      policy["Auth + policy"]
      tools["Typed tools + manifests"]
      jobs["Durable jobs + workers"]
      health["Health + observability"]
    end
    core --> domain["Application domain services"]
    domain --> adapter["ThingdBackend adapter"]
    adapter --> stores
    subgraph stores["Storage paths"]
      direction LR
      memory["Memory backend"]
      native["Native Thingd"]
      http["HTTP Thingd"]
    end
    http -. future contract .-> cloud["Hosted Thingd / Cloud"]
    classDef edge fill:#171127,stroke:#a78bfa,color:#f5f3ff
    classDef core fill:#102530,stroke:#22d3ee,color:#ecfeff
    classDef store fill:#21152a,stroke:#f0abfc,color:#fff1ff
    class client,api,policy,tools,jobs,health,domain edge
    class adapter core
    class memory,native,http,cloud store`,
  "use-cases": `flowchart TB
    request["Request from a person, program, or agent"] --> discover{"What does the caller need?"}
    discover -->|"A capability"| tool["Discover typed tool + permissions"]
    discover -->|"Work that takes time"| job["Enqueue durable job"]
    discover -->|"A service operation"| route["Call HTTP route"]
    tool --> policy["Validate identity, scope, and input"]
    job --> policy
    route --> policy
    policy --> execute["Run application service"]
    execute --> record["Record objects, events, links, or queue state"]
    record --> observe["Expose health, logs, metrics, and status"]
    observe --> operator["Operator or client can inspect the result"]
    classDef start fill:#21152a,stroke:#f0abfc,color:#fff1ff
    classDef action fill:#171127,stroke:#a78bfa,color:#f5f3ff
    classDef result fill:#102530,stroke:#22d3ee,color:#ecfeff
    class request,discover start
    class tool,job,route,policy,execute action
    class record,observe,operator result`,
  schema: `flowchart LR
    file["schema.thingd in your repository"] --> local["Arqen loads and hashes it"]
    local --> validate["Thingd /v1/schema/validate"]
    validate --> inspect["Inspect current schema + migration history"]
    inspect --> operator["Operator applies supported migration"]
    operator --> ready["Start backend with schema validation"]
    classDef source fill:#21152a,stroke:#f0abfc,color:#fff1ff
    classDef check fill:#171127,stroke:#a78bfa,color:#f5f3ff
    classDef result fill:#102530,stroke:#22d3ee,color:#ecfeff
    class file source
    class local,validate,inspect,operator check
    class ready result`,
} as const;

async function render() {
  if (!diagram.value) return;
  mermaid.initialize({
    startOnLoad: false,
    securityLevel: "strict",
    theme: "base",
    themeVariables: {
      darkMode: document.documentElement.classList.contains("dark"),
      background: "transparent",
      primaryColor: "#171127",
      primaryTextColor: "#f5f3ff",
      primaryBorderColor: "#a78bfa",
      lineColor: "#8b5cf6",
      secondaryColor: "#102530",
      tertiaryColor: "#21152a",
      fontSize: "18px",
      fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
    },
  });
  const id = `arqen-mermaid-${props.type}`;
  const result = await mermaid.render(id, definitions[props.type]);
  diagram.value.innerHTML = result.svg;
}

onMounted(() => {
  render();
  observer = new MutationObserver(() => render());
  observer.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
});

onBeforeUnmount(() => observer?.disconnect());
</script>

<template>
  <div
    ref="diagram"
    class="mermaid-diagram"
    :class="`mermaid-diagram--${type}`"
    role="img"
    :aria-label="`${type} diagram`"
  />
</template>
