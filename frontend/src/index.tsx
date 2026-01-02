import React from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "./index.css";
import { ThemeProvider } from "./themes/ThemeContext";

const container = document.getElementById("root") || document.body.appendChild(document.createElement("div"));
container.id = "root";
const root = createRoot(container);
root.render(
	React.createElement(
		ThemeProvider,
		null,
		React.createElement(App)
	)
);
