<script setup lang="ts">
withDefaults(
  defineProps<{
    kind: "arqen" | "native-thingd" | "http-api";
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
    {{
      label
        ? kind === "arqen"
          ? "Arqen "
          : kind === "native-thingd"
            ? "Native Thingd "
            : "Thingd HTTP API "
        : ""
    }}{{
      kind === "arqen"
        ? meta.arqenVersion
        : kind === "native-thingd"
          ? meta.nativeThingdVersion
          : meta.httpApiVersion
    }}
  </span>
</template>
