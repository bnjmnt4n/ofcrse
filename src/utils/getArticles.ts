import { getCollection } from "astro:content";

export async function getArticles(options = { drafts: false }) {
	const posts = await getCollection("articles");

	if (options.drafts) {
		return posts;
	} else {
		return posts.filter((post) => !!post.data.publishedAt);
	}
}
