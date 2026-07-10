<template>
  <div class="cert-loading">
    <div class="cert-loading__stats">
      <article v-for="i in 4" :key="`stat-${i}`" class="cert-loading__stat-card">
        <div class="cert-loading__shimmer" style="width: 60%; height: 12px;" />
        <div class="cert-loading__shimmer cert-loading__shimmer--lg" style="width: 40%; height: 28px; margin-top: 8px;" />
        <div class="cert-loading__shimmer" style="width: 50%; height: 10px; margin-top: 8px;" />
      </article>
    </div>
    <div class="cert-loading__workspace">
      <aside class="cert-loading__list">
        <div v-for="i in 5" :key="`row-${i}`" class="cert-loading__row">
          <div class="cert-loading__shimmer cert-loading__shimmer--circle" />
          <div style="flex: 1; display: grid; gap: 6px;">
            <div class="cert-loading__shimmer" style="width: 70%; height: 12px;" />
            <div class="cert-loading__shimmer" style="width: 50%; height: 10px;" />
          </div>
        </div>
      </aside>
      <main class="cert-loading__detail">
        <div class="cert-loading__shimmer" style="width: 40%; height: 20px;" />
        <div class="cert-loading__metrics">
          <article v-for="i in 4" :key="`m-${i}`" class="cert-loading__shimmer" style="min-height: 70px;" />
        </div>
        <div class="cert-loading__shimmer" style="width: 100%; height: 120px;" />
      </main>
    </div>
  </div>
</template>

<script setup lang="ts"></script>

<style scoped>
.cert-loading {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.cert-loading__stats {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-3);
}

.cert-loading__stat-card {
  min-height: 100px;
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.cert-loading__workspace {
  display: grid;
  grid-template-columns: minmax(300px, 380px) minmax(0, 1fr);
  gap: var(--space-4);
  min-height: 400px;
}

.cert-loading__list,
.cert-loading__detail {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  padding: var(--space-3);
}

.cert-loading__row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  margin-bottom: var(--space-2);
}

.cert-loading__metrics {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-3);
  margin: var(--space-4) 0;
}

.cert-loading__shimmer {
  border-radius: var(--radius-sm);
  background: linear-gradient(
    90deg,
    var(--bg-input),
    var(--bg-surface-hover),
    var(--bg-input)
  );
  background-size: 200% 100%;
  animation: cert-shimmer 1.3s ease-in-out infinite;
}

.cert-loading__shimmer--lg {
  border-radius: var(--radius-md);
}

.cert-loading__shimmer--circle {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-full);
  flex-shrink: 0;
}

@keyframes cert-shimmer {
  from { background-position: 200% 0; }
  to { background-position: -200% 0; }
}

@media (max-width: 1120px) {
  .cert-loading__stats {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .cert-loading__workspace {
    grid-template-columns: 1fr;
  }
  .cert-loading__metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
