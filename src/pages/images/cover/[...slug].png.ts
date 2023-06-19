import type { APIRoute } from "astro";
import { getEntry } from "astro:content";
import fs from "node:fs";
import path from "node:path";
import satori from "satori";
import sharp from "sharp";
import { getArticles } from "../../../utils/getArticles";

const FONT_PATH = path.join(process.cwd(), "public/assets/fonts/basiersquarenarrow-regular-webfont.ttf");
const font = fs.readFileSync(FONT_PATH);

type ImageProperties = {
  title: string;
  subtitle: string;
  titleFontSize: number;
  subtitleFontSize: number;
}

const DEFAULT_COVER = {
  title: "ofcrse",
  subtitle: "is Benjamin Tan’s home on the internet.",
  titleFontSize: 200,
  subtitleFontSize: 70,
};

async function getImageProperties(slug: string): Promise<ImageProperties> {
  if (slug === "default") {
    return DEFAULT_COVER;
  }

  const post = await getEntry("articles", slug);
  if (!post) {
    throw new Error(`Could not get entry for ${slug}`);
  }

  return {
    title: post.data.title,
    subtitle: "By Benjamin Tan",
    titleFontSize: 100,
    subtitleFontSize: 50,
    ...(post.data.cover === true ? {} : post.data.cover),
  };
}

export async function getStaticPaths() {
  const articles = await getArticles({ drafts: true });

  return [
    {
      params: { slug: "default" },
    },
    ...articles
      .filter(post => !!post.data.cover)
      .map((post) => ({
        params: { slug: post.slug },
      }))
  ];
}

export const get: APIRoute = async ({ params }) => {
  const imageProperties = await getImageProperties(params.slug!);

  const svg = await satori(
    {
      type: "div",
      props: {
        children: [
          {
            type: "div",
            props: {
              children: imageProperties.title,
              style: {
                whiteSpace: 'pre',
                fontSize: imageProperties.titleFontSize,
                fontWeight: 400,
                lineHeight: 1,
                letterSpacing: -2,
                marginBottom: 60,
              }
            },
          },
          {
            type: "div",
            props: {
              children: imageProperties.subtitle,
              style: {
                color: "#83758c",
                fontSize: imageProperties.subtitleFontSize,
                fontWeight: 400,
              }
            },
          }
        ],
        style: {
          display: "flex",
          height: "100%",
          width: "100%",
          padding: "120px 80px",
          flexDirection: "column",
          alignItems: "flex-start",
          justifyContent: "flex-end",
          backgroundColor: "#fdfdfa",
          color: "#301940",
        },
      },
    },
    {
      width: 1200,
      height: 630,
      fonts: [
        {
          name: "Basier Square Narrow",
          data: font,
          weight: 400,
          style: "normal",
        },
      ],
    },
  );

  const png = sharp(Buffer.from(svg)).png();
  const response = await png.toBuffer();

  return new Response(response, {
    status: 200,
    headers: {
      "Content-Type": "image/png",
    },
  });
};
