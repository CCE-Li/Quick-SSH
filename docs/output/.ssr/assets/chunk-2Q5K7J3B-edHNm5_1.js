import { n as __name } from "./chunk-Y2CYZVJY-BEdw2br0.js";
//#region node_modules/.pnpm/mermaid@11.16.0/node_modules/mermaid/dist/chunks/mermaid.core/chunk-2Q5K7J3B.mjs
var ImperativeState = class {
	/**
	* @param init - Function that creates the default state.
	*/
	constructor(init) {
		this.init = init;
		this.records = this.init();
	}
	static {
		__name(this, "ImperativeState");
	}
	reset() {
		this.records = this.init();
	}
};
//#endregion
export { ImperativeState as t };
