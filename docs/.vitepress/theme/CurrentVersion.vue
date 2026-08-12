<script setup lang="ts">
withDefaults(
  defineProps<{
    kind: "arqen" | "thingd";
    label?: boolean;
  }>(),
  { label: true },
);

declare const __ARQEN_DOCS_META__: {
  arqenVersion: string;
  thingdVersion: string;
};

const meta = __ARQEN_DOCS_META__;
</script>

<template>
  <span class="current-version">
    {{ label ? (kind === "arqen" ? "Arqen " : "Thingd ") : ""
    }}{{ kind === "arqen" ? meta.arqenVersion : meta.thingdVersion }}
  </span>
</template>
