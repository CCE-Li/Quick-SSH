import { g as createRailroadServices, t as MermaidParseError } from "./mermaid-parser.core-01GgPkMX.js";
import { n as __name } from "./chunk-Y2CYZVJY-BEdw2br0.js";
import { m as log } from "./src-C4Yh00CL.js";
import "./chunk-WYO6CB5R-Bo6VAQS5.js";
import "./chunk-VAUOI2AC-CYNiIU19.js";
import { n as getStyles, r as renderer, t as db } from "./chunk-MOJQB5TN-DBGuxvzx.js";
import { t as populateCommonDb } from "./chunk-JWPE2WC7-GcjhM0rQ.js";
//#region node_modules/.pnpm/mermaid@11.16.0/node_modules/mermaid/dist/chunks/mermaid.core/railroadDiagram-RFXS5EU6.mjs
var langiumParser = createRailroadServices().Railroad.parser.LangiumParser;
var transformExpression = /* @__PURE__ */ __name((expr) => {
	switch (expr.$type) {
		case "RailroadTerminalExpr": return {
			type: "terminal",
			value: expr.value
		};
		case "RailroadNonTerminalExpr": return {
			type: "nonterminal",
			name: expr.name
		};
		case "RailroadSpecialExpr": return {
			type: "special",
			text: expr.text
		};
		case "RailroadSequenceExpr": {
			const elements = expr.elements.map(transformExpression);
			return elements.length === 1 ? elements[0] : {
				type: "sequence",
				elements
			};
		}
		case "RailroadChoiceExpr": {
			const alternatives = expr.alternatives.map(transformExpression);
			return alternatives.length === 1 ? alternatives[0] : {
				type: "choice",
				alternatives
			};
		}
		case "RailroadOptionalExpr": return {
			type: "optional",
			element: transformExpression(expr.element)
		};
		case "RailroadOneOrMoreExpr": return {
			type: "repetition",
			element: transformExpression(expr.element),
			min: 1,
			max: Infinity
		};
		case "RailroadZeroOrMoreExpr": return {
			type: "repetition",
			element: transformExpression(expr.element),
			min: 0,
			max: Infinity
		};
		default: throw new Error(`Unsupported railroad expression: ${expr.$type}`);
	}
}, "transformExpression");
var transformRule = /* @__PURE__ */ __name((rule) => {
	return {
		name: rule.name,
		definition: transformExpression(rule.definition)
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
			log.debug("[Railroad Parser] Starting Langium parse");
			const result = langiumParser.parse(input);
			if (result.lexerErrors.length > 0 || result.parserErrors.length > 0) throw new MermaidParseError(result);
			const ast = result.value;
			log.debug("[Railroad Parser] Parsed rules:", ast.rules.length);
			populateDb(ast);
			log.debug("[Railroad Parser] Parse complete");
		}, "parse"),
		parser: { yy: db }
	},
	db,
	renderer,
	styles: getStyles
};
//#endregion
export { diagram };
