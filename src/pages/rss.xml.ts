import rss from "@astrojs/rss";
import type { APIContext } from "astro";
import MarkdownIt from "markdown-it";
import sanitizeHtml from "sanitize-html";
import { SITE_TITLE, SITE_DESCRIPTION } from "../consts";
import { getArticles } from "../utils/getArticles";

export async function get(context: APIContext) {
	const articles = await getArticles();
	const parser = new MarkdownIt();

	return rss({
		title: SITE_TITLE,
		description: SITE_DESCRIPTION,
		site: context.site as unknown as string,
		trailingSlash: false,
		items: articles.map((post) => ({
			...post.data,
			pubDate: post.data.publishedAt!,
			link: `/${post.slug}`,
			content: sanitizeHtml(parser.render(post.body)),
		})),
	});
}
