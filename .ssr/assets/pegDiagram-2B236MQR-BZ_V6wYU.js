import { t as MermaidParseError, u as createRailroadPegServices } from "./mermaid-parser.core-01GgPkMX.js";
import { n as __name } from "./chunk-Y2CYZVJY-BEdw2br0.js";
import { m as log } from "./src-C4Yh00CL.js";
import "./chunk-WYO6CB5R-Bo6VAQS5.js";
import "./chunk-VAUOI2AC-CYNiIU19.js";
import { n as getStyles, r as renderer, t as db } from "./chunk-MOJQB5TN-DBGuxvzx.js";
import { t as populateCommonDb } from "./chunk-JWPE2WC7-GcjhM0rQ.js";
//#region node_modules/.pnpm/mermaid@11.16.0/node_modules/mermaid/dist/chunks/mermaid.core/pegDiagram-2B236MQR.mjs
var langiumParser = createRailroadPegServices().RailroadPeg.parser.LangiumParser;
var transformOrderedChoice = /* @__PURE__ */ __name((choice) => {
	const alternatives = choice.alternatives.map(transformSequence);
	if (alternatives.length === 1) return alternatives[0];
	return {
		type: "choice",
		alternatives
	};
}, "transformOrderedChoice");
var transformSequence = /* @__PURE__ */ __name((sequence) => {
	const elements = sequence.elements.map(transformPrefix);
	if (elements.length === 1) return elements[0];
	return {
		type: "sequence",
		elements
	};
}, "transformSequence");
var transformPrefix = /* @__PURE__ */ __name((prefix) => {
	const inner = transformSuffix(prefix.suffix);
	if (!prefix.operator) return inner;
	return {
		type: "special",
		text: prefix.operator === "&" ? `&${nodeToLabel(inner)}` : `!${nodeToLabel(inner)}`
	};
}, "transformPrefix");
var nodeToLabel = /* @__PURE__ */ __name((node) => {
	switch (node.type) {
		case "terminal": return `"${node.value}"`;
		case "nonterminal": return node.name;
		case "special": return node.text;
		default: return "(...)";
	}
}, "nodeToLabel");
var transformSuffix = /* @__PURE__ */ __name((suffix) => {
	const inner = transformPrimary(suffix.primary);
	if (!suffix.operator) return inner;
	switch (suffix.operator) {
		case "?": return {
			type: "optional",
			element: inner
		};
		case "*": return {
			type: "repetition",
			element: inner,
			min: 0,
			max: Infinity
		};
		case "+": return {
			type: "repetition",
			element: inner,
			min: 1,
			max: Infinity
		};
		default: throw new Error(`Unsupported PEG suffix operator: ${suffix.operator}`);
	}
}, "transformSuffix");
var transformPrimary = /* @__PURE__ */ __name((primary) => {
	switch (primary.$type) {
		case "PegLiteral": return {
			type: "terminal",
			value: primary.value
		};
		case "PegIdentifier": return {
			type: "nonterminal",
			name: primary.name
		};
		case "PegGroup": return transformOrderedChoice(primary.element);
		case "PegAny": return {
			type: "special",
			text: primary.dot
		};
		default: throw new Error(`Unsupported PEG primary node: ${primary.$type}`);
	}
}, "transformPrimary");
var transformRule = /* @__PURE__ */ __name((rule) => {
	return {
		name: rule.name,
		definition: transformOrderedChoice(rule.definition)
	};
}, "transformRule");
var populateDb = /* @__PURE__ */ __name((ast) => {
	populateCommonDb(ast, db);
	if (ast.title) db.setTitle(ast.title);
	ast.rules.map((rule) => db.addRule(transformRule(rule)));
}, "populateDb");
var diagram = {
	parser: {
		parse: /* @__PURE__ */ __name((input) => {
			db.clear();
			log.debug("[PEG Parser] Starting Langium parse");
			const result = langiumParser.parse(input);
			if (result.lexerErrors.length > 0 || result.parserErrors.length > 0) throw new MermaidParseError(result);
			const ast = result.value;
			log.debug("[PEG Parser] Parsed rules:", ast.rules.length);
			populateDb(ast);
			log.debug("[PEG Parser] Parse complete");
		}, "parse"),
		parser: { yy: db }
	},
	db,
	renderer,
	styles: getStyles
};
//#endregion
export { diagram };
