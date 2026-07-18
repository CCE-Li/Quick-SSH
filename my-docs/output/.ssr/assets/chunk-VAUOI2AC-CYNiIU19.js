import { n as __name } from "./chunk-Y2CYZVJY-BEdw2br0.js";
import { p as select_default } from "./src-C4Yh00CL.js";
import { x as getConfig2 } from "./chunk-WYO6CB5R-Bo6VAQS5.js";
//#region node_modules/.pnpm/mermaid@11.16.0/node_modules/mermaid/dist/chunks/mermaid.core/chunk-VAUOI2AC.mjs
var selectSvgElement = /* @__PURE__ */ __name((id) => {
	const { securityLevel } = getConfig2();
	let root = select_default("body");
	if (securityLevel === "sandbox") root = select_default((select_default(`#i${id}`).node()?.contentDocument ?? document).body);
	return root.select(`#${id}`);
}, "selectSvgElement");
//#endregion
export { selectSvgElement as t };
