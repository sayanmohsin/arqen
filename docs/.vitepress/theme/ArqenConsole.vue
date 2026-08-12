<script setup lang="ts">
const layers = [
  { index: "01", label: "Discover", detail: "manifest · tools · scopes" },
  { index: "02", label: "Operate", detail: "routes · jobs · modules" },
  { index: "03", label: "Persist", detail: "objects · events · queues" },
];
</script>

<template>
  <section class="arqen-console" aria-label="Arqen application boundary">
    <div class="arqen-console__topline">
      <span class="eyebrow">ARQEN / APPLICATION BOUNDARY</span>
      <span class="pulse"><i /> LIVE CONTRACT</span>
    </div>
    <div class="arqen-console__body">
      <div class="arqen-console__copy">
        <p class="arqen-console__kicker">one boundary, many operators</p>
        <h2>Make your backend legible.</h2>
        <p>
          Arqen gives people, programs, and agents the same dependable surface: discoverable
          capabilities, explicit policies, durable work, and visible health.
        </p>
        <div class="arqen-console__signal">
          <span>request</span><b>→</b><span>policy</span><b>→</b><span>work</span><b>→</b
          ><span>record</span>
        </div>
      </div>
      <div class="arqen-console__stack">
        <div v-for="layer in layers" :key="layer.index" class="arqen-console__layer">
          <span class="arqen-console__index">{{ layer.index }}</span>
          <strong>{{ layer.label }}</strong>
          <small>{{ layer.detail }}</small>
          <span class="arqen-console__arrow">↗</span>
        </div>
        <div class="arqen-console__base">
          <span /> thingd adapter boundary <em>memory · native · HTTP</em>
        </div>
      </div>
    </div>
  </section>
</template>
