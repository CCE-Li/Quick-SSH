import { m as createRailroadAbnfServices, t as MermaidParseError } from "./mermaid-parser.core-01GgPkMX.js";
import { n as __name } from "./chunk-Y2CYZVJY-BEdw2br0.js";
import { m as log } from "./src-C4Yh00CL.js";
import "./chunk-WYO6CB5R-Bo6VAQS5.js";
import "./chunk-VAUOI2AC-CYNiIU19.js";
import { n as getStyles, r as renderer, t as db } from "./chunk-MOJQB5TN-DBGuxvzx.js";
import { t as populateCommonDb } from "./chunk-JWPE2WC7-GcjhM0rQ.js";
//#region node_modules/.pnpm/mermaid@11.16.0/node_modules/mermaid/dist/chunks/mermaid.core/abnfDiagram-VRR7QNED.mjs
var langiumParser = createRailroadAbnfServices().RailroadAbnf.parser.LangiumParser;
var transformAlternation = /* @__PURE__ */ __name((alt) => {
	const alternatives = alt.alternatives.map(transformConcatenation);
	if (alternatives.length === 1) return alternatives[0];
	return {
		type: "choice",
		alternatives
	};
}, "transformAlternation");
var transformConcatenation = /* @__PURE__ */ __name((concat) => {
	const elements = concat.elements.map(transformElement);
	if (elements.length === 1) return elements[0];
	return {
		type: "sequence",
		elements
	};
}, "transformConcatenation");
var parseRepeat = /* @__PURE__ */ __name((repeat) => {
	if (repeat.includes("*")) {
		const [minStr, maxStr] = repeat.split("*");
		return {
			min: minStr ? parseInt(minStr, 10) : 0,
			max: maxStr ? parseInt(maxStr, 10) : Infinity
		};
	}
	const exact = parseInt(repeat, 10);
	return {
		min: exact,
		max: exact
	};
}, "parseRepeat");
var transformElement = /* @__PURE__ */ __name((element) => {
	const inner = transformPrimary(element.primary);
	if (!element.repeat) return inner;
	const { min, max } = parseRepeat(element.repeat);
	if (min === 0 && max === 1) return {
		type: "optional",
		element: inner
	};
	return {
		type: "repetition",
		element: inner,
		min,
		max
	};
}, "transformElement");
var transformPrimary = /* @__PURE__ */ __name((primary) => {
	switch (primary.$type) {
		case "AbnfStringLiteral": return {
			type: "terminal",
			value: primary.value
		};
		case "AbnfNumVal": return {
			type: "terminal",
			value: primary.value
		};
		case "AbnfRuleName": return {
			type: "nonterminal",
			name: primary.name
		};
		case "AbnfGroup": return transformAlternation(primary.element);
		case "AbnfOptionalGroup": return {
			type: "optional",
			element: transformAlternation(primary.element)
		};
		default: throw new Error(`Unsupported ABNF primary node: ${primary.$type}`);
	}
}, "transformPrimary");
var transformRule = /* @__PURE__ */ __name((rule) => {
	return {
		name: rule.name,
		definition: transformAlternation(rule.definition)
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
			log.debug("[ABNF Parser] Starting Langium parse");
			const result = langiumParser.parse(input);
			if (result.lexerErrors.length > 0 || result.parserErrors.length > 0) throw new MermaidParseError(result);
			const ast = result.value;
			log.debug("[ABNF Parser] Parsed rules:", ast.rules.length);
			populateDb(ast);
			log.debug("[ABNF Parser] Parse complete");
		}, "parse"),
		parser: { yy: db }
	},
	db,
	renderer,
	styles: getStyles
};
//#endregion
export { diagram };
