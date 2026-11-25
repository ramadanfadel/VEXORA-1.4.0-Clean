/*
 * VEXORA UI smoke test — runs the real app.js against a mocked Tauri bridge.
 * No native window is needed: this catches broken handlers, missing DOM nodes
 * and runtime exceptions that `node --check` cannot detect.
 */
const fs = require("fs");
const path = require("path");
const assert = require("assert");
const { JSDOM, VirtualConsole } = require("jsdom");

const root = path.resolve(__dirname, "..");
const html = fs.readFileSync(path.join(root, "src", "index.html"), "utf8")
  .replace(/<script src="app\.js"[^>]*><\/script>/, "");
const appJs = fs.readFileSync(path.join(root, "src", "app.js"), "utf8");

const errors = [];
const vc = new VirtualConsole();
vc.on("jsdomError", (e) => errors.push(String(e && e.message ? e.message : e)));
vc.on("error", (m) => errors.push(String(m)));

const settings = {
  language: "ar",
  theme: "dark",
  lastPage: "settings",
  windowW: 1200,
  windowH: 800,
  appearanceLayout: "grid",
  appearanceAccent: "cyan",
  appearanceMode: "dark",
  appearanceGlow: true,
  appearanceAnimation: true,
  appearanceMigrated: true,
  githubRepo: "ramadanfadel/VEXORA-1.4.0-Clean",
  exclusions: [],
  disabledCleaners: [],
  schedule: { enabled: false, frequency: "weekly", time: "03:00", lastRun: 0 },
};

const responses = {
  get_settings: settings,
  save_settings: settings,
  app_info: { version: "1.4.3", dataDir: "", reportsDir: "", admin: false },
  get_catalog: { cleaners: [] },
  programs_list: [],
};

const dom = new JSDOM(html, {
  url: "http://localhost/",
  runScripts: "outside-only",
  pretendToBeVisual: true,
  virtualConsole: vc,
});
const { window } = dom;
window.__TAURI__ = {
  core: {
    invoke: async (cmd) => {
      if (Object.prototype.hasOwnProperty.call(responses, cmd)) return responses[cmd];
      return {};
    },
  },
  event: { listen: async () => () => {} },
  window: { getCurrentWindow: () => ({ setSize() {}, setPosition() {}, show() {}, hide() {} }) },
};
window.matchMedia = window.matchMedia || (() => ({ matches: false, addListener() {}, removeListener() {} }));
window.scrollTo = () => {};

const wait = (ms) => new Promise((r) => setTimeout(r, ms));

(async () => {
  window.eval(appJs);
  window.document.dispatchEvent(new window.Event("DOMContentLoaded", { bubbles: true }));
  await wait(350);

  const $ = (id) => window.document.getElementById(id);
  assert.ok($("nav"), "sidebar nav must render");
  assert.ok(window.document.querySelectorAll("[data-nav]").length >= 10, "all nav items must render");
  assert.ok($("vexAppearance"), "Appearance Studio card must render");
  assert.ok($("vexPreview"), "live preview container must render");
  assert.ok($("vexPreview").children.length > 0, "live preview must have content");
  assert.ok($("vexMyStyleLabel"), "My Style label must render");
  assert.ok($("vexUpdateDock"), "update badge must render");
  assert.ok($("vexVersionUpdate"), "version update icon must render");

  // Accent click must update preview without rebuilding it.
  const preview = $("vexPreview");
  const before = preview.innerHTML;
  const pink = window.document.querySelector('[data-vex-accent="pink"]');
  assert.ok(pink, "pink swatch must exist");
  pink.click();
  await wait(20);
  assert.notStrictEqual(preview.style.getPropertyValue("--vex-accent"), "", "accent var must be set");
  assert.strictEqual(preview.innerHTML, before, "colour change must not rebuild preview DOM");

  // Layout click rebuilds the preview with the new structure.
  const dock = window.document.querySelector('[data-vex-layout="dock"]');
  assert.ok(dock, "dock layout button must exist");
  await dock.onclick();
  await wait(20);
  assert.strictEqual(preview.dataset.layout, "dock", "layout change must rebuild the preview");

  // Appearance is actually applied to the shell.
  assert.strictEqual(window.document.body.dataset.vexLayout, "grid", "initial layout must be applied");
  assert.strictEqual(window.document.documentElement.dataset.vexMode, "dark");

  assert.deepStrictEqual(errors, [], `no runtime errors expected, got: ${errors.join(" | ")}`);
  console.log("UI smoke test: PASS (nav, studio, preview, swatches, layouts, shell, no JS errors)");
  dom.window.close();
})().catch((e) => {
  console.error("UI smoke test: FAIL");
  console.error(e);
  process.exitCode = 1;
});
