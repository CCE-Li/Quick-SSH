import { o as __toESM } from "./rolldown-runtime-CE-6LUnI.js";
import { a as Search, c as LoaderCircle, d as ze, f as useLocation, g as require_react, h as useBuiltInText, l as Lt, m as clsx, n as require_jsx_runtime, o as SearchX, p as useNavigate, t as prefixHref, u as ht } from "../entry-server.js";
//#region node_modules/.pnpm/@clarify-labs+renderer@0.11.0_@types+react@19.2.17_react-dom@19.2.7_react@19.2.7__react-route_f7as6ftkxm7zwmx7isv45mpmvm/node_modules/@clarify-labs/renderer/output/SearchDialog-CBJjDs_9.js
var import_react = /* @__PURE__ */ __toESM(require_react(), 1);
var import_jsx_runtime = require_jsx_runtime();
function routeGroupTitles(navigation) {
	const titles = /* @__PURE__ */ new Map();
	for (const node of navigation) if (node.children?.length) for (const child of node.children) titles.set(child.path, node.title);
	else titles.set(node.path, node.title);
	return titles;
}
function buildSearchItems(routes, navigation) {
	const groupTitles = routeGroupTitles(navigation);
	return routes.flatMap((route) => {
		if (route.isBareAlias) return [];
		const groupTitle = groupTitles.get(route.path);
		return [{
			title: route.title,
			pageTitle: route.title,
			sectionTitle: groupTitle,
			url: route.path,
			keywords: [
				groupTitle,
				route.title,
				route.path,
				route.kind
			].filter(Boolean).join(" ").toLowerCase()
		}, ...route.sections?.map((section) => ({
			title: section.title,
			pageTitle: route.title,
			sectionTitle: groupTitle,
			url: `${route.path}#${section.id}`,
			keywords: [
				groupTitle,
				route.title,
				section.title,
				route.path,
				section.id
			].filter(Boolean).join(" ").toLowerCase()
		})) ?? []];
	});
}
var pagefindPromises = /* @__PURE__ */ new Map();
function pagefindCacheKey(routePrefix, locale) {
	return `${routePrefix || "/"}:${locale || "default"}`;
}
function loadPagefind(routePrefix, locale) {
	if (typeof window === "undefined") return Promise.resolve(null);
	const language = locale ?? document.documentElement.lang;
	const cacheKey = pagefindCacheKey(routePrefix, language);
	const cachedPagefind = pagefindPromises.get(cacheKey);
	if (cachedPagefind) return cachedPagefind;
	const pagefindPromise = import(
		/* @vite-ignore */
		prefixHref("/pagefind/pagefind.js", routePrefix)
).then(async (module) => {
		const previousLanguage = document.documentElement.lang;
		if (language) document.documentElement.lang = language;
		const pagefind = module.createInstance();
		if (previousLanguage && previousLanguage !== language) document.documentElement.lang = previousLanguage;
		await pagefind.init?.();
		return pagefind;
	}).catch((error) => {
		pagefindPromises.delete(cacheKey);
		console.warn("[clarify:search] Pagefind instance failed", {
			cacheKey,
			language,
			error
		});
		return null;
	});
	pagefindPromises.set(cacheKey, pagefindPromise);
	return pagefindPromise;
}
function normalizePagefindUrl(url, routePrefix) {
	const parsedUrl = new URL(url, typeof window === "undefined" ? "http://localhost" : window.location.origin);
	let pathname = parsedUrl.pathname;
	const prefix = routePrefix && routePrefix !== "/" ? `/${routePrefix.replace(/^\/+|\/+$/g, "")}` : "";
	if (prefix && (pathname === prefix || pathname.startsWith(`${prefix}/`))) pathname = pathname.slice(prefix.length) || "/";
	return `${pathname}${parsedUrl.search}${parsedUrl.hash}`;
}
var SearchInput = (0, import_react.forwardRef)(function SearchInput(arg0, inputRef) {
	const { query, setQuery, onClose, onSubmit, activeDescendantId, loading = false } = arg0;
	const t = useBuiltInText();
	return /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
		className: "clarify-search-input group relative flex h-12",
		children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)(loading ? LoaderCircle : Search, {
			"aria-hidden": "true",
			className: clsx("pointer-events-none absolute top-0 left-3 h-full w-5 stroke-(--clarify-theme-tokens-colors-muted)", loading && "animate-spin")
		}), /* @__PURE__ */ (0, import_jsx_runtime.jsx)("input", {
			ref: inputRef,
			"data-autofocus": true,
			role: "combobox",
			"aria-expanded": query !== "",
			"aria-controls": "clarify-search-results",
			"aria-activedescendant": activeDescendantId,
			value: query,
			onChange: (event) => setQuery(event.currentTarget.value),
			onKeyDown: (event) => {
				if (event.key === "Escape") if (query) setQuery("");
				else onClose();
				if (event.key === "Enter") {
					event.preventDefault();
					onSubmit();
				}
			},
			placeholder: t("search.placeholder"),
			className: "clarify-search-field flex-auto appearance-none bg-transparent pr-4 pl-10 outline-hidden focus:w-full focus:flex-none [&::-webkit-search-cancel-button]:hidden [&::-webkit-search-decoration]:hidden [&::-webkit-search-results-button]:hidden [&::-webkit-search-results-decoration]:hidden"
		})]
	});
});
function stripHtml(value) {
	if (typeof document === "undefined") return value.replace(/<[^>]*>/g, "");
	const template = document.createElement("template");
	template.innerHTML = value;
	return template.content.textContent ?? "";
}
function escapeRegExp(value) {
	return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
function HighlightQuery(arg0) {
	const { text, query } = arg0;
	if (!query) return text;
	return /* @__PURE__ */ (0, import_jsx_runtime.jsx)(import_jsx_runtime.Fragment, { children: text.split(new RegExp(`(${escapeRegExp(query)})`, "ig")).map((part, index) => part.toLowerCase() === query.toLowerCase() ? /* @__PURE__ */ (0, import_jsx_runtime.jsx)("mark", {
		className: "bg-transparent text-(--clarify-theme-tokens-colors-primary) underline",
		children: part
	}, index) : /* @__PURE__ */ (0, import_jsx_runtime.jsx)(import_react.Fragment, { children: part }, index)) });
}
function renderExcerptNode(node, key) {
	if (node.nodeType === Node.TEXT_NODE) return node.textContent;
	if (node.nodeType !== Node.ELEMENT_NODE) return null;
	const element = node;
	const children = Array.from(element.childNodes).map((child, index) => renderExcerptNode(child, `${key}-${index}`));
	if (element.tagName.toLowerCase() === "mark") return /* @__PURE__ */ (0, import_jsx_runtime.jsx)("mark", {
		className: "bg-transparent text-(--clarify-theme-tokens-colors-primary) underline",
		children
	}, key);
	return /* @__PURE__ */ (0, import_jsx_runtime.jsx)(import_react.Fragment, { children }, key);
}
function HighlightExcerpt(arg0) {
	const { excerpt, query } = arg0;
	if (typeof document === "undefined" || !excerpt.includes("<mark")) return /* @__PURE__ */ (0, import_jsx_runtime.jsx)(HighlightQuery, {
		text: stripHtml(excerpt),
		query
	});
	const template = document.createElement("template");
	template.innerHTML = excerpt;
	return /* @__PURE__ */ (0, import_jsx_runtime.jsx)(import_jsx_runtime.Fragment, { children: Array.from(template.content.childNodes).map((node, index) => renderExcerptNode(node, String(index))) });
}
function SearchResult(arg0) {
	const { result, query, active, onActive, onSelect } = arg0;
	const id = (0, import_react.useId)();
	const hierarchy = result.type === "quick" ? [result.sectionTitle, result.pageTitle].filter((value) => typeof value === "string") : [];
	return /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("li", {
		id,
		className: clsx("clarify-search-result group block cursor-pointer px-4 py-3", active && "clarify-search-result-active"),
		"aria-selected": active,
		onMouseEnter: onActive,
		onMouseMove: onActive,
		onMouseDown: (event) => {
			event.preventDefault();
			onSelect();
		},
		children: [
			/* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
				className: "clarify-search-result-title",
				children: /* @__PURE__ */ (0, import_jsx_runtime.jsx)(HighlightQuery, {
					text: result.title,
					query
				})
			}),
			hierarchy.length > 0 ? /* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
				className: "clarify-search-result-hierarchy mt-1 truncate whitespace-nowrap",
				children: hierarchy.map((item, itemIndex, items) => /* @__PURE__ */ (0, import_jsx_runtime.jsxs)(import_react.Fragment, { children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)(HighlightQuery, {
					text: item,
					query
				}), /* @__PURE__ */ (0, import_jsx_runtime.jsx)("span", {
					className: itemIndex === items.length - 1 ? "sr-only" : "clarify-search-result-separator mx-2",
					children: "/"
				})] }, `${item}-${itemIndex}`))
			}) : null,
			result.type === "full-text" && result.excerpt ? /* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
				className: "clarify-search-result-excerpt mt-1 line-clamp-2 text-xs leading-5 text-(--clarify-theme-tokens-colors-muted)",
				children: /* @__PURE__ */ (0, import_jsx_runtime.jsx)(HighlightExcerpt, {
					excerpt: result.excerpt,
					query
				})
			}) : null
		]
	});
}
function useSearchDialogLifecycle(arg0) {
	const { open, setOpen } = arg0;
	const location = useLocation();
	(0, import_react.useEffect)(() => {
		setOpen(false);
	}, [
		location.pathname,
		location.search,
		location.hash,
		setOpen
	]);
	(0, import_react.useEffect)(() => {
		if (typeof window === "undefined") return void 0;
		if (open) return void 0;
		function onKeyDown(event) {
			if (event.key === "k" && (event.metaKey || event.ctrlKey)) {
				event.preventDefault();
				setOpen(true);
			}
		}
		window.addEventListener("keydown", onKeyDown);
		return () => window.removeEventListener("keydown", onKeyDown);
	}, [open, setOpen]);
}
function useSearchData(arg0) {
	const { query, searchItems, routePrefix, currentLocale } = arg0;
	const [fullTextState, setFullTextState] = (0, import_react.useState)(null);
	const [pagefindState, setPagefindState] = (0, import_react.useState)(null);
	const quickResults = (0, import_react.useMemo)(() => {
		const normalizedQuery = query.trim().toLowerCase();
		if (!normalizedQuery) return [];
		return searchItems.filter((item) => item.keywords.includes(normalizedQuery)).slice(0, 8).map((item) => ({
			...item,
			type: "quick"
		}));
	}, [query, searchItems]);
	const pagefindKey = pagefindCacheKey(routePrefix, currentLocale);
	const activePagefindState = pagefindState?.key === pagefindKey ? pagefindState : null;
	const pagefind = activePagefindState?.pagefind ?? null;
	const pagefindAvailable = activePagefindState?.available ?? null;
	const trimmedQuery = query.trim();
	const fullTextKey = `${pagefindKey}:${trimmedQuery}`;
	const results = (fullTextState?.key === fullTextKey ? fullTextState.results : null) ?? quickResults;
	const searchInputLoading = pagefindAvailable === null || Boolean(trimmedQuery && pagefind && fullTextState?.key !== fullTextKey);
	const showNoResults = Boolean(query.trim()) && results.length === 0 && pagefindAvailable !== null;
	(0, import_react.useEffect)(() => {
		let cancelled = false;
		loadPagefind(routePrefix, currentLocale).then((loadedPagefind) => {
			if (cancelled) return;
			setPagefindState({
				key: pagefindKey,
				pagefind: loadedPagefind,
				available: Boolean(loadedPagefind)
			});
		});
		return () => {
			cancelled = true;
		};
	}, [
		currentLocale,
		pagefindKey,
		routePrefix
	]);
	(0, import_react.useEffect)(() => {
		let cancelled = false;
		if (!trimmedQuery || !pagefind) return void 0;
		Promise.resolve().then(async () => {
			if (cancelled) return;
			try {
				const search = await pagefind.search(trimmedQuery, { limit: 8 });
				const data = await Promise.all(search.results.map(async (result) => ({
					id: result.id,
					data: await result.data()
				})));
				if (cancelled) return;
				setFullTextState({
					key: fullTextKey,
					results: data.map(({ id, data }) => ({
						type: "full-text",
						id,
						title: data.meta.title || data.url,
						url: normalizePagefindUrl(data.url, routePrefix),
						excerpt: data.excerpt
					}))
				});
			} catch (error) {
				console.warn("[clarify:search] Pagefind search failed", {
					currentLocale,
					query: trimmedQuery,
					error
				});
				if (!cancelled) setFullTextState({
					key: fullTextKey,
					results: null
				});
			}
		});
		return () => {
			cancelled = true;
		};
	}, [
		currentLocale,
		fullTextKey,
		pagefind,
		routePrefix,
		trimmedQuery
	]);
	return {
		results,
		searchInputLoading,
		showNoResults
	};
}
function SearchDialog(arg0) {
	const { open, setOpen, routes, navigation, routePrefix = "/", currentLocale, className, onNavigate = () => {} } = arg0;
	const t = useBuiltInText();
	const navigate = useNavigate();
	const inputRef = (0, import_react.useRef)(null);
	const [query, setQuery] = (0, import_react.useState)("");
	const [activeIndex, setActiveIndex] = (0, import_react.useState)(0);
	const searchItems = (0, import_react.useMemo)(() => buildSearchItems(routes, navigation), [navigation, routes]);
	useSearchDialogLifecycle({
		open,
		setOpen
	});
	const { results, searchInputLoading, showNoResults } = useSearchData({
		query,
		searchItems,
		routePrefix,
		currentLocale
	});
	const updateQuery = (value) => {
		setQuery(value);
		setActiveIndex(0);
	};
	function closeDialog() {
		updateQuery("");
		setOpen(false);
	}
	function selectResult(result = results[activeIndex]) {
		if (!result) return;
		navigate(result.url);
		onNavigate();
		closeDialog();
	}
	function renderResultsPanel() {
		return /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
			className: "border-t border-(--clarify-theme-tokens-colors-border) bg-(--clarify-theme-tokens-colors-background) empty:hidden dark:border-zinc-100/5 dark:bg-white/2.5",
			children: [showNoResults ? /* @__PURE__ */ (0, import_jsx_runtime.jsxs)("div", {
				className: "p-6 text-center",
				children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)(SearchX, { className: "mx-auto h-5 w-5 stroke-(--clarify-theme-tokens-colors-foreground) dark:stroke-zinc-600" }), /* @__PURE__ */ (0, import_jsx_runtime.jsx)("p", {
					className: "mt-2 text-xs text-(--clarify-theme-tokens-colors-muted) dark:text-zinc-400",
					children: t("search.noResults", { query })
				})]
			}) : null, results.length ? /* @__PURE__ */ (0, import_jsx_runtime.jsx)("ul", {
				id: "clarify-search-results",
				className: "clarify-search-results",
				role: "listbox",
				children: results.map((result, resultIndex) => /* @__PURE__ */ (0, import_jsx_runtime.jsx)(SearchResult, {
					result,
					query,
					active: resultIndex === activeIndex,
					onActive: () => setActiveIndex(resultIndex),
					onSelect: () => selectResult(result)
				}, result.type === "full-text" ? result.id : result.url))
			}) : null]
		});
	}
	return /* @__PURE__ */ (0, import_jsx_runtime.jsxs)(ht, {
		open,
		onClose: closeDialog,
		className: clsx("clarify-search-dialog fixed inset-0 z-50", className),
		children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)(Lt, {
			transition: true,
			className: "clarify-search-backdrop clarify-ui-backdrop fixed inset-0 backdrop-blur-xs data-closed:opacity-0 data-enter:duration-300 data-enter:ease-out data-leave:duration-200 data-leave:ease-in"
		}), /* @__PURE__ */ (0, import_jsx_runtime.jsx)("div", {
			className: "fixed inset-0 overflow-y-auto px-4 py-4 sm:px-6 sm:py-20 md:py-32 lg:px-8 lg:py-(--clarify-search-dialog-padding-block-lg)",
			children: /* @__PURE__ */ (0, import_jsx_runtime.jsxs)(ze, {
				transition: true,
				className: "clarify-search-panel mx-auto transform-gpu overflow-hidden rounded-(--clarify-theme-tokens-radius-lg) bg-(--clarify-theme-tokens-colors-surface) shadow-xl ring-1 ring-(--clarify-theme-tokens-colors-border) data-closed:scale-95 data-closed:opacity-0 data-enter:duration-300 data-enter:ease-out data-leave:duration-200 data-leave:ease-in sm:max-w-xl dark:bg-zinc-900 dark:ring-zinc-800",
				onKeyDown: (event) => {
					if (event.key === "ArrowDown") {
						event.preventDefault();
						setActiveIndex((index) => results.length ? (index + 1) % results.length : 0);
					}
					if (event.key === "ArrowUp") {
						event.preventDefault();
						setActiveIndex((index) => results.length ? (index - 1 + results.length) % results.length : 0);
					}
				},
				children: [/* @__PURE__ */ (0, import_jsx_runtime.jsx)(SearchInput, {
					ref: inputRef,
					query,
					setQuery: updateQuery,
					onClose: closeDialog,
					onSubmit: () => selectResult(),
					loading: searchInputLoading
				}), renderResultsPanel()]
			})
		})]
	});
}
//#endregion
export { SearchDialog, SearchDialog as default };
