import { n as __name } from "./chunk-Y2CYZVJY-BEdw2br0.js";
import { m as log } from "./src-C4Yh00CL.js";
import { b as getConfig, s as common_default } from "./chunk-WYO6CB5R-Bo6VAQS5.js";
import { d as interpolateToCurve } from "./chunk-ICXQ74PX-BYzvWCOG.js";
import { a as insertNode, i as insertCluster, s as labelHelper } from "./chunk-ZGVPDNZ5-CKZI2aLQ.js";
import { a as markers_default, i as insertEdgeLabel, o as positionEdgeLabel, r as insertEdge } from "./chunk-52WLFC77-Cnswye_V.js";
//#region node_modules/.pnpm/mermaid@11.16.0/node_modules/mermaid/dist/chunks/mermaid.core/chunk-FWX5IMBZ.mjs
var internalHelpers = {
	common: common_default,
	getConfig,
	insertCluster,
	insertEdge,
	insertEdgeLabel,
	insertMarkers: markers_default,
	insertNode,
	interpolateToCurve,
	labelHelper,
	log,
	positionEdgeLabel
};
var layoutAlgorithms = {};
var registerLayoutLoaders = /* @__PURE__ */ __name((loaders) => {
	for (const loader of loaders) layoutAlgorithms[loader.name] = loader;
}, "registerLayoutLoaders");
(/* @__PURE__ */ __name(() => {
	registerLayoutLoaders([
		{
			name: "dagre",
			loader: /* @__PURE__ */ __name(async () => await import("./dagre-VKFMJZFB-C5DtoB6L.js"), "loader")
		},
		{
			name: "swimlane",
			loader: /* @__PURE__ */ __name(async () => await import("./swimlanes-5IMT3BWC-DGdOoyZe.js"), "loader")
		},
		...[{
			name: "cose-bilkent",
			loader: /* @__PURE__ */ __name(async () => await import("./cose-bilkent-JH36ORCC-0IPsa_Xf.js"), "loader")
		}]
	]);
}, "registerDefaultLayoutLoaders"))();
var render = /* @__PURE__ */ __name(async (data4Layout, svg, positions) => {
	if (!(data4Layout.layoutAlgorithm in layoutAlgorithms)) throw new Error(`Unknown layout algorithm: ${data4Layout.layoutAlgorithm}`);
	if (data4Layout.diagramId) for (const node of data4Layout.nodes) {
		const originalDomId = node.domId || node.id;
		node.domId = `${data4Layout.diagramId}-${originalDomId}`;
	}
	const layoutDefinition = layoutAlgorithms[data4Layout.layoutAlgorithm];
	const layoutRenderer = await layoutDefinition.loader();
	const { theme, themeVariables } = data4Layout.config;
	const { useGradient, gradientStart, gradientStop } = themeVariables;
	const svgId = svg.attr("id");
	svg.append("defs").append("filter").attr("id", `${svgId}-drop-shadow`).attr("height", "130%").attr("width", "130%").append("feDropShadow").attr("dx", "4").attr("dy", "4").attr("stdDeviation", 0).attr("flood-opacity", "0.06").attr("flood-color", `${theme?.includes("dark") ? "#FFFFFF" : "#000000"}`);
	svg.append("defs").append("filter").attr("id", `${svgId}-drop-shadow-small`).attr("height", "150%").attr("width", "150%").append("feDropShadow").attr("dx", "2").attr("dy", "2").attr("stdDeviation", 0).attr("flood-opacity", "0.06").attr("flood-color", `${theme?.includes("dark") ? "#FFFFFF" : "#000000"}`);
	if (useGradient) {
		const gradient = svg.append("linearGradient").attr("id", svg.attr("id") + "-gradient").attr("gradientUnits", "objectBoundingBox").attr("x1", "0%").attr("y1", "0%").attr("x2", "100%").attr("y2", "0%");
		gradient.append("svg:stop").attr("offset", "0%").attr("stop-color", gradientStart).attr("stop-opacity", 1);
		gradient.append("svg:stop").attr("offset", "100%").attr("stop-color", gradientStop).attr("stop-opacity", 1);
	}
	return layoutRenderer.render(data4Layout, svg, internalHelpers, { algorithm: layoutDefinition.algorithm }, positions);
}, "render");
var getRegisteredLayoutAlgorithm = /* @__PURE__ */ __name((algorithm = "", { fallback = "dagre" } = {}) => {
	if (algorithm in layoutAlgorithms) return algorithm;
	if (fallback in layoutAlgorithms) {
		log.warn(`Layout algorithm ${algorithm} is not registered. Using ${fallback} as fallback.`);
		return fallback;
	}
	throw new Error(`Both layout algorithms ${algorithm} and ${fallback} are not registered.`);
}, "getRegisteredLayoutAlgorithm");
//#endregion
export { registerLayoutLoaders as n, render as r, getRegisteredLayoutAlgorithm as t };
