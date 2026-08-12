import DefaultTheme from "vitepress/theme";
import "./custom.css";
import ArqenConsole from "./ArqenConsole.vue";
import CurrentVersion from "./CurrentVersion.vue";
import MermaidDiagram from "./MermaidDiagram.vue";
import ProjectStatus from "./ProjectStatus.vue";

export default {
  ...DefaultTheme,
  enhanceApp({ app }) {
    app.component("ArqenConsole", ArqenConsole);
    app.component("CurrentVersion", CurrentVersion);
    app.component("MermaidDiagram", MermaidDiagram);
    app.component("ProjectStatus", ProjectStatus);
  },
};
