import type { APIRoute } from "astro";
import { getEntry } from "astro:content";
import { getArticles } from "../../../utils/getArticles";
import { renderPng } from "../../../utils/renderPng";

export async function getStaticPaths() {
  const articles = await getArticles({ drafts: true });

  return articles
    .filter(post => !!post.data.cover)
    .map((post) => ({
      params: { slug: post.slug },
    }));
}

async function getImageProperties(slug: string) {
  const post = await getEntry("articles", slug);
  if (!post) {
    throw new Error(`Could not get entry for ${slug}`);
  }

  const title = typeof post.data.cover === "object"
    && "title" in post.data.cover
    && post.data.cover.title
    || post.data.title;

  return {
    title,
    properties: [
      { name: "An article by", value: "Benjamin Tan" },
      post.data.publishedAt
        ? {
            name: "Published on",
            value: post.data.publishedAt.toLocaleDateString('en-us', {
              year: 'numeric',
              month: 'long',
              day: 'numeric',
            })
          }
        : { name: "Status", value: "Draft" },
      ...(post.data.updatedAt
            ? [{
                name: "Last updated",
                value: post.data.updatedAt.toLocaleDateString('en-us', {
                  year: 'numeric',
                  month: 'long',
                  day: 'numeric',
                })
              }]
            : [])
    ],
  };
}

export const get: APIRoute = async ({ params }) => {
  const imageProperties = await getImageProperties(params.slug!);

  const image = await renderPng(
    {
      type: "div",
      props: {
        children: [
          {
            type: "div",
            props: {
              children: "ofcrse",
              style: {
                fontSize: 40,
                lineHeight: 1,
                letterSpacing: -1,
                color: "#128886",
              }
            },
          },
          {
            type: "div",
            props: {
              children: imageProperties.title,
              style: {
                whiteSpace: "pre",
                fontSize: 110,
                lineHeight: 1,
                letterSpacing: -2,
                marginBottom: 100,
              }
            },
          },
          {
            type: "div",
            props: {
              children: imageProperties.properties.map(({ name, value }) =>
                ({
                  type: "div",
                  props: {
                    children: [
                      {
                        type: "div",
                        props: {
                          children: name,
                          style: {
                            color: "#83758c",
                            textTransform: "uppercase",
                            fontSize: 28,
                            lineHeight: 1,
                            letterSpacing: -1,
                          }
                        },
                      },
                      {
                        type: "div",
                        props: {
                          children: value,
                          style: {
                            color: "#301940",
                            fontSize: 40,
                            lineHeight: 1,
                          }
                        },
                      },
                    ],
                    style: {
                      display: "flex",
                      flexDirection: "column",
                      gap: 8,
                    }
                  },
                })),
              style: {
                display: "flex",
                gap: 56,
                color: "#83758c",
                fontSize: 50,
              }
            },
          },
        ],
        style: {
          display: "flex",
          height: "100%",
          width: "100%",
          padding: "100px 80px",
          flexDirection: "column",
          alignItems: "stretch",
          justifyContent: "flex-end",
          backgroundColor: "#fdfdfa",
          color: "#301940",
          fontWeight: 400,
        },
      },
    },
  );

  return new Response(image, {
    status: 200,
    headers: {
      "Content-Type": "image/png",
    },
  });
};
