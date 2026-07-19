import { n as __name } from "./chunk-Y2CYZVJY-BEdw2br0.js";
import { h as setLogLevel, m as log, p as select_default } from "./src-C4Yh00CL.js";
import { $ as purify, C as getDiagramLoader, E as getSiteConfig, I as reset, L as sanitizeCss, N as registerDiagram, P as registerLazyLoadedDiagrams, Q as updateSiteConfig, S as getDiagram, T as getEffectiveHtmlLabels, V as saveConfigFromInitialize, W as setConfig, X as styles_default, Z as themes_default, _ as frontMatterRegex, b as getConfig, c as configureSvgSize, g as evaluate, l as cssStyleSheetToString, m as detectors, n as addDirective, p as detectType, q as setSiteConfig, r as assignWithDepth_default, t as UnknownDiagramError, u as defaultConfig } from "./chunk-WYO6CB5R-Bo6VAQS5.js";
import { S as isBuffer, a as decodeEntities, f as isDetailedError, g as utils_default, h as removeDirectives, i as cleanAndMerge, o as encodeEntities, v as isTypedArray, x as isArrayLike, y as isArguments } from "./chunk-ICXQ74PX-BYzvWCOG.js";
import { t as selectSvgElement } from "./chunk-VAUOI2AC-CYNiIU19.js";
import { r as registerIconPacks } from "./chunk-HOUHSVGY-DMo3pC7M.js";
import { r as dedent } from "./chunk-Q4XR5HBZ-D8VDN49y.js";
import "./chunk-7BUUIJ7U-HmUKNdXe.js";
import "./chunk-OGEWGWER-C_0xq-Zy.js";
import "./chunk-C7G6YPKG-PS92SJXY.js";
import "./chunk-ZGVPDNZ5-CKZI2aLQ.js";
import "./chunk-52WLFC77-Cnswye_V.js";
import { n as registerLayoutLoaders } from "./chunk-FWX5IMBZ-CdzV97ed.js";
import { n as load, t as JSON_SCHEMA } from "./chunk-ZIRB5QZD-CSCFClLz.js";
//#region node_modules/.pnpm/es-toolkit@1.49.0/node_modules/es-toolkit/dist/compat/_internal/isPrototype.mjs
function isPrototype(value) {
	const constructor = value?.constructor;
	return value === (typeof constructor === "function" ? constructor.prototype : Object.prototype);
}
//#endregion
//#region node_modules/.pnpm/es-toolkit@1.49.0/node_modules/es-toolkit/dist/compat/predicate/isEmpty.mjs
/**
* Checks if a given value is empty.
*
* - If the given value is a string, checks if it is an empty string.
* - If the given value is an array, `Map`, or `Set`, checks if its size is 0.
* - If the given value is an [array-like object](../predicate/isArrayLike.md), checks if its length is 0.
* - If the given value is an object, checks if it is an empty object with no properties.
* - Primitive values (booleans, numbers, or bigints) are considered empty.
*
* @param [value] - The value to check.
* @returns `true` if the value is empty, `false` otherwise.
*
* @example
* isEmpty(); // true
* isEmpty(null); // true
* isEmpty(""); // true
* isEmpty([]); // true
* isEmpty({}); // true
* isEmpty(new Map()); // true
* isEmpty(new Set()); // true
* isEmpty("hello"); // false
* isEmpty([1, 2, 3]); // false
* isEmpty({ a: 1 }); // false
* isEmpty(new Map([["key", "value"]])); // false
* isEmpty(new Set([1, 2, 3])); // false
*/
function isEmpty(value) {
	if (value == null) return true;
	if (isArrayLike(value)) {
		if (typeof value.splice !== "function" && typeof value !== "string" && !isBuffer(value) && !isTypedArray(value) && !isArguments(value)) return false;
		return value.length === 0;
	}
	if (typeof value === "object" || typeof value === "function") {
		if (value instanceof Map || value instanceof Set) return value.size === 0;
		const keys = Object.keys(value);
		if (isPrototype(value)) return keys.filter((x) => x !== "constructor").length === 0;
		return keys.length === 0;
	}
	return true;
}
//#endregion
//#region node_modules/.pnpm/stylis@4.4.0/node_modules/stylis/src/Enum.js
var COMMENT = "comm";
var RULESET = "rule";
var DECLARATION = "decl";
var IMPORT = "@import";
var NAMESPACE = "@namespace";
var KEYFRAMES = "@keyframes";
var LAYER = "@layer";
//#endregion
//#region node_modules/.pnpm/stylis@4.4.0/node_modules/stylis/src/Utility.js
/**
* @param {number}
* @return {number}
*/
var abs = Math.abs;
/**
* @param {number}
* @return {string}
*/
var from = String.fromCharCode;
/**
* @param {string} value
* @return {string}
*/
function trim(value) {
	return value.trim();
}
/**
* @param {string} value
* @param {(string|RegExp)} pattern
* @param {string} replacement
* @return {string}
*/
function replace(value, pattern, replacement) {
	return value.replace(pattern, replacement);
}
/**
* @param {string} value
* @param {number} index
* @return {number}
*/
function charat(value, index) {
	return value.charCodeAt(index) | 0;
}
/**
* @param {string} value
* @param {number} begin
* @param {number} end
* @return {string}
*/
function substr(value, begin, end) {
	return value.slice(begin, end);
}
/**
* @param {string} value
* @return {number}
*/
function strlen(value) {
	return value.length;
}
/**
* @param {any[]} value
* @return {number}
*/
function sizeof(value) {
	return value.length;
}
/**
* @param {any} value
* @param {any[]} array
* @return {any}
*/
function append(value, array) {
	return array.push(value), value;
}
//#endregion
//#region node_modules/.pnpm/stylis@4.4.0/node_modules/stylis/src/Tokenizer.js
var line = 1;
var column = 1;
var length = 0;
var position = 0;
var character = 0;
var characters = "";
/**
* @param {string} value
* @param {object | null} root
* @param {object | null} parent
* @param {string} type
* @param {string[] | string} props
* @param {object[] | string} children
* @param {object[]} siblings
* @param {number} length
*/
function node(value, root, parent, type, props, children, length, siblings) {
	return {
		value,
		root,
		parent,
		type,
		props,
		children,
		line,
		column,
		length,
		return: "",
		siblings
	};
}
/**
* @return {number}
*/
function char() {
	return character;
}
/**
* @return {number}
*/
function prev() {
	character = position > 0 ? charat(characters, --position) : 0;
	if (column--, character === 10) column = 1, line--;
	return character;
}
/**
* @return {number}
*/
function next() {
	character = position < length ? charat(characters, position++) : 0;
	if (column++, character === 10) column = 1, line++;
	return character;
}
/**
* @return {number}
*/
function peek() {
	return charat(characters, position);
}
/**
* @return {number}
*/
function caret() {
	return position;
}
/**
* @param {number} begin
* @param {number} end
* @return {string}
*/
function slice(begin, end) {
	return substr(characters, begin, end);
}
/**
* @param {number} type
* @return {number}
*/
function token(type) {
	switch (type) {
		case 0:
		case 9:
		case 10:
		case 13:
		case 32: return 5;
		case 33:
		case 43:
		case 44:
		case 47:
		case 62:
		case 64:
		case 126:
		case 59:
		case 123:
		case 125: return 4;
		case 58: return 3;
		case 34:
		case 39:
		case 40:
		case 91: return 2;
		case 41:
		case 93: return 1;
	}
	return 0;
}
/**
* @param {string} value
* @return {any[]}
*/
function alloc(value) {
	return line = column = 1, length = strlen(characters = value), position = 0, [];
}
/**
* @param {any} value
* @return {any}
*/
function dealloc(value) {
	return characters = "", value;
}
/**
* @param {number} type
* @return {string}
*/
function delimit(type) {
	return trim(slice(position - 1, delimiter(type === 91 ? type + 2 : type === 40 ? type + 1 : type)));
}
/**
* @param {number} type
* @return {string}
*/
function whitespace(type) {
	while (character = peek()) if (character < 33) next();
	else break;
	return token(type) > 2 || token(character) > 3 ? "" : " ";
}
/**
* @param {number} index
* @param {number} count
* @return {string}
*/
function escaping(index, count) {
	while (--count && next()) if (character < 48 || character > 102 || character > 57 && character < 65 || character > 70 && character < 97) break;
	return slice(index, caret() + (count < 6 && peek() == 32 && next() == 32));
}
/**
* @param {number} type
* @return {number}
*/
function delimiter(type) {
	while (next()) switch (character) {
		case type: return position;
		case 34:
		case 39:
			if (type !== 34 && type !== 39) delimiter(character);
			break;
		case 40:
			if (type === 41) delimiter(type);
			break;
		case 92:
			next();
			break;
	}
	return position;
}
/**
* @param {number} type
* @param {number} index
* @return {number}
*/
function commenter(type, index) {
	while (next()) if (type + character === 57) break;
	else if (type + character === 84 && peek() === 47) break;
	return "/*" + slice(index, position - 1) + "*" + from(type === 47 ? type : next());
}
/**
* @param {number} index
* @return {string}
*/
function identifier(index) {
	while (!token(peek())) next();
	return slice(index, position);
}
//#endregion
//#region node_modules/.pnpm/stylis@4.4.0/node_modules/stylis/src/Parser.js
/**
* @param {string} value
* @return {object[]}
*/
function compile(value) {
	return dealloc(parse$1("", null, null, null, [""], value = alloc(value), 0, [0], value));
}
/**
* @param {string} value
* @param {object} root
* @param {object?} parent
* @param {string[]} rule
* @param {string[]} rules
* @param {string[]} rulesets
* @param {number[]} pseudo
* @param {number[]} points
* @param {string[]} declarations
* @return {object}
*/
function parse$1(value, root, parent, rule, rules, rulesets, pseudo, points, declarations) {
	var index = 0;
	var offset = 0;
	var length = pseudo;
	var atrule = 0;
	var property = 0;
	var previous = 0;
	var variable = 1;
	var scanning = 1;
	var ampersand = 1;
	var parens = 0;
	var character = 0;
	var type = "";
	var props = rules;
	var children = rulesets;
	var reference = rule;
	var characters = type;
	while (scanning) switch (previous = character, character = next()) {
		case 40:
			if (previous != 108 && charat(characters, length - 1) == 58) parens++, characters += "(";
			else characters += delimit(character);
			break;
		case 41:
			parens--, characters += ")";
			break;
		case 34:
		case 39:
		case 91:
			characters += delimit(character);
			break;
		case 9:
		case 10:
		case 13:
		case 32:
			if (parens > 0) {
				characters += from(character);
				break;
			}
			characters += whitespace(previous);
			break;
		case 92:
			characters += escaping(caret() - 1, 7);
			continue;
		case 47:
			switch (peek()) {
				case 42:
				case 47:
					append(comment(commenter(next(), caret()), root, parent, declarations), declarations);
					if ((token(previous || 1) == 5 || token(peek() || 1) == 5) && strlen(characters) && substr(characters, -1, void 0) !== " ") characters += " ";
					break;
				default: characters += "/";
			}
			break;
		case 123 * variable: points[index++] = strlen(characters) * ampersand;
		case 125 * variable:
		case 59:
		case 0:
			if (parens > 0 && character) {
				characters += from(character);
				break;
			}
			switch (character) {
				case 0:
				case 125: scanning = 0;
				case 59 + offset:
					if (ampersand == -1) characters = replace(characters, /\f/g, "");
					if (property > 0 && (strlen(characters) - length || variable === 0)) append(property > 32 ? declaration(characters + ";", rule, parent, length - 1, declarations) : declaration(replace(characters, " ", "") + ";", rule, parent, length - 2, declarations), declarations);
					break;
				case 59: characters += ";";
				default:
					append(reference = ruleset(characters, root, parent, index, offset, rules, points, type, props = [], children = [], length, rulesets), rulesets);
					if (character === 123) if (offset === 0) parse$1(characters, root, reference, reference, props, rulesets, length, points, children);
					else {
						switch (atrule) {
							case 99: if (charat(characters, 3) === 110) break;
							case 108: if (charat(characters, 2) === 97) break;
							default: offset = 0;
							case 100:
							case 109:
							case 115:
						}
						if (offset) parse$1(value, reference, reference, rule && append(ruleset(value, reference, reference, 0, 0, rules, points, type, rules, props = [], length, children), children), rules, children, length, points, rule ? props : children);
						else parse$1(characters, reference, reference, reference, [""], children, 0, points, children);
					}
			}
			index = offset = property = 0, variable = ampersand = 1, type = characters = "", length = pseudo;
			break;
		case 58: length = 1 + strlen(characters), property = previous;
		default:
			if (variable < 1) {
				if (character == 123) --variable;
				else if (character == 125 && variable++ == 0 && prev() == 125) continue;
			}
			switch (characters += from(character), character * variable) {
				case 38:
					ampersand = offset > 0 ? 1 : (characters += "\f", -1);
					break;
				case 44:
					if (parens > 0) break;
					points[index++] = (strlen(characters) - 1) * ampersand, ampersand = 1;
					break;
				case 64:
					if (peek() === 45) characters += delimit(next());
					atrule = peek(), offset = length = strlen(type = characters += identifier(caret())), character++;
					break;
				case 45: if (previous === 45 && strlen(characters) == 2) variable = 0;
			}
	}
	return rulesets;
}
/**
* @param {string} value
* @param {object} root
* @param {object?} parent
* @param {number} index
* @param {number} offset
* @param {string[]} rules
* @param {number[]} points
* @param {string} type
* @param {string[]} props
* @param {string[]} children
* @param {number} length
* @param {object[]} siblings
* @return {object}
*/
function ruleset(value, root, parent, index, offset, rules, points, type, props, children, length, siblings) {
	var post = offset - 1;
	var rule = offset === 0 ? rules : [""];
	var size = sizeof(rule);
	for (var i = 0, j = 0, k = 0; i < index; ++i) for (var x = 0, y = substr(value, post + 1, post = abs(j = points[i])), z = value; x < size; ++x) if (z = trim(j > 0 ? rule[x] + " " + y : replace(y, /&\f/g, rule[x]))) props[k++] = z;
	return node(value, root, parent, offset === 0 ? RULESET : type, props, children, length, siblings);
}
/**
* @param {number} value
* @param {object} root
* @param {object?} parent
* @param {object[]} siblings
* @return {object}
*/
function comment(value, root, parent, siblings) {
	return node(value, root, parent, COMMENT, from(char()), substr(value, 2, -2), 0, siblings);
}
/**
* @param {string} value
* @param {object} root
* @param {object?} parent
* @param {number} length
* @param {object[]} siblings
* @return {object}
*/
function declaration(value, root, parent, length, siblings) {
	return node(value, root, parent, DECLARATION, substr(value, 0, length), substr(value, length + 1, -1), length, siblings);
}
//#endregion
//#region node_modules/.pnpm/stylis@4.4.0/node_modules/stylis/src/Serializer.js
/**
* @param {object[]} children
* @param {function} callback
* @return {string}
*/
function serialize(children, callback) {
	var output = "";
	for (var i = 0; i < children.length; i++) output += callback(children[i], i, children, callback) || "";
	return output;
}
/**
* @param {object} element
* @param {number} index
* @param {object[]} children
* @param {function} callback
* @return {string}
*/
function stringify(element, index, children, callback) {
	switch (element.type) {
		case LAYER: if (element.children.length) break;
		case IMPORT:
		case NAMESPACE:
		case DECLARATION: return element.return = element.return || element.value;
		case COMMENT: return "";
		case KEYFRAMES: return element.return = element.value + "{" + serialize(element.children, callback) + "}";
		case RULESET: if (!strlen(element.value = element.props.join(","))) return "";
	}
	return strlen(children = serialize(element.children, callback)) ? element.return = element.value + "{" + children + "}" : "";
}
//#endregion
//#region node_modules/.pnpm/stylis@4.4.0/node_modules/stylis/src/Middleware.js
/**
* @param {function[]} collection
* @return {function}
*/
function middleware(collection) {
	var length = sizeof(collection);
	return function(element, index, children, callback) {
		var output = "";
		for (var i = 0; i < length; i++) output += collection[i](element, index, children, callback) || "";
		return output;
	};
}
//#endregion
//#region node_modules/.pnpm/mermaid@11.16.0/node_modules/mermaid/dist/mermaid.core.mjs
var id = "c4";
var c4Detector_default = {
	id,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*C4Context|C4Container|C4Component|C4Dynamic|C4Deployment/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./c4Diagram-LMCZKHZV-DfRJ6Pxq.js");
		return {
			id,
			diagram: diagram2
		};
	}, "loader")
};
var id2 = "flowchart";
var flowDetector_default = {
	id: id2,
	detector: /* @__PURE__ */ __name((txt, config) => {
		if (config?.flowchart?.defaultRenderer === "dagre-wrapper" || config?.flowchart?.defaultRenderer === "elk") return false;
		return /^\s*graph/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./flowDiagram-23GEKE2U-YW6WhgY_.js");
		return {
			id: id2,
			diagram: diagram2
		};
	}, "loader")
};
var id3 = "flowchart-v2";
var flowDetector_v2_default = {
	id: id3,
	detector: /* @__PURE__ */ __name((txt, config) => {
		if (config?.flowchart?.defaultRenderer === "dagre-d3") return false;
		if (config?.flowchart?.defaultRenderer === "elk") config.layout = "elk";
		if (/^\s*graph/.test(txt) && config?.flowchart?.defaultRenderer === "dagre-wrapper") return true;
		return /^\s*flowchart/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./flowDiagram-23GEKE2U-YW6WhgY_.js");
		return {
			id: id3,
			diagram: diagram2
		};
	}, "loader")
};
var id4 = "swimlane";
var detector_default = {
	id: id4,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*swimlane-beta\b/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./swimlanesDiagram-G3AALYLV-DVkGXIrt.js");
		return {
			id: id4,
			diagram: diagram2
		};
	}, "loader")
};
var id5 = "er";
var erDetector_default = {
	id: id5,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*erDiagram/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./erDiagram-Q63AITRT-C-eUBJyZ.js");
		return {
			id: id5,
			diagram: diagram2
		};
	}, "loader")
};
var id6 = "gitGraph";
var gitGraphDetector_default = {
	id: id6,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*gitGraph/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./gitGraphDiagram-IHSO6WYX-DnHUAgiw.js");
		return {
			id: id6,
			diagram: diagram2
		};
	}, "loader")
};
var id7 = "gantt";
var ganttDetector_default = {
	id: id7,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*gantt/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./ganttDiagram-NO4QXBWP-woCcfQ6L.js");
		return {
			id: id7,
			diagram: diagram2
		};
	}, "loader")
};
var id8 = "info";
var info = {
	id: id8,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*info/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./infoDiagram-FWYZ7A6U-UsyFM3S3.js");
		return {
			id: id8,
			diagram: diagram2
		};
	}, "loader")
};
var id9 = "pie";
var pie = {
	id: id9,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*pie/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./pieDiagram-ENE6RG2P-VZgm96BA.js");
		return {
			id: id9,
			diagram: diagram2
		};
	}, "loader")
};
var id10 = "quadrantChart";
var quadrantDetector_default = {
	id: id10,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*quadrantChart/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./quadrantDiagram-ABIIQ3AL-4VPaeF07.js");
		return {
			id: id10,
			diagram: diagram2
		};
	}, "loader")
};
var id11 = "xychart";
var xychartDetector_default = {
	id: id11,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*xychart(-beta)?/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./xychartDiagram-FW5EYKEG-BXNB5eOD.js");
		return {
			id: id11,
			diagram: diagram2
		};
	}, "loader")
};
var id12 = "requirement";
var requirementDetector_default = {
	id: id12,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*requirement(Diagram)?/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./requirementDiagram-TGXJPOKE-CMejO-wf.js");
		return {
			id: id12,
			diagram: diagram2
		};
	}, "loader")
};
var id13 = "sequence";
var sequenceDetector_default = {
	id: id13,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*sequenceDiagram/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./sequenceDiagram-DBY2YBRQ-Cxz7Kr_w.js");
		return {
			id: id13,
			diagram: diagram2
		};
	}, "loader")
};
var id14 = "class";
var classDetector_default = {
	id: id14,
	detector: /* @__PURE__ */ __name((txt, config) => {
		if (config?.class?.defaultRenderer === "dagre-wrapper") return false;
		return /^\s*classDiagram/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./classDiagram-OUVF2IWQ-DAkLdUZV.js");
		return {
			id: id14,
			diagram: diagram2
		};
	}, "loader")
};
var id15 = "classDiagram";
var classDetector_V2_default = {
	id: id15,
	detector: /* @__PURE__ */ __name((txt, config) => {
		if (/^\s*classDiagram/.test(txt) && config?.class?.defaultRenderer === "dagre-wrapper") return true;
		return /^\s*classDiagram-v2/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./classDiagram-v2-EOCWNBFH-BjZw4M9g.js");
		return {
			id: id15,
			diagram: diagram2
		};
	}, "loader")
};
var id16 = "state";
var stateDetector_default = {
	id: id16,
	detector: /* @__PURE__ */ __name((txt, config) => {
		if (config?.state?.defaultRenderer === "dagre-wrapper") return false;
		return /^\s*stateDiagram/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./stateDiagram-2N3HPSRC-BZ-PiIUF.js");
		return {
			id: id16,
			diagram: diagram2
		};
	}, "loader")
};
var id17 = "stateDiagram";
var stateDetector_V2_default = {
	id: id17,
	detector: /* @__PURE__ */ __name((txt, config) => {
		if (/^\s*stateDiagram-v2/.test(txt)) return true;
		if (/^\s*stateDiagram/.test(txt) && config?.state?.defaultRenderer === "dagre-wrapper") return true;
		return false;
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./stateDiagram-v2-6OUMAXLB-BkSlpJV7.js");
		return {
			id: id17,
			diagram: diagram2
		};
	}, "loader")
};
var id18 = "journey";
var journeyDetector_default = {
	id: id18,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*journey/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./journeyDiagram-5HDEW3XC-0M6f87JI.js");
		return {
			id: id18,
			diagram: diagram2
		};
	}, "loader")
};
var renderer = { draw: /* @__PURE__ */ __name((_text, id39, version) => {
	log.debug("rendering svg for syntax error\n");
	const svg = selectSvgElement(id39);
	const g = svg.append("g");
	svg.attr("viewBox", "0 0 2412 512");
	configureSvgSize(svg, 100, 512, true);
	g.append("path").attr("class", "error-icon").attr("d", "m411.313,123.313c6.25-6.25 6.25-16.375 0-22.625s-16.375-6.25-22.625,0l-32,32-9.375,9.375-20.688-20.688c-12.484-12.5-32.766-12.5-45.25,0l-16,16c-1.261,1.261-2.304,2.648-3.31,4.051-21.739-8.561-45.324-13.426-70.065-13.426-105.867,0-192,86.133-192,192s86.133,192 192,192 192-86.133 192-192c0-24.741-4.864-48.327-13.426-70.065 1.402-1.007 2.79-2.049 4.051-3.31l16-16c12.5-12.492 12.5-32.758 0-45.25l-20.688-20.688 9.375-9.375 32.001-31.999zm-219.313,100.687c-52.938,0-96,43.063-96,96 0,8.836-7.164,16-16,16s-16-7.164-16-16c0-70.578 57.422-128 128-128 8.836,0 16,7.164 16,16s-7.164,16-16,16z");
	g.append("path").attr("class", "error-icon").attr("d", "m459.02,148.98c-6.25-6.25-16.375-6.25-22.625,0s-6.25,16.375 0,22.625l16,16c3.125,3.125 7.219,4.688 11.313,4.688 4.094,0 8.188-1.563 11.313-4.688 6.25-6.25 6.25-16.375 0-22.625l-16.001-16z");
	g.append("path").attr("class", "error-icon").attr("d", "m340.395,75.605c3.125,3.125 7.219,4.688 11.313,4.688 4.094,0 8.188-1.563 11.313-4.688 6.25-6.25 6.25-16.375 0-22.625l-16-16c-6.25-6.25-16.375-6.25-22.625,0s-6.25,16.375 0,22.625l15.999,16z");
	g.append("path").attr("class", "error-icon").attr("d", "m400,64c8.844,0 16-7.164 16-16v-32c0-8.836-7.156-16-16-16-8.844,0-16,7.164-16,16v32c0,8.836 7.156,16 16,16z");
	g.append("path").attr("class", "error-icon").attr("d", "m496,96.586h-32c-8.844,0-16,7.164-16,16 0,8.836 7.156,16 16,16h32c8.844,0 16-7.164 16-16 0-8.836-7.156-16-16-16z");
	g.append("path").attr("class", "error-icon").attr("d", "m436.98,75.605c3.125,3.125 7.219,4.688 11.313,4.688 4.094,0 8.188-1.563 11.313-4.688l32-32c6.25-6.25 6.25-16.375 0-22.625s-16.375-6.25-22.625,0l-32,32c-6.251,6.25-6.251,16.375-0.001,22.625z");
	g.append("text").attr("class", "error-text").attr("x", 1440).attr("y", 250).attr("font-size", "150px").style("text-anchor", "middle").text("Syntax error in text");
	g.append("text").attr("class", "error-text").attr("x", 1250).attr("y", 400).attr("font-size", "100px").style("text-anchor", "middle").text(`mermaid version ${version}`);
}, "draw") };
var errorRenderer_default = renderer;
var errorDiagram_default = {
	db: {},
	renderer,
	parser: { parse: /* @__PURE__ */ __name(() => {}, "parse") }
};
var id19 = "flowchart-elk";
var detector_default2 = {
	id: id19,
	detector: /* @__PURE__ */ __name((txt, config = {}) => {
		if (/^\s*flowchart-elk/.test(txt) || /^\s*(flowchart|graph)/.test(txt) && config?.flowchart?.defaultRenderer === "elk") {
			config.layout = "elk";
			return true;
		}
		return false;
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./flowDiagram-23GEKE2U-YW6WhgY_.js");
		return {
			id: id19,
			diagram: diagram2
		};
	}, "loader")
};
var id20 = "timeline";
var detector_default3 = {
	id: id20,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*timeline/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./timeline-definition-FHXFAJF6-BLYQW6s4.js");
		return {
			id: id20,
			diagram: diagram2
		};
	}, "loader")
};
var id21 = "mindmap";
var detector_default4 = {
	id: id21,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*mindmap/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./mindmap-definition-LN4V7U3C-BmYES8Th.js");
		return {
			id: id21,
			diagram: diagram2
		};
	}, "loader")
};
var id22 = "kanban";
var detector_default5 = {
	id: id22,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*kanban/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./kanban-definition-HUTT4EX6-F6c9I9-J.js");
		return {
			id: id22,
			diagram: diagram2
		};
	}, "loader")
};
var id23 = "sankey";
var sankeyDetector_default = {
	id: id23,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*sankey(-beta)?/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./sankeyDiagram-HTMAVEWB-hYZB5Mvj.js");
		return {
			id: id23,
			diagram: diagram2
		};
	}, "loader")
};
var id24 = "packet";
var packet = {
	id: id24,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*packet(-beta)?/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./diagram-NH7WQ7WH-C0YB3i0I.js");
		return {
			id: id24,
			diagram: diagram2
		};
	}, "loader")
};
var id25 = "radar";
var radar = {
	id: id25,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*radar-beta/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./diagram-WEI45ONY-Dvw7KPuD.js");
		return {
			id: id25,
			diagram: diagram2
		};
	}, "loader")
};
var id26 = "block";
var blockDetector_default = {
	id: id26,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*block(-beta)?/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./blockDiagram-677ZJIJ3-ChQM-ZIb.js");
		return {
			id: id26,
			diagram: diagram2
		};
	}, "loader")
};
var id27 = "treeView";
var detector_default6 = {
	id: id27,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*treeView-beta/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./diagram-OA4YK3LP-MA_bApww.js");
		return {
			id: id27,
			diagram: diagram2
		};
	}, "loader")
};
var id28 = "architecture";
var architectureDetector_default = {
	id: id28,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*architecture/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./architectureDiagram-ZJ3FMSHR-BOQIEdHW.js");
		return {
			id: id28,
			diagram: diagram2
		};
	}, "loader")
};
var id29 = "eventmodeling";
var detector_default7 = {
	id: id29,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*eventmodeling/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./diagram-FQU43EPY-Dysm4nd_.js");
		return {
			id: id29,
			diagram: diagram2
		};
	}, "loader")
};
var id30 = "ishikawa";
var ishikawa = {
	id: id30,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*ishikawa(-beta)?\b/i.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./ishikawaDiagram-FXEZZL3T-Rxv3afNo.js");
		return {
			id: id30,
			diagram: diagram2
		};
	}, "loader")
};
var id31 = "venn";
var vennDetector_default = {
	id: id31,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*venn-beta/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./vennDiagram-L72KCM5P-orNJJhQQ.js");
		return {
			id: id31,
			diagram: diagram2
		};
	}, "loader")
};
var id32 = "treemap";
var treemap = {
	id: id32,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*treemap/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./diagram-G47NLZAW-DH-Gj6ij.js");
		return {
			id: id32,
			diagram: diagram2
		};
	}, "loader")
};
var id33 = "wardley";
var wardleyDetector_default = {
	id: id33,
	detector: /* @__PURE__ */ __name((text) => {
		return /^\s*wardley-beta/i.test(text);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./wardleyDiagram-EHGQE667-D1tu5Lxt.js");
		return {
			id: id33,
			diagram: diagram2
		};
	}, "loader")
};
var id34 = "cynefin";
var cynefin = {
	id: id34,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*cynefin-beta(?:[\s:]|$)/.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./cynefinDiagram-TSTJHNR4-l0PFS4PQ.js");
		return {
			id: id34,
			diagram: diagram2
		};
	}, "loader")
};
var id35 = "railroad";
var railroad = {
	id: id35,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*railroad-beta/i.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./railroadDiagram-RFXS5EU6-Cc345AnA.js");
		return {
			id: id35,
			diagram: diagram2
		};
	}, "loader")
};
var id36 = "railroadEbnf";
var railroadEbnf = {
	id: id36,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*railroad-ebnf-beta/i.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./ebnfDiagram-CCIWWBDH-mUBtDfwG.js");
		return {
			id: id36,
			diagram: diagram2
		};
	}, "loader")
};
var id37 = "railroadAbnf";
var railroadAbnf = {
	id: id37,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*railroad-abnf-beta/i.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./abnfDiagram-VRR7QNED-ejcdIlfh.js");
		return {
			id: id37,
			diagram: diagram2
		};
	}, "loader")
};
var id38 = "railroadPeg";
var railroadPeg = {
	id: id38,
	detector: /* @__PURE__ */ __name((txt) => {
		return /^\s*railroad-peg-beta/i.test(txt);
	}, "detector"),
	loader: /* @__PURE__ */ __name(async () => {
		const { diagram: diagram2 } = await import("./pegDiagram-2B236MQR-BZ_V6wYU.js");
		return {
			id: id38,
			diagram: diagram2
		};
	}, "loader")
};
var hasLoadedDiagrams = false;
var addDiagrams = /* @__PURE__ */ __name(() => {
	if (hasLoadedDiagrams) return;
	hasLoadedDiagrams = true;
	registerDiagram("error", errorDiagram_default, (text) => {
		return text.toLowerCase().trim() === "error";
	});
	registerDiagram("---", {
		db: { clear: /* @__PURE__ */ __name(() => {}, "clear") },
		styles: {},
		renderer: { draw: /* @__PURE__ */ __name(() => {}, "draw") },
		parser: { parse: /* @__PURE__ */ __name(() => {
			throw new Error("Diagrams beginning with --- are not valid. If you were trying to use a YAML front-matter, please ensure that you've correctly opened and closed the YAML front-matter with un-indented `---` blocks");
		}, "parse") },
		init: /* @__PURE__ */ __name(() => null, "init")
	}, (text) => {
		return text.toLowerCase().trimStart().startsWith("---");
	});
	registerLazyLoadedDiagrams(detector_default2, detector_default4, architectureDetector_default);
	registerLazyLoadedDiagrams(c4Detector_default, detector_default5, classDetector_V2_default, classDetector_default, erDetector_default, ganttDetector_default, info, pie, requirementDetector_default, sequenceDetector_default, detector_default, flowDetector_v2_default, flowDetector_default, detector_default3, gitGraphDetector_default, stateDetector_V2_default, stateDetector_default, journeyDetector_default, quadrantDetector_default, sankeyDetector_default, packet, xychartDetector_default, blockDetector_default, detector_default7, detector_default6, radar, ishikawa, treemap, railroad, railroadEbnf, railroadAbnf, railroadPeg, vennDetector_default, wardleyDetector_default, cynefin);
}, "addDiagrams");
var loadRegisteredDiagrams = /* @__PURE__ */ __name(async () => {
	log.debug(`Loading registered diagrams`);
	const failed = (await Promise.allSettled(Object.entries(detectors).map(async ([key, { detector: detector39, loader: loader39 }]) => {
		if (!loader39) return;
		try {
			getDiagram(key);
		} catch {
			try {
				const { diagram: diagram2, id: id39 } = await loader39();
				registerDiagram(id39, diagram2, detector39);
			} catch (err) {
				log.error(`Failed to load external diagram with key ${key}. Removing from detectors.`);
				delete detectors[key];
				throw err;
			}
		}
	}))).filter((result) => result.status === "rejected");
	if (failed.length > 0) {
		log.error(`Failed to load ${failed.length} external diagrams`);
		for (const res of failed) log.error(res);
		throw new Error(`Failed to load ${failed.length} external diagrams`);
	}
}, "loadRegisteredDiagrams");
var SVG_ROLE = "graphics-document document";
function setA11yDiagramInfo(svg, diagramType) {
	svg.attr("role", SVG_ROLE);
	if (diagramType !== "") svg.attr("aria-roledescription", diagramType);
}
__name(setA11yDiagramInfo, "setA11yDiagramInfo");
function addSVGa11yTitleDescription(svg, a11yTitle, a11yDesc, baseId) {
	if (svg.insert === void 0) return;
	if (a11yDesc) {
		const descId = `chart-desc-${baseId}`;
		svg.attr("aria-describedby", descId);
		svg.insert("desc", ":first-child").attr("id", descId).text(a11yDesc);
	}
	if (a11yTitle) {
		const titleId = `chart-title-${baseId}`;
		svg.attr("aria-labelledby", titleId);
		svg.insert("title", ":first-child").attr("id", titleId).text(a11yTitle);
	}
}
__name(addSVGa11yTitleDescription, "addSVGa11yTitleDescription");
var Diagram = class _Diagram {
	constructor(type, text, db, parser, renderer2) {
		this.type = type;
		this.text = text;
		this.db = db;
		this.parser = parser;
		this.renderer = renderer2;
	}
	static {
		__name(this, "Diagram");
	}
	static async fromText(text, metadata = {}) {
		const config = getConfig();
		const type = detectType(text, config);
		text = encodeEntities(text) + "\n";
		try {
			getDiagram(type);
		} catch {
			const loader39 = getDiagramLoader(type);
			if (!loader39) throw new UnknownDiagramError(`Diagram ${type} not found.`);
			const { id: id39, diagram: diagram2 } = await loader39();
			registerDiagram(id39, diagram2);
		}
		const { db, parser, renderer: renderer2, init: init2 } = getDiagram(type);
		if (parser.parser) parser.parser.yy = db;
		db.clear?.();
		init2?.(config);
		if (metadata.title) db.setDiagramTitle?.(metadata.title);
		await parser.parse(text);
		return new _Diagram(type, text, db, parser, renderer2);
	}
	async render(id39, version) {
		await this.renderer.draw(this.text, id39, version, this);
	}
	getParser() {
		return this.parser;
	}
	getType() {
		return this.type;
	}
};
var interactionFunctions = [];
var attachFunctions = /* @__PURE__ */ __name(() => {
	interactionFunctions.forEach((f) => {
		f();
	});
	interactionFunctions = [];
}, "attachFunctions");
var cleanupComments = /* @__PURE__ */ __name((text) => {
	return text.replace(/^\s*%%(?!{)[^\n]+\n?/gm, "").trimStart();
}, "cleanupComments");
function extractFrontMatter(text) {
	const matches = text.match(frontMatterRegex);
	if (!matches) return {
		text,
		metadata: {}
	};
	const indent = matches[1];
	let parsed = load(indent ? matches[2].split("\n").map((line) => line.startsWith(indent) ? line.slice(indent.length) : line).join("\n") : matches[2], { schema: JSON_SCHEMA }) ?? {};
	parsed = typeof parsed === "object" && !Array.isArray(parsed) ? parsed : {};
	const metadata = {};
	if (parsed.displayMode) metadata.displayMode = parsed.displayMode.toString();
	if (parsed.title) metadata.title = parsed.title.toString();
	if (parsed.config) metadata.config = parsed.config;
	return {
		text: text.slice(matches[0].length),
		metadata
	};
}
__name(extractFrontMatter, "extractFrontMatter");
var cleanupText = /* @__PURE__ */ __name((code) => {
	return code.replace(/\r\n?/g, "\n").replace(/<(\w+)([^>]*)>/g, (match, tag, attributes) => "<" + tag + attributes.replace(/="([^"]*)"/g, "='$1'") + ">");
}, "cleanupText");
var processFrontmatter = /* @__PURE__ */ __name((code) => {
	const { text, metadata } = extractFrontMatter(code);
	const { displayMode, title, config = {} } = metadata;
	if (displayMode) {
		if (!config.gantt) config.gantt = {};
		config.gantt.displayMode = displayMode;
	}
	return {
		title,
		config,
		text
	};
}, "processFrontmatter");
var processDirectives = /* @__PURE__ */ __name((code) => {
	const initDirective = utils_default.detectInit(code) ?? {};
	const wrapDirectives = utils_default.detectDirective(code, "wrap");
	if (Array.isArray(wrapDirectives)) initDirective.wrap = wrapDirectives.some(({ type }) => type === "wrap");
	else if (wrapDirectives?.type === "wrap") initDirective.wrap = true;
	return {
		text: removeDirectives(code),
		directive: initDirective
	};
}, "processDirectives");
function preprocessDiagram(code) {
	const frontMatterResult = processFrontmatter(cleanupText(code));
	const directiveResult = processDirectives(frontMatterResult.text);
	const config = cleanAndMerge(frontMatterResult.config, directiveResult.directive);
	code = cleanupComments(directiveResult.text);
	return {
		code,
		title: frontMatterResult.title,
		config
	};
}
__name(preprocessDiagram, "preprocessDiagram");
function toBase64(str) {
	const utf8Bytes = new TextEncoder().encode(str);
	const utf8Str = Array.from(utf8Bytes, (byte) => String.fromCodePoint(byte)).join("");
	return btoa(utf8Str);
}
__name(toBase64, "toBase64");
var MAX_TEXTLENGTH = 5e4;
var MAX_TEXTLENGTH_EXCEEDED_MSG = "graph TB;a[Maximum text size in diagram exceeded];style a fill:#faa";
var SECURITY_LVL_SANDBOX = "sandbox";
var SECURITY_LVL_LOOSE = "loose";
var XMLNS_SVG_STD = "http://www.w3.org/2000/svg";
var XMLNS_XLINK_STD = "http://www.w3.org/1999/xlink";
var XMLNS_XHTML_STD = "http://www.w3.org/1999/xhtml";
var IFRAME_WIDTH = "100%";
var IFRAME_HEIGHT = "100%";
var IFRAME_STYLES = "border:0;margin:0;";
var IFRAME_BODY_STYLE = "margin:0";
var IFRAME_SANDBOX_OPTS = "allow-top-navigation-by-user-activation allow-popups";
var IFRAME_NOT_SUPPORTED_MSG = "The \"iframe\" tag is not supported by your browser.";
var DOMPURIFY_TAGS = ["foreignobject"];
var DOMPURIFY_ATTR = ["dominant-baseline"];
function processAndSetConfigs(text) {
	const processed = preprocessDiagram(text);
	reset();
	addDirective(processed.config ?? {});
	return processed;
}
__name(processAndSetConfigs, "processAndSetConfigs");
async function parse(text, parseOptions) {
	addDiagrams();
	try {
		const { code, config } = processAndSetConfigs(text);
		return {
			diagramType: (await getDiagramFromText(code)).type,
			config
		};
	} catch (error) {
		if (parseOptions?.suppressErrors) return false;
		throw error;
	}
}
__name(parse, "parse");
var cssImportantStyles = /* @__PURE__ */ __name((cssClass, element, cssClasses = []) => {
	return `.${cssClass} ${element} ${sanitizeCss(`{ ${cssClasses.join(" !important; ")} !important; }`)}`;
}, "cssImportantStyles");
var createCssStyles = /* @__PURE__ */ __name((config, classDefs = /* @__PURE__ */ new Map()) => {
	const cssStyles = new CSSStyleSheet();
	if (config.fontFamily !== void 0) cssStyles.insertRule(`:root { --mermaid-font-family: ${config.fontFamily}}`, cssStyles.cssRules.length);
	if (config.altFontFamily !== void 0) cssStyles.insertRule(`:root { --mermaid-alt-font-family: ${config.altFontFamily}}`, cssStyles.cssRules.length);
	if (classDefs instanceof Map) {
		const cssElements = getEffectiveHtmlLabels(config) ? ["> *", "span"] : [
			"rect",
			"polygon",
			"ellipse",
			"circle",
			"path"
		];
		classDefs.forEach((styleClassDef) => {
			if (!isEmpty(styleClassDef.styles)) cssElements.forEach((cssElement) => {
				cssStyles.insertRule(cssImportantStyles(styleClassDef.id, cssElement, styleClassDef.styles), cssStyles.cssRules.length);
			});
			if (!isEmpty(styleClassDef.textStyles)) cssStyles.insertRule(cssImportantStyles(styleClassDef.id, "tspan", (styleClassDef?.textStyles || []).map((s) => s.replace("color", "fill"))), cssStyles.cssRules.length);
		});
	}
	let cssString = "";
	if (config.themeCSS !== void 0) if (typeof cssStyles.replaceSync === "function") {
		const themeCssStyleSheet = new CSSStyleSheet();
		themeCssStyleSheet.replaceSync(config.themeCSS);
		cssString = cssStyleSheetToString(themeCssStyleSheet) + "\n";
	} else cssString += `${config.themeCSS}
`;
	return cssString + cssStyleSheetToString(cssStyles);
}, "createCssStyles");
var compileCSS = /* @__PURE__ */ __name((namespace, css) => {
	return serialize(compile(`${namespace}{${css}}`), middleware([/* @__PURE__ */ __name(function addNamespace(element, _index, _children, _callback) {
		if (element.type === "rule" && Array.isArray(element.props)) {
			if (element.parent && element.parent.type === "@keyframes") return;
			element.props = element.props.map((prop) => {
				if (!prop.startsWith(namespace)) return `${namespace} ${prop}`;
				return prop;
			});
		} else if (element.type.startsWith("@")) {
			if (![...[
				"@media",
				"@supports",
				"@layer",
				"@scope",
				"@container",
				"@starting-style"
			], "@keyframes"].includes(element.type)) {
				log.warn(`Removing unsupported at-rule ${element.type} from CSS`);
				element.type = COMMENT;
			}
		}
	}, "addNamespace"), stringify]));
}, "compileCSS");
var createUserStyles = /* @__PURE__ */ __name((config, graphType, classDefs, svgId) => {
	return compileCSS(svgId, styles_default(graphType, createCssStyles(config, classDefs), {
		...config.themeVariables,
		theme: config.theme,
		look: config.look
	}, svgId));
}, "createUserStyles");
var cleanUpSvgCode = /* @__PURE__ */ __name((svgCode = "", inSandboxMode, useArrowMarkerUrls) => {
	let cleanedUpSvg = svgCode;
	if (!useArrowMarkerUrls && !inSandboxMode) cleanedUpSvg = cleanedUpSvg.replace(/marker-end="url\([\d+./:=?A-Za-z-]*?#/g, "marker-end=\"url(#");
	cleanedUpSvg = decodeEntities(cleanedUpSvg);
	cleanedUpSvg = cleanedUpSvg.replace(/<br>/g, "<br/>");
	return cleanedUpSvg;
}, "cleanUpSvgCode");
var putIntoIFrame = /* @__PURE__ */ __name((svgCode = "", svgElement) => {
	return `<iframe style="width:${IFRAME_WIDTH};height:${svgElement?.viewBox?.baseVal?.height ? svgElement.viewBox.baseVal.height + "px" : IFRAME_HEIGHT};${IFRAME_STYLES}" src="data:text/html;charset=UTF-8;base64,${toBase64(`<body style="${IFRAME_BODY_STYLE}">${svgCode}</body>`)}" sandbox="${IFRAME_SANDBOX_OPTS}">
  ${IFRAME_NOT_SUPPORTED_MSG}
</iframe>`;
}, "putIntoIFrame");
var appendDivSvgG = /* @__PURE__ */ __name((parentRoot, id39, enclosingDivId, divStyle, svgXlink) => {
	const enclosingDiv = parentRoot.append("div");
	enclosingDiv.attr("id", enclosingDivId);
	if (divStyle) enclosingDiv.attr("style", divStyle);
	const svgNode = enclosingDiv.append("svg").attr("id", id39).attr("width", "100%").attr("xmlns", XMLNS_SVG_STD);
	if (svgXlink) svgNode.attr("xmlns:xlink", svgXlink);
	svgNode.append("g");
	return parentRoot;
}, "appendDivSvgG");
function sandboxedIframe(parentNode, iFrameId) {
	return parentNode.append("iframe").attr("id", iFrameId).attr("style", "width: 100%; height: 100%;").attr("sandbox", "");
}
__name(sandboxedIframe, "sandboxedIframe");
var removeExistingElements = /* @__PURE__ */ __name((doc, id39, divId, iFrameId) => {
	doc.getElementById(id39)?.remove();
	doc.getElementById(divId)?.remove();
	doc.getElementById(iFrameId)?.remove();
}, "removeExistingElements");
var render = /* @__PURE__ */ __name(async function(id39, text, svgContainingElement) {
	addDiagrams();
	const processed = processAndSetConfigs(text);
	text = processed.code;
	const config = getConfig();
	log.debug(config);
	if (text.length > (config?.maxTextSize ?? MAX_TEXTLENGTH)) text = MAX_TEXTLENGTH_EXCEEDED_MSG;
	const idSelector = `#${id39}`;
	const iFrameID = "i" + id39;
	const iFrameID_selector = "#" + iFrameID;
	const enclosingDivID = "d" + id39;
	const enclosingDivID_selector = "#" + enclosingDivID;
	const removeTempElements = /* @__PURE__ */ __name(() => {
		const node = select_default(isSandboxed ? iFrameID_selector : enclosingDivID_selector).node();
		if (node && "remove" in node) node.remove();
	}, "removeTempElements");
	let root = select_default(document.body);
	const isSandboxed = config.securityLevel === SECURITY_LVL_SANDBOX;
	const isLooseSecurityLevel = config.securityLevel === SECURITY_LVL_LOOSE;
	const fontFamily = config.fontFamily;
	if (svgContainingElement !== void 0) {
		if (svgContainingElement) svgContainingElement.innerHTML = "";
		if (isSandboxed) {
			root = select_default(sandboxedIframe(select_default(svgContainingElement), iFrameID).nodes()[0].contentDocument.body);
			root.node().style.margin = "0";
		} else root = select_default(svgContainingElement);
		appendDivSvgG(root, id39, enclosingDivID, `font-family: ${fontFamily}`, XMLNS_XLINK_STD);
	} else {
		removeExistingElements(document, id39, enclosingDivID, iFrameID);
		if (isSandboxed) {
			root = select_default(sandboxedIframe(select_default(document.body), iFrameID).nodes()[0].contentDocument.body);
			root.node().style.margin = "0";
		} else root = select_default("body");
		appendDivSvgG(root, id39, enclosingDivID);
	}
	let diag;
	let parseEncounteredException;
	try {
		diag = await Diagram.fromText(text, { title: processed.title });
	} catch (error) {
		if (config.suppressErrorRendering) {
			removeTempElements();
			throw error;
		}
		diag = await Diagram.fromText("error");
		parseEncounteredException = error;
	}
	const element = root.select(enclosingDivID_selector).node();
	const diagramType = diag.type;
	const svg = element.firstChild;
	const firstChild = svg.firstChild;
	const diagramClassDefs = diag.renderer.getClasses?.(text, diag);
	const rules = createUserStyles(config, diagramType, diagramClassDefs, idSelector);
	const style1 = document.createElement("style");
	style1.innerHTML = rules;
	svg.insertBefore(style1, firstChild);
	try {
		await diag.renderer.draw(text, id39, "11.16.0", diag);
	} catch (e) {
		if (config.suppressErrorRendering) removeTempElements();
		else errorRenderer_default.draw(text, id39, "11.16.0");
		throw e;
	}
	const svgNode = root.select(`${enclosingDivID_selector} svg`);
	const a11yTitle = diag.db.getAccTitle?.();
	const a11yDescr = diag.db.getAccDescription?.();
	addA11yInfo(diagramType, svgNode, a11yTitle, a11yDescr);
	root.select(`[id="${id39}"]`).selectAll("foreignobject > *").attr("xmlns", XMLNS_XHTML_STD);
	let svgCode = root.select(enclosingDivID_selector).node().innerHTML;
	log.debug("config.arrowMarkerAbsolute", config.arrowMarkerAbsolute);
	svgCode = cleanUpSvgCode(svgCode, isSandboxed, evaluate(config.arrowMarkerAbsolute));
	if (isSandboxed) {
		const svgEl = root.select(enclosingDivID_selector + " svg").node();
		svgCode = putIntoIFrame(svgCode, svgEl);
	} else if (!isLooseSecurityLevel) svgCode = purify.sanitize(svgCode, {
		ADD_TAGS: DOMPURIFY_TAGS,
		ADD_ATTR: DOMPURIFY_ATTR,
		HTML_INTEGRATION_POINTS: { foreignobject: true }
	});
	attachFunctions();
	if (parseEncounteredException) throw parseEncounteredException;
	removeTempElements();
	return {
		diagramType,
		svg: svgCode,
		bindFunctions: diag.db.bindFunctions
	};
}, "render");
function initialize(userOptions = {}) {
	const options = assignWithDepth_default({}, userOptions);
	if (options?.fontFamily && !options.themeVariables?.fontFamily) {
		if (!options.themeVariables) options.themeVariables = {};
		options.themeVariables.fontFamily = options.fontFamily;
	}
	saveConfigFromInitialize(options);
	if (options?.theme && options.theme in themes_default) options.themeVariables = themes_default[options.theme].getThemeVariables(options.themeVariables);
	else if (options) options.themeVariables = themes_default.default.getThemeVariables(options.themeVariables);
	setLogLevel((typeof options === "object" ? setSiteConfig(options) : getSiteConfig()).logLevel);
	addDiagrams();
}
__name(initialize, "initialize");
var getDiagramFromText = /* @__PURE__ */ __name((text, metadata = {}) => {
	const { code } = preprocessDiagram(text);
	return Diagram.fromText(code, metadata);
}, "getDiagramFromText");
function addA11yInfo(diagramType, svgNode, a11yTitle, a11yDescr) {
	setA11yDiagramInfo(svgNode, diagramType);
	addSVGa11yTitleDescription(svgNode, a11yTitle, a11yDescr, svgNode.attr("id"));
}
__name(addA11yInfo, "addA11yInfo");
var mermaidAPI = Object.freeze({
	render,
	parse,
	getDiagramFromText,
	initialize,
	getConfig,
	setConfig,
	getSiteConfig,
	updateSiteConfig,
	reset: /* @__PURE__ */ __name(() => {
		reset();
	}, "reset"),
	globalReset: /* @__PURE__ */ __name(() => {
		reset(defaultConfig);
	}, "globalReset"),
	defaultConfig
});
setLogLevel(getConfig().logLevel);
reset(getConfig());
var handleError = /* @__PURE__ */ __name((error, errors, parseError) => {
	log.warn(error);
	if (isDetailedError(error)) {
		if (parseError) parseError(error.str, error.hash);
		errors.push({
			...error,
			message: error.str,
			error
		});
	} else {
		if (parseError) parseError(error);
		if (error instanceof Error) errors.push({
			str: error.message,
			message: error.message,
			hash: error.name,
			error
		});
	}
}, "handleError");
var run = /* @__PURE__ */ __name(async function(options = { querySelector: ".mermaid" }) {
	try {
		await runThrowsErrors(options);
	} catch (e) {
		if (isDetailedError(e)) log.error(e.str);
		if (mermaid.parseError) mermaid.parseError(e);
		if (!options.suppressErrors) {
			log.error("Use the suppressErrors option to suppress these errors");
			throw e;
		}
	}
}, "run");
var runThrowsErrors = /* @__PURE__ */ __name(async function({ postRenderCallback, querySelector, nodes } = { querySelector: ".mermaid" }) {
	const conf = mermaidAPI.getConfig();
	log.debug(`${!postRenderCallback ? "No " : ""}Callback function found`);
	let nodesToProcess;
	if (nodes) nodesToProcess = nodes;
	else if (querySelector) nodesToProcess = document.querySelectorAll(querySelector);
	else throw new Error("Nodes and querySelector are both undefined");
	log.debug(`Found ${nodesToProcess.length} diagrams`);
	if (conf?.startOnLoad !== void 0) {
		log.debug("Start On Load: " + conf?.startOnLoad);
		mermaidAPI.updateSiteConfig({ startOnLoad: conf?.startOnLoad });
	}
	const idGenerator = new utils_default.InitIDGenerator(conf.deterministicIds, conf.deterministicIDSeed);
	let txt;
	const errors = [];
	for (const element of Array.from(nodesToProcess)) {
		log.info("Rendering diagram: " + element.id);
		if (element.getAttribute("data-processed")) continue;
		element.setAttribute("data-processed", "true");
		const id39 = `mermaid-${idGenerator.next()}`;
		txt = element.innerHTML;
		txt = dedent(utils_default.entityDecode(txt)).trim().replace(/<br\s*\/?>/gi, "<br/>");
		const init2 = utils_default.detectInit(txt);
		if (init2) log.debug("Detected early reinit: ", init2);
		try {
			const { svg, bindFunctions } = await render2(id39, txt, element);
			element.innerHTML = svg;
			if (postRenderCallback) await postRenderCallback(id39);
			if (bindFunctions) bindFunctions(element);
		} catch (error) {
			handleError(error, errors, mermaid.parseError);
		}
	}
	if (errors.length > 0) throw errors[0];
}, "runThrowsErrors");
var initialize2 = /* @__PURE__ */ __name(function(config) {
	mermaidAPI.initialize(config);
}, "initialize");
var init = /* @__PURE__ */ __name(async function(config, nodes, callback) {
	log.warn("mermaid.init is deprecated. Please use run instead.");
	if (config) initialize2(config);
	const runOptions = {
		postRenderCallback: callback,
		querySelector: ".mermaid"
	};
	if (typeof nodes === "string") runOptions.querySelector = nodes;
	else if (nodes) if (nodes instanceof HTMLElement) runOptions.nodes = [nodes];
	else runOptions.nodes = nodes;
	await run(runOptions);
}, "init");
var registerExternalDiagrams = /* @__PURE__ */ __name(async (diagrams, { lazyLoad = true } = {}) => {
	addDiagrams();
	registerLazyLoadedDiagrams(...diagrams);
	if (lazyLoad === false) await loadRegisteredDiagrams();
}, "registerExternalDiagrams");
var contentLoaded = /* @__PURE__ */ __name(function() {
	if (mermaid.startOnLoad) {
		const { startOnLoad } = mermaidAPI.getConfig();
		if (startOnLoad) mermaid.run().catch((err) => log.error("Mermaid failed to initialize", err));
	}
}, "contentLoaded");
if (typeof document !== "undefined") window.addEventListener("load", contentLoaded, false);
var setParseErrorHandler = /* @__PURE__ */ __name(function(parseErrorHandler) {
	mermaid.parseError = parseErrorHandler;
}, "setParseErrorHandler");
var executionQueue = [];
var executionQueueRunning = false;
var executeQueue = /* @__PURE__ */ __name(async () => {
	if (executionQueueRunning) return;
	executionQueueRunning = true;
	while (executionQueue.length > 0) {
		const f = executionQueue.shift();
		if (f) try {
			await f();
		} catch (e) {
			log.error("Error executing queue", e);
		}
	}
	executionQueueRunning = false;
}, "executeQueue");
var parse2 = /* @__PURE__ */ __name(async (text, parseOptions) => {
	return new Promise((resolve, reject) => {
		const performCall = /* @__PURE__ */ __name(() => new Promise((res, rej) => {
			mermaidAPI.parse(text, parseOptions).then((r) => {
				res(r);
				resolve(r);
			}, (e) => {
				log.error("Error parsing", e);
				mermaid.parseError?.(e);
				rej(e);
				reject(e);
			});
		}), "performCall");
		executionQueue.push(performCall);
		executeQueue().catch(reject);
	});
}, "parse");
var render2 = /* @__PURE__ */ __name((id39, text, container) => {
	return new Promise((resolve, reject) => {
		const performCall = /* @__PURE__ */ __name(() => new Promise((res, rej) => {
			mermaidAPI.render(id39, text, container).then((r) => {
				res(r);
				resolve(r);
			}, (e) => {
				log.error("Error parsing", e);
				mermaid.parseError?.(e);
				rej(e);
				reject(e);
			});
		}), "performCall");
		executionQueue.push(performCall);
		executeQueue().catch(reject);
	});
}, "render");
var mermaid = {
	startOnLoad: true,
	mermaidAPI,
	parse: parse2,
	render: render2,
	init,
	run,
	registerExternalDiagrams,
	registerLayoutLoaders,
	initialize: initialize2,
	parseError: void 0,
	contentLoaded,
	setParseErrorHandler,
	detectType,
	registerIconPacks,
	getRegisteredDiagramsMetadata: /* @__PURE__ */ __name(() => {
		return Object.keys(detectors).map((id39) => ({ id: id39 }));
	}, "getRegisteredDiagramsMetadata")
};
var mermaid_default = mermaid;
/*! Check if previously processed */
/*!
* Wait for document loaded before starting the execution
*/
//#endregion
export { mermaid_default as default };
