<template>
  <div class="settings-loading">
    <div class="settings-loading__line g-skeleton" />
    <div
      v-for="index in 8"
      :key="index"
      class="settings-loading__row"
    >
      <span class="g-skeleton" />
      <strong class="g-skeleton" />
    </div>
  </div>
</template>
