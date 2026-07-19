import { n as __name } from "./chunk-Y2CYZVJY-BEdw2br0.js";
import { m as log } from "./src-C4Yh00CL.js";
import { c as configureSvgSize } from "./chunk-WYO6CB5R-Bo6VAQS5.js";
//#region node_modules/.pnpm/mermaid@11.16.0/node_modules/mermaid/dist/chunks/mermaid.core/chunk-VR4S4FIN.mjs
var setupViewPortForSVG = /* @__PURE__ */ __name((svg, padding, cssDiagram, useMaxWidth) => {
	svg.attr("class", cssDiagram);
	const { width, height, x, y } = calculateDimensionsWithPadding(svg, padding);
	configureSvgSize(svg, height, width, useMaxWidth);
	const viewBox = createViewBox(x, y, width, height, padding);
	svg.attr("viewBox", viewBox);
	log.debug(`viewBox configured: ${viewBox} with padding: ${padding}`);
}, "setupViewPortForSVG");
var calculateDimensionsWithPadding = /* @__PURE__ */ __name((svg, padding) => {
	const bounds = svg.node()?.getBBox() || {
		width: 0,
		height: 0,
		x: 0,
		y: 0
	};
	return {
		width: bounds.width + padding * 2,
		height: bounds.height + padding * 2,
		x: bounds.x,
		y: bounds.y
	};
}, "calculateDimensionsWithPadding");
var createViewBox = /* @__PURE__ */ __name((x, y, width, height, padding) => {
	return `${x - padding} ${y - padding} ${width} ${height}`;
}, "createViewBox");
//#endregion
export { setupViewPortForSVG as t };
