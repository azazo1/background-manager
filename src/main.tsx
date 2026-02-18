import React from "react";
import ReactDOM from "react-dom/client";
import { I18nextProvider } from "react-i18next";
import { Toaster } from "sonner";
import i18n from "./i18n/config";
import "./App.css";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <I18nextProvider i18n={i18n}>
      <App />
      <Toaster position="bottom-right" richColors />
    </I18nextProvider>
  </React.StrictMode>,
);
