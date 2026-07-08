import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import './styles/main.css'
import { loadI18n } from './lib/i18n'
import { registerPwa } from './lib/pwa'

loadI18n()

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
registerPwa()
