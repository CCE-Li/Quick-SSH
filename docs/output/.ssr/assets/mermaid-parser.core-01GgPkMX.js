import { C as createDefaultSharedCoreModule, S as createDefaultCoreModule, _ as RailroadPegGrammarGeneratedModule, a as CynefinGrammarGeneratedModule, b as WardleyGrammarGeneratedModule, c as GitGraphGrammarGeneratedModule, d as PacketGrammarGeneratedModule, f as PieGrammarGeneratedModule, g as RailroadGrammarGeneratedModule, h as RailroadEbnfGrammarGeneratedModule, i as CommonValueConverter, l as InfoGrammarGeneratedModule, m as RailroadAbnfGrammarGeneratedModule, n as AbstractMermaidValueConverter, o as EmptyFileSystem, p as RadarGrammarGeneratedModule, r as ArchitectureGrammarGeneratedModule, s as EventModelingGeneratedModule, t as AbstractMermaidTokenBuilder, u as MermaidGeneratedSharedModule, v as TreeViewGrammarGeneratedModule, w as inject, x as __name, y as TreemapGrammarGeneratedModule } from "./chunk-KEIR6QF5-DMI3r2C0.js";
//#region node_modules/.pnpm/@mermaid-js+parser@1.2.0/node_modules/@mermaid-js/parser/dist/chunks/mermaid-parser.core/chunk-MOZMSUNE.mjs
var ArchitectureTokenBuilder = class extends AbstractMermaidTokenBuilder {
	static {
		__name(this, "ArchitectureTokenBuilder");
	}
	constructor() {
		super(["architecture"]);
	}
};
var ArchitectureValueConverter = class extends AbstractMermaidValueConverter {
	static {
		__name(this, "ArchitectureValueConverter");
	}
	runCustomConverter(rule, input, _cstNode) {
		if (rule.name === "ARCH_ICON") return input.replace(/[()]/g, "").trim();
		else if (rule.name === "ARCH_TEXT_ICON") return input.replace(/["()]/g, "");
		else if (rule.name === "ARCH_TITLE") {
			let result = input.replace(/^\[|]$/g, "").trim();
			if (result.startsWith("\"") && result.endsWith("\"") || result.startsWith("'") && result.endsWith("'")) {
				result = result.slice(1, -1);
				result = result.replace(/\\"/g, "\"").replace(/\\'/g, "'");
			}
			return result.trim();
		}
	}
};
var ArchitectureModule = { parser: {
	TokenBuilder: /* @__PURE__ */ __name(() => new ArchitectureTokenBuilder(), "TokenBuilder"),
	ValueConverter: /* @__PURE__ */ __name(() => new ArchitectureValueConverter(), "ValueConverter")
} };
function createArchitectureServices(context = EmptyFileSystem) {
	const shared = inject(createDefaultSharedCoreModule(context), MermaidGeneratedSharedModule);
	const Architecture = inject(createDefaultCoreModule({ shared }), ArchitectureGrammarGeneratedModule, ArchitectureModule);
	shared.ServiceRegistry.register(Architecture);
	return {
		shared,
		Architecture
	};
}
__name(createArchitectureServices, "createArchitectureServices");
//#endregion
//#region node_modules/.pnpm/@mermaid-js+parser@1.2.0/node_modules/@mermaid-js/parser/dist/chunks/mermaid-parser.core/chunk-OSBZ3O6U.mjs
var CynefinTokenBuilder = class extends AbstractMermaidTokenBuilder {
	static {
		__name(this, "CynefinTokenBuilder");
	}
	constructor() {
		super(["cynefin-beta"]);
	}
};
var CynefinModule = { parser: {
	TokenBuilder: /* @__PURE__ */ __name(() => new CynefinTokenBuilder(), "TokenBuilder"),
	ValueConverter: /* @__PURE__ */ __name(() => new CommonValueConverter(), "ValueConverter")
} };
function createCynefinServices(context = EmptyFileSystem) {
	const shared = inject(createDefaultSharedCoreModule(context), MermaidGeneratedSharedModule);
	const Cynefin = inject(createDefaultCoreModule({ shared }), CynefinGrammarGeneratedModule, CynefinModule);
	shared.ServiceRegistry.register(Cynefin);
	return {
		shared,
		Cynefin
	};
}
__name(createCynefinServices, "createCynefinServices");
//#endregion
//#region node_modules/.pnpm/@mermaid-js+parser@1.2.0/node_modules/@mermaid-js/parser/dist/chunks/mermaid-parser.core/chunk-5JV3BV7I.mjs
var EventModelingTokenBuilder = class extends AbstractMermaidTokenBuilder {
	static {
		__name(this, "EventModelingTokenBuilder");
	}
	constructor() {
		super(["eventmodeling"]);
	}
};
var COMMAND_TYPES = /* @__PURE__ */ new Set(["cmd", "command"]);
var EVENT_TYPES = /* @__PURE__ */ new Set(["evt", "event"]);
var READMODEL_TYPES = /* @__PURE__ */ new Set(["rmo", "readmodel"]);
var PROCESSOR_TYPES = /* @__PURE__ */ new Set(["pcr", "processor"]);
var UI_TYPES = /* @__PURE__ */ new Set(["ui"]);
function registerValidationChecks$1(services) {
	const validator = services.validation.EventModelingValidator;
	const registry = services.validation.ValidationRegistry;
	if (registry) {
		const checks = {
			EmTimeFrame: validator.checkSourceFrameTypes.bind(validator),
			EmResetFrame: validator.checkSourceFrameTypes.bind(validator)
		};
		registry.register(checks, validator);
	}
}
__name(registerValidationChecks$1, "registerValidationChecks");
var EventModelingValidator = class {
	static {
		__name(this, "EventModelingValidator");
	}
	checkSourceFrameTypes(frame, accept) {
		if (frame.sourceFrames.length === 0) return;
		if (COMMAND_TYPES.has(frame.modelEntityType)) this.validateSources(frame, /* @__PURE__ */ new Set([...UI_TYPES, ...PROCESSOR_TYPES]), "command", "ui or processor", accept);
		else if (EVENT_TYPES.has(frame.modelEntityType)) this.validateSources(frame, COMMAND_TYPES, "event", "command", accept);
		else if (READMODEL_TYPES.has(frame.modelEntityType)) this.validateSources(frame, EVENT_TYPES, "read model", "event", accept);
		else if (PROCESSOR_TYPES.has(frame.modelEntityType)) this.validateSources(frame, READMODEL_TYPES, "processor", "read model", accept);
		else if (UI_TYPES.has(frame.modelEntityType)) this.validateSources(frame, READMODEL_TYPES, "ui", "read model", accept);
	}
	validateSources(frame, allowedSourceTypes, targetLabel, expectedSourceLabel, accept) {
		for (const sourceRef of frame.sourceFrames) {
			const source = sourceRef.ref;
			if (source !== void 0 && !allowedSourceTypes.has(source.modelEntityType)) accept("error", `A ${targetLabel} can only receive input from a ${expectedSourceLabel}, not from '${source.modelEntityType}'.`, {
				node: frame,
				property: "sourceFrames"
			});
		}
	}
};
var EventModelingModule = {
	parser: {
		TokenBuilder: /* @__PURE__ */ __name(() => new EventModelingTokenBuilder(), "TokenBuilder"),
		ValueConverter: /* @__PURE__ */ __name(() => new CommonValueConverter(), "ValueConverter")
	},
	validation: { EventModelingValidator: /* @__PURE__ */ __name(() => new EventModelingValidator(), "EventModelingValidator") }
};
function createEventModelingServices(context = EmptyFileSystem) {
	const shared = inject(createDefaultSharedCoreModule(context), MermaidGeneratedSharedModule);
	const EventModel = inject(createDefaultCoreModule({ shared }), EventModelingGeneratedModule, EventModelingModule);
	shared.ServiceRegistry.register(EventModel);
	registerValidationChecks$1(EventModel);
	return {
		shared,
		EventModel
	};
}
__name(createEventModelingServices, "createEventModelingServices");
//#endregion
//#region node_modules/.pnpm/@mermaid-js+parser@1.2.0/node_modules/@mermaid-js/parser/dist/chunks/mermaid-parser.core/chunk-CYSBUYHQ.mjs
var GitGraphTokenBuilder = class extends AbstractMermaidTokenBuilder {
	static {
		__name(this, "GitGraphTokenBuilder");
	}
	constructor() {
		super(["gitGraph"]);
	}
};
var GitGraphModule = { parser: {
	TokenBuilder: /* @__PURE__ */ __name(() => new GitGraphTokenBuilder(), "TokenBuilder"),
	ValueConverter: /* @__PURE__ */ __name(() => new CommonValueConverter(), "ValueConverter")
} };
function createGitGraphServices(context = EmptyFileSystem) {
	const shared = inject(createDefaultSharedCoreModule(context), MermaidGeneratedSharedModule);
	const GitGraph = inject(createDefaultCoreModule({ shared }), GitGraphGrammarGeneratedModule, GitGraphModule);
	shared.ServiceRegistry.register(GitGraph);
	return {
		shared,
		GitGraph
	};
}
__name(createGitGraphServices, "createGitGraphServices");
//#endregion
//#region node_modules/.pnpm/@mermaid-js+parser@1.2.0/node_modules/@mermaid-js/parser/dist/chunks/mermaid-parser.core/chunk-BIQX33UG.mjs
var InfoTokenBuilder = class extends AbstractMermaidTokenBuilder {
	static {
		__name(this, "InfoTokenBuilder");
	}
	constructor() {
		super(["info", "showInfo"]);
	}
};
var InfoModule = { parser: {
	TokenBuilder: /* @__PURE__ */ __name(() => new InfoTokenBuilder(), "TokenBuilder"),
	ValueConverter: /* @__PURE__ */ __name(() => new CommonValueConverter(), "ValueConverter")
} };
function createInfoServices(context = EmptyFileSystem) {
	const shared = inject(createDefaultSharedCoreModule(context), MermaidGeneratedSharedModule);
	const Info = inject(createDefaultCoreModule({ shared }), InfoGrammarGeneratedModule, InfoModule);
	shared.ServiceRegistry.register(Info);
	return {
		shared,
		Info
	};
}
__name(createInfoServices, "createInfoServices");
//#endregion
//#region node_modules/.pnpm/@mermaid-js+parser@1.2.0/node_modules/@mermaid-js/parser/dist/chunks/mermaid-parser.core/chunk-EMLP6XTP.mjs
var PacketTokenBuilder = class extends AbstractMermaidTokenBuilder {
	static {
		__name(this, "PacketTokenBuilder");
	}
	constructor() {
		super(["packet"]);
	}
};
var PacketModule = { parser: {
	TokenBuilder: /* @__PURE__ */ __name(() => new PacketTokenBuilder(), "TokenBuilder"),
	ValueConverter: /* @__PURE__ */ __name(() => new CommonValueConverter(), "ValueConverter")
} };
function createPacketServices(context = EmptyFileSystem) {
	const shared = inject(createDefaultSharedCoreModule(context), MermaidGeneratedSharedModule);
	const Packet = inject(createDefaultCoreModule({ shared }), PacketGrammarGeneratedModule, PacketModule);
	shared.ServiceRegistry.register(Packet);
	return {
		shared,
		Packet
	};
}
__name(createPacketServices, "createPacketServices");
//#endregion
//#region node_modules/.pnpm/@mermaid-js+parser@1.2.0/node_modules/@mermaid-js/parser/dist/chunks/mermaid-parser.core/chunk-YOTPTUD7.mjs
var PieTokenBuilder = class extends AbstractMermaidTokenBuilder {
	static {
		__name(this, "PieTokenBuilder");
	}
	constructor() {
		super(["pie", "showData"]);
	}
};
var PieValueConverter = class extends AbstractMermaidValueConverter {
	static {
		__name(this, "PieValueConverter");
	}
	runCustomConverter(rule, input, _cstNode) {
		if (rule.name !== "PIE_SECTION_LABEL") return;
		return input.replace(/"/g, "").trim();
	}
};
var PieModule = { parser: {
	TokenBuilder: /* @__PURE__ */ __name(() => new PieTokenBuilder(), "TokenBuilder"),
	ValueConverter: /* @__PURE__ */ __name(() => new PieValueConverter(), "ValueConverter")
} };
function createPieServices(context = EmptyFileSystem) {
	const shared = inject(createDefaultSharedCoreModule(context), MermaidGeneratedSharedModule);
	const Pie = inject(createDefaultCoreModule({ shared }), PieGrammarGeneratedModule, PieModule);
	shared.ServiceRegistry.register(Pie);
	return {
		shared,
		Pie
	};
}
__name(createPieServices, "createPieServices");
//#endregion
//#region node_modules/.pnpm/@mermaid-js+parser@1.2.0/node_modules/@mermaid-js/parser/dist/chunks/mermaid-parser.core/chunk-QBLGF6JB.mjs
var RadarTokenBuilder = class extends AbstractMermaidTokenBuilder {
	static {
		__name(this, "RadarTokenBuilder");
	}
	constructor() {
		super(["radar-beta"]);
	}
};
var RadarModule = { parser: {
	TokenBuilder: /* @__PURE__ */ __name(() => new RadarTokenBuilder(), "TokenBuilder"),
	ValueConverter: /* @__PURE__ */ __name(() => new CommonValueConverter(), "ValueConverter")
} };
function createRadarServices(context = EmptyFileSystem) {
	const shared = inject(createDefaultSharedCoreModule(context), MermaidGeneratedSharedModule);
	const Radar = inject(createDefaultCoreModule({ shared }), RadarGrammarGeneratedModule, RadarModule);
	shared.ServiceRegistry.register(Radar);
	return {
		shared,
		Radar
	};
}
__name(createRadarServices, "createRadarServices");
//#endregion
//#region node_modules/.pnpm/@mermaid-js+parser@1.2.0/node_modules/@mermaid-js/parser/dist/chunks/mermaid-parser.core/chunk-5TONJI2A.mjs
var RailroadTokenBuilder = class extends AbstractMermaidTokenBuilder {
	static {
		__name(this, "RailroadTokenBuilder");
	}
	constructor() {
		super(["railroad-beta"]);
	}
};
var decodeEscapedString$2 = /* @__PURE__ */ __name((input) => {
	const content = input.slice(1, -1);
	let value = "";
	for (let index = 0; index < content.length; index++) {
		const character = content[index];
		if (character === "\\" && index + 1 < content.length) {
			index++;
			const escaped = content[index];
			switch (escaped) {
				case "n":
					value += "\n";
					break;
				case "r":
					value += "\r";
					break;
				case "t":
					value += "	";
					break;
				default: value += escaped;
			}
			continue;
		}
		value += character;
	}
	return value;
}, "decodeEscapedString");
var RailroadValueConverter = class extends AbstractMermaidValueConverter {
	static {
		__name(this, "RailroadValueConverter");
	}
	runConverter(rule, input, cstNode) {
		const value = super.runConverter(rule, input, cstNode);
		if (rule.name === "TITLE" && typeof value === "string") {
			const trimmedValue = value.trim();
			if (trimmedValue.startsWith("\"") && trimmedValue.endsWith("\"") || trimmedValue.startsWith("'") && trimmedValue.endsWith("'")) return decodeEscapedString$2(trimmedValue);
		}
		return value;
	}
	runCustomConverter(rule, input, _cstNode) {
		if (rule.name === "RR_STRING") return decodeEscapedString$2(input);
	}
};
var RailroadModule = { parser: {
	TokenBuilder: /* @__PURE__ */ __name(() => new RailroadTokenBuilder(), "TokenBuilder"),
	ValueConverter: /* @__PURE__ */ __name(() => new RailroadValueConverter(), "ValueConverter")
} };
function createRailroadServices(context = EmptyFileSystem) {
	const shared = inject(createDefaultSharedCoreModule(context), MermaidGeneratedSharedModule);
	const Railroad = inject(createDefaultCoreModule({ shared }), RailroadGrammarGeneratedModule, RailroadModule);
	shared.ServiceRegistry.register(Railroad);
	return {
		shared,
		Railroad
	};
}
__name(createRailroadServices, "createRailroadServices");
//#endregion
//#region node_modules/.pnpm/@mermaid-js+parser@1.2.0/node_modules/@mermaid-js/parser/dist/chunks/mermaid-parser.core/chunk-5HE753X5.mjs
var RailroadAbnfTokenBuilder = class extends AbstractMermaidTokenBuilder {
	static {
		__name(this, "RailroadAbnfTokenBuilder");
	}
	constructor() {
		super(["railroad-abnf-beta"]);
	}
};
var RailroadAbnfValueConverter = class extends AbstractMermaidValueConverter {
	static {
		__name(this, "RailroadAbnfValueConverter");
	}
	runConverter(rule, input, cstNode) {
		const value = super.runConverter(rule, input, cstNode);
		if (rule.name === "TITLE" && typeof value === "string") {
			const trimmedValue = value.trim();
			if (trimmedValue.startsWith("\"") && trimmedValue.endsWith("\"") || trimmedValue.startsWith("'") && trimmedValue.endsWith("'")) return trimmedValue.slice(1, -1);
		}
		return value;
	}
	runCustomConverter(rule, input, _cstNode) {
		if (rule.name === "ABNF_STRING") return input.slice(1, -1);
	}
};
var RailroadAbnfModule = { parser: {
	TokenBuilder: /* @__PURE__ */ __name(() => new RailroadAbnfTokenBuilder(), "TokenBuilder"),
	ValueConverter: /* @__PURE__ */ __name(() => new RailroadAbnfValueConverter(), "ValueConverter")
} };
function createRailroadAbnfServices(context = EmptyFileSystem) {
	const shared = inject(createDefaultSharedCoreModule(context), MermaidGeneratedSharedModule);
	const RailroadAbnf = inject(createDefaultCoreModule({ shared }), RailroadAbnfGrammarGeneratedModule, RailroadAbnfModule);
	shared.ServiceRegistry.register(RailroadAbnf);
	return {
		shared,
		RailroadAbnf
	};
}
__name(createRailroadAbnfServices, "createRailroadAbnfServices");
//#endregion
//#region node_modules/.pnpm/@mermaid-js+parser@1.2.0/node_modules/@mermaid-js/parser/dist/chunks/mermaid-parser.core/chunk-U6XO7XAA.mjs
var RailroadEbnfTokenBuilder = class extends AbstractMermaidTokenBuilder {
	static {
		__name(this, "RailroadEbnfTokenBuilder");
	}
	constructor() {
		super(["railroad-ebnf-beta"]);
	}
};
var decodeEscapedString$1 = /* @__PURE__ */ __name((input) => {
	const content = input.slice(1, -1);
	let value = "";
	for (let index = 0; index < content.length; index++) {
		const character = content[index];
		if (character === "\\" && index + 1 < content.length) {
			index++;
			const escaped = content[index];
			switch (escaped) {
				case "n":
					value += "\n";
					break;
				case "r":
					value += "\r";
					break;
				case "t":
					value += "	";
					break;
				default: value += escaped;
			}
			continue;
		}
		value += character;
	}
	return value;
}, "decodeEscapedString");
var RailroadEbnfValueConverter = class extends AbstractMermaidValueConverter {
	static {
		__name(this, "RailroadEbnfValueConverter");
	}
	runConverter(rule, input, cstNode) {
		const value = super.runConverter(rule, input, cstNode);
		if (rule.name === "TITLE" && typeof value === "string") {
			const trimmedValue = value.trim();
			if (trimmedValue.startsWith("\"") && trimmedValue.endsWith("\"") || trimmedValue.startsWith("'") && trimmedValue.endsWith("'")) return decodeEscapedString$1(trimmedValue);
		}
		return value;
	}
	runCustomConverter(rule, input, _cstNode) {
		if (rule.name === "EBNF_STRING") return decodeEscapedString$1(input);
		if (rule.name === "EBNF_SPECIAL_SEQUENCE") return input.slice(1, -1).trim();
	}
};
var RailroadEbnfModule = { parser: {
	TokenBuilder: /* @__PURE__ */ __name(() => new RailroadEbnfTokenBuilder(), "TokenBuilder"),
	ValueConverter: /* @__PURE__ */ __name(() => new RailroadEbnfValueConverter(), "ValueConverter")
} };
function createRailroadEbnfServices(context = EmptyFileSystem) {
	const shared = inject(createDefaultSharedCoreModule(context), MermaidGeneratedSharedModule);
	const RailroadEbnf = inject(createDefaultCoreModule({ shared }), RailroadEbnfGrammarGeneratedModule, RailroadEbnfModule);
	shared.ServiceRegistry.register(RailroadEbnf);
	return {
		shared,
		RailroadEbnf
	};
}
__name(createRailroadEbnfServices, "createRailroadEbnfServices");
//#endregion
//#region node_modules/.pnpm/@mermaid-js+parser@1.2.0/node_modules/@mermaid-js/parser/dist/chunks/mermaid-parser.core/chunk-JG7HCLWE.mjs
var RailroadPegTokenBuilder = class extends AbstractMermaidTokenBuilder {
	static {
		__name(this, "RailroadPegTokenBuilder");
	}
	constructor() {
		super(["railroad-peg-beta"]);
	}
};
var decodeEscapedString = /* @__PURE__ */ __name((input) => {
	const content = input.slice(1, -1);
	let value = "";
	for (let index = 0; index < content.length; index++) {
		const character = content[index];
		if (character === "\\" && index + 1 < content.length) {
			index++;
			const escaped = content[index];
			switch (escaped) {
				case "n":
					value += "\n";
					break;
				case "r":
					value += "\r";
					break;
				case "t":
					value += "	";
					break;
				default: value += escaped;
			}
			continue;
		}
		value += character;
	}
	return value;
}, "decodeEscapedString");
var RailroadPegValueConverter = class extends AbstractMermaidValueConverter {
	static {
		__name(this, "RailroadPegValueConverter");
	}
	runConverter(rule, input, cstNode) {
		const value = super.runConverter(rule, input, cstNode);
		if (rule.name === "TITLE" && typeof value === "string") {
			const trimmedValue = value.trim();
			if (trimmedValue.startsWith("\"") && trimmedValue.endsWith("\"") || trimmedValue.startsWith("'") && trimmedValue.endsWith("'")) return decodeEscapedString(trimmedValue);
		}
		return value;
	}
	runCustomConverter(rule, input, _cstNode) {
		if (rule.name === "PEG_STRING") return decodeEscapedString(input);
	}
};
var RailroadPegModule = { parser: {
	TokenBuilder: /* @__PURE__ */ __name(() => new RailroadPegTokenBuilder(), "TokenBuilder"),
	ValueConverter: /* @__PURE__ */ __name(() => new RailroadPegValueConverter(), "ValueConverter")
} };
function createRailroadPegServices(context = EmptyFileSystem) {
	const shared = inject(createDefaultSharedCoreModule(context), MermaidGeneratedSharedModule);
	const RailroadPeg = inject(createDefaultCoreModule({ shared }), RailroadPegGrammarGeneratedModule, RailroadPegModule);
	shared.ServiceRegistry.register(RailroadPeg);
	return {
		shared,
		RailroadPeg
	};
}
__name(createRailroadPegServices, "createRailroadPegServices");
//#endregion
//#region node_modules/.pnpm/@mermaid-js+parser@1.2.0/node_modules/@mermaid-js/parser/dist/chunks/mermaid-parser.core/chunk-CQNSW5MT.mjs
var TreeViewValueConverter = class extends AbstractMermaidValueConverter {
	static {
		__name(this, "TreeViewValueConverter");
	}
	runCustomConverter(rule, input, _cstNode) {
		if (rule.name === "INDENTATION") return input?.length || 0;
		if (rule.name === "QUOTED_NAME") return input.substring(1, input.length - 1);
		if (rule.name === "BARE_NAME") return input.replace(/[\t ]+$/, "");
		if (rule.name === "CLASS_ANNOTATION") return input.trim().substring(3).trim();
		if (rule.name === "ICON_ANNOTATION") {
			const trimmed = input.trim();
			return trimmed.substring(5, trimmed.length - 1);
		}
		if (rule.name === "DESC_ANNOTATION") return input.trim().substring(2).trim();
	}
};
var TreeViewTokenBuilder = class extends AbstractMermaidTokenBuilder {
	static {
		__name(this, "TreeViewTokenBuilder");
	}
	constructor() {
		super(["treeView-beta"]);
	}
};
var TreeViewModule = { parser: {
	TokenBuilder: /* @__PURE__ */ __name(() => new TreeViewTokenBuilder(), "TokenBuilder"),
	ValueConverter: /* @__PURE__ */ __name(() => new TreeViewValueConverter(), "ValueConverter")
} };
function createTreeViewServices(context = EmptyFileSystem) {
	const shared = inject(createDefaultSharedCoreModule(context), MermaidGeneratedSharedModule);
	const TreeView = inject(createDefaultCoreModule({ shared }), TreeViewGrammarGeneratedModule, TreeViewModule);
	shared.ServiceRegistry.register(TreeView);
	return {
		shared,
		TreeView
	};
}
__name(createTreeViewServices, "createTreeViewServices");
//#endregion
//#region node_modules/.pnpm/@mermaid-js+parser@1.2.0/node_modules/@mermaid-js/parser/dist/chunks/mermaid-parser.core/chunk-R7FJI6CG.mjs
var TreemapTokenBuilder = class extends AbstractMermaidTokenBuilder {
	static {
		__name(this, "TreemapTokenBuilder");
	}
	constructor() {
		super(["treemap"]);
	}
};
var classDefRegex = /classDef\s+([A-Z_a-z]\w+)(?:\s+([^\n\r;]*))?;?/;
var TreemapValueConverter = class extends AbstractMermaidValueConverter {
	static {
		__name(this, "TreemapValueConverter");
	}
	runCustomConverter(rule, input, _cstNode) {
		if (rule.name === "NUMBER2") return parseFloat(input.replace(/,/g, ""));
		else if (rule.name === "SEPARATOR") return input.substring(1, input.length - 1);
		else if (rule.name === "STRING2") return input.substring(1, input.length - 1);
		else if (rule.name === "INDENTATION") return input.length;
		else if (rule.name === "ClassDef") {
			if (typeof input !== "string") return input;
			const match = classDefRegex.exec(input);
			if (match) return {
				$type: "ClassDefStatement",
				className: match[1],
				styleText: match[2] || void 0
			};
		}
	}
};
function registerValidationChecks(services) {
	const validator = services.validation.TreemapValidator;
	const registry = services.validation.ValidationRegistry;
	if (registry) {
		const checks = { Treemap: validator.checkSingleRoot.bind(validator) };
		registry.register(checks, validator);
	}
}
__name(registerValidationChecks, "registerValidationChecks");
var TreemapValidator = class {
	static {
		__name(this, "TreemapValidator");
	}
	/**
	* Validates that a treemap has only one root node.
	* A root node is defined as a node that has no indentation.
	*/
	checkSingleRoot(doc, accept) {
		let rootNodeIndentation;
		for (const row of doc.TreemapRows) {
			if (!row.item) continue;
			if (rootNodeIndentation === void 0 && row.indent === void 0) rootNodeIndentation = 0;
			else if (row.indent === void 0) accept("error", "Multiple root nodes are not allowed in a treemap.", {
				node: row,
				property: "item"
			});
			else if (rootNodeIndentation !== void 0 && rootNodeIndentation >= parseInt(row.indent, 10)) accept("error", "Multiple root nodes are not allowed in a treemap.", {
				node: row,
				property: "item"
			});
		}
	}
};
var TreemapModule = {
	parser: {
		TokenBuilder: /* @__PURE__ */ __name(() => new TreemapTokenBuilder(), "TokenBuilder"),
		ValueConverter: /* @__PURE__ */ __name(() => new TreemapValueConverter(), "ValueConverter")
	},
	validation: { TreemapValidator: /* @__PURE__ */ __name(() => new TreemapValidator(), "TreemapValidator") }
};
function createTreemapServices(context = EmptyFileSystem) {
	const shared = inject(createDefaultSharedCoreModule(context), MermaidGeneratedSharedModule);
	const Treemap = inject(createDefaultCoreModule({ shared }), TreemapGrammarGeneratedModule, TreemapModule);
	shared.ServiceRegistry.register(Treemap);
	registerValidationChecks(Treemap);
	return {
		shared,
		Treemap
	};
}
__name(createTreemapServices, "createTreemapServices");
//#endregion
//#region node_modules/.pnpm/@mermaid-js+parser@1.2.0/node_modules/@mermaid-js/parser/dist/chunks/mermaid-parser.core/chunk-5FCAYU7R.mjs
var WardleyValueConverter = class extends AbstractMermaidValueConverter {
	static {
		__name(this, "WardleyValueConverter");
	}
	runCustomConverter(rule, input, _cstNode) {
		switch (rule.name.toUpperCase()) {
			case "LINK_LABEL": return input.substring(1).trim();
			default: return;
		}
	}
};
var WardleyModule = { parser: { ValueConverter: /* @__PURE__ */ __name(() => new WardleyValueConverter(), "ValueConverter") } };
function createWardleyServices(context = EmptyFileSystem) {
	const shared = inject(createDefaultSharedCoreModule(context), MermaidGeneratedSharedModule);
	const Wardley = inject(createDefaultCoreModule({ shared }), WardleyGrammarGeneratedModule, WardleyModule);
	shared.ServiceRegistry.register(Wardley);
	return {
		shared,
		Wardley
	};
}
__name(createWardleyServices, "createWardleyServices");
//#endregion
//#region node_modules/.pnpm/@mermaid-js+parser@1.2.0/node_modules/@mermaid-js/parser/dist/mermaid-parser.core.mjs
var parsers = {};
var initializers = {
	info: /* @__PURE__ */ __name(async () => {
		const { createInfoServices: createInfoServices2 } = await import("./info-DKCQHKI2-gwY2rW1C.js");
		parsers.info = createInfoServices2().Info.parser.LangiumParser;
	}, "info"),
	packet: /* @__PURE__ */ __name(async () => {
		const { createPacketServices: createPacketServices2 } = await import("./packet-7NZHBO7P-Ba-2CU8i.js");
		parsers.packet = createPacketServices2().Packet.parser.LangiumParser;
	}, "packet"),
	pie: /* @__PURE__ */ __name(async () => {
		const { createPieServices: createPieServices2 } = await import("./pie-RZYD4A2V-C_s6dmnZ.js");
		parsers.pie = createPieServices2().Pie.parser.LangiumParser;
	}, "pie"),
	treeView: /* @__PURE__ */ __name(async () => {
		const { createTreeViewServices: createTreeViewServices2 } = await import("./treeView-QDETBFTQ-BT-x5MXw.js");
		parsers.treeView = createTreeViewServices2().TreeView.parser.LangiumParser;
	}, "treeView"),
	architecture: /* @__PURE__ */ __name(async () => {
		const { createArchitectureServices: createArchitectureServices2 } = await import("./architecture-TIHT7OUA-C5O2PX-b.js");
		parsers.architecture = createArchitectureServices2().Architecture.parser.LangiumParser;
	}, "architecture"),
	gitGraph: /* @__PURE__ */ __name(async () => {
		const { createGitGraphServices: createGitGraphServices2 } = await import("./gitGraph-TEB2WS4Q-DWDFTTmC.js");
		parsers.gitGraph = createGitGraphServices2().GitGraph.parser.LangiumParser;
	}, "gitGraph"),
	eventmodeling: /* @__PURE__ */ __name(async () => {
		const { createEventModelingServices: createEventModelingServices2 } = await import("./eventmodeling-45OFAUF4-PJd4irlj.js");
		parsers.eventmodeling = createEventModelingServices2().EventModel.parser.LangiumParser;
	}, "eventmodeling"),
	radar: /* @__PURE__ */ __name(async () => {
		const { createRadarServices: createRadarServices2 } = await import("./radar-I7S5WNFK-npOhrWu1.js");
		parsers.radar = createRadarServices2().Radar.parser.LangiumParser;
	}, "radar"),
	railroad: /* @__PURE__ */ __name(async () => {
		const { createRailroadServices: createRailroadServices2 } = await import("./railroad-3IZDKUUU-BUP9GLpb.js");
		parsers.railroad = createRailroadServices2().Railroad.parser.LangiumParser;
	}, "railroad"),
	railroadEbnf: /* @__PURE__ */ __name(async () => {
		const { createRailroadEbnfServices: createRailroadEbnfServices2 } = await import("./railroad-ebnf-EBAXGLYW-p5yvXiKm.js");
		parsers.railroadEbnf = createRailroadEbnfServices2().RailroadEbnf.parser.LangiumParser;
	}, "railroadEbnf"),
	railroadAbnf: /* @__PURE__ */ __name(async () => {
		const { createRailroadAbnfServices: createRailroadAbnfServices2 } = await import("./railroad-abnf-AHOZXSZD-Yq_ayqo3.js");
		parsers.railroadAbnf = createRailroadAbnfServices2().RailroadAbnf.parser.LangiumParser;
	}, "railroadAbnf"),
	railroadPeg: /* @__PURE__ */ __name(async () => {
		const { createRailroadPegServices: createRailroadPegServices2 } = await import("./railroad-peg-LSFZ7HO6-CQUgCsqb.js");
		parsers.railroadPeg = createRailroadPegServices2().RailroadPeg.parser.LangiumParser;
	}, "railroadPeg"),
	treemap: /* @__PURE__ */ __name(async () => {
		const { createTreemapServices: createTreemapServices2 } = await import("./treemap-6X3UGDF4-W3o5QO3n.js");
		parsers.treemap = createTreemapServices2().Treemap.parser.LangiumParser;
	}, "treemap"),
	wardley: /* @__PURE__ */ __name(async () => {
		const { createWardleyServices: createWardleyServices2 } = await import("./wardley-OPB4EBWU-v11AFUnI.js");
		parsers.wardley = createWardleyServices2().Wardley.parser.LangiumParser;
	}, "wardley"),
	cynefin: /* @__PURE__ */ __name(async () => {
		const { createCynefinServices: createCynefinServices2 } = await import("./cynefin-VYW2F7L2-0xajPkPs.js");
		parsers.cynefin = createCynefinServices2().Cynefin.parser.LangiumParser;
	}, "cynefin")
};
async function parse(diagramType, text) {
	const initializer = initializers[diagramType];
	if (!initializer) throw new Error(`Unknown diagram type: ${diagramType}`);
	if (!parsers[diagramType]) await initializer();
	const result = parsers[diagramType].parse(text);
	if (result.lexerErrors.length > 0 || result.parserErrors.length > 0) throw new MermaidParseError(result);
	return result.value;
}
__name(parse, "parse");
var MermaidParseError = class extends Error {
	constructor(result) {
		const lexerErrors = result.lexerErrors.map((err) => {
			return `Lexer error on line ${err.line !== void 0 && !isNaN(err.line) ? err.line : "?"}, column ${err.column !== void 0 && !isNaN(err.column) ? err.column : "?"}: ${err.message}`;
		}).join("\n");
		const parserErrors = result.parserErrors.map((err) => {
			return `Parse error on line ${err.token.startLine !== void 0 && !isNaN(err.token.startLine) ? err.token.startLine : "?"}, column ${err.token.startColumn !== void 0 && !isNaN(err.token.startColumn) ? err.token.startColumn : "?"}: ${err.message}`;
		}).join("\n");
		super(`Parsing failed: ${lexerErrors} ${parserErrors}`);
		this.result = result;
	}
	static {
		__name(this, "MermaidParseError");
	}
};
//#endregion
export { createCynefinServices as A, InfoModule as C, EventModelingModule as D, createGitGraphServices as E, createArchitectureServices as M, createEventModelingServices as O, createPacketServices as S, GitGraphModule as T, RadarModule as _, TreemapModule as a, createPieServices as b, createTreeViewServices as c, RailroadEbnfModule as d, createRailroadEbnfServices as f, createRailroadServices as g, RailroadModule as h, createWardleyServices as i, ArchitectureModule as j, CynefinModule as k, RailroadPegModule as l, createRailroadAbnfServices as m, parse as n, createTreemapServices as o, RailroadAbnfModule as p, WardleyModule as r, TreeViewModule as s, MermaidParseError as t, createRailroadPegServices as u, createRadarServices as v, createInfoServices as w, PacketModule as x, PieModule as y };
