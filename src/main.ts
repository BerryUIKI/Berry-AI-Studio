import { createApp } from "vue";
import "./style.css";
import App from "./App.vue";
import { dialog } from "./utils/dialog";

createApp(App).directive("dialog", dialog).mount("#app");
