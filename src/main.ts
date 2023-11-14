import { createApp } from "vue";
import "./styles.css";
import './assets/fonts.css'
import "element-plus/dist/index.css"
import App from "./App.vue";
import ElementPlus from 'element-plus'

const app = createApp(App)

app.use(ElementPlus)
app.mount("#app")