import { at as Color, ot as Utils } from "./chunk-WYO6CB5R-Bo6VAQS5.js";
//#region node_modules/.pnpm/khroma@2.1.0/node_modules/khroma/dist/methods/channel.js
var channel = (color, channel) => {
	return Utils.lang.round(Color.parse(color)[channel]);
};
//#endregion
export { channel as t };
