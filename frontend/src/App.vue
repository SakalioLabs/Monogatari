<template>
  <div id="app">
    <nav v-if="showNav" class="sidebar">
      <div class="logo">
        <h2>🎮 LLM Galgame</h2>
      </div>
      <div class="nav-links">
        <router-link to="/" class="nav-item">
          <span class="icon">🏠</span>
          <span>Home</span>
        </router-link>
        <router-link to="/game" class="nav-item">
          <span class="icon">🎭</span>
          <span>Game</span>
        </router-link>
        <router-link to="/editor" class="nav-item">
          <span class="icon">🔧</span>
          <span>Workflow</span>
        </router-link>
        <router-link to="/settings" class="nav-item">
          <span class="icon">⚙️</span>
          <span>Settings</span>
        </router-link>
      </div>
    </nav>
    <main class="content">
      <router-view />
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const showNav = computed(() => route.name !== 'game')
</script>

<style scoped>
#app {
  display: flex;
  min-height: 100vh;
}

.sidebar {
  width: 200px;
  background: var(--bg-card);
  border-right: 1px solid var(--border);
  padding: 20px 0;
  display: flex;
  flex-direction: column;
}

.logo {
  padding: 0 20px 20px;
  border-bottom: 1px solid var(--border);
  margin-bottom: 20px;
}

.logo h2 {
  font-size: 18px;
  color: var(--primary);
}

.nav-links {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 0 10px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border-radius: var(--radius);
  color: var(--text-muted);
  text-decoration: none;
  transition: all 0.2s;
}

.nav-item:hover {
  background: var(--bg-input);
  color: var(--text);
}

.nav-item.router-link-active {
  background: var(--primary);
  color: white;
}

.icon {
  font-size: 18px;
}

.content {
  flex: 1;
  overflow: auto;
}
</style>
