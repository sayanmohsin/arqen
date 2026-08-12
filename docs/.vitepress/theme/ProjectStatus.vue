<script setup lang="ts">
declare const __ARQEN_DOCS_META__: {
  arqenVersion: string;
  thingdVersion: string;
  releaseDate: string;
  docsRevision: string;
};

const meta = __ARQEN_DOCS_META__;
</script>

<template>
  <div class="project-status" aria-label="Current project status">
    <div>
      <span>Current release</span><strong>Arqen {{ meta.arqenVersion }}</strong>
    </div>
    <div>
      <span>Thingd boundary</span><strong>Thingd {{ meta.thingdVersion }}</strong>
    </div>
    <div>
      <span>Released</span><strong>{{ meta.releaseDate }}</strong>
    </div>
    <div>
      <span>Docs revision</span><strong>{{ meta.docsRevision }}</strong>
    </div>
  </div>
</template>

<style scoped>
.project-status {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 1px;
  margin: 2rem 0;
  overflow: hidden;
  border: 1px solid var(--vp-c-divider);
  border-radius: 14px;
  background: var(--vp-c-divider);
}
.project-status > div {
  display: grid;
  gap: 0.35rem;
  padding: 1rem 1.1rem;
  background: var(--vp-c-bg-soft);
}
.project-status span {
  color: var(--vp-c-text-2);
  font-size: 0.75rem;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}
@media (max-width: 640px) {
  .project-status {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
