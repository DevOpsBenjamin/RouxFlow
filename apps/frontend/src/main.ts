import { createApp } from 'vue'
import { createPinia } from 'pinia'
import './style.css'
import App from './App.vue'
import router from './router'
import { useAuthStore } from './stores/auth'
import { ensureWasm } from './services/cube/bridge'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)

// Initialize auth state
const auth = useAuthStore(pinia)

async function start() {
    await auth.init()
    await ensureWasm(auth.user?.id)
    console.log('[Main] WASM initialized')
    app.mount('#app')
}
start()

